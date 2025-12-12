use edgevec::persistence::entry::WalEntry;
use edgevec::persistence::wal::{WalError, WalIterator, MAX_PAYLOAD_SIZE};
use proptest::prelude::*;
use std::io::{Cursor, Write};

// Helper to verify durability. Mimics the missing WalAppender.
// This ensures we test the WalIterator against a compliant writer.
struct WalAppender<W> {
    writer: W,
    sequence: u64,
}

impl<W: Write> WalAppender<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            sequence: 0,
        }
    }

    fn append(&mut self, payload: &[u8]) -> std::io::Result<()> {
        let entry = WalEntry::new(self.sequence, 0, payload.len() as u32);
        self.sequence += 1;

        // 1. Write Header
        self.writer.write_all(entry.as_bytes())?;

        // 2. Write Payload
        self.writer.write_all(payload)?;

        // 3. Compute & Write CRC
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(entry.as_bytes());
        hasher.update(payload);
        let crc = hasher.finalize();
        self.writer.write_all(&crc.to_le_bytes())?;

        Ok(())
    }
}

#[test]
fn test_payload_size_boundary() {
    // UNIT TEST: Boundary checks for payload size
    // We mock the header to avoid allocating 16MB+ in memory.

    let check_size = |size: u32| -> Result<(), WalError> {
        let entry = WalEntry::new(0, 0, size);
        let entry_bytes = entry.as_bytes();
        // We only provide the header.
        // If size is valid, it will try to read payload and fail with Truncated.
        // If size is invalid, it will fail with PayloadTooLarge immediately.
        let cursor = Cursor::new(entry_bytes);
        let mut iter = WalIterator::new(cursor);
        match iter.next() {
            Some(Ok(_)) => Ok(()),
            Some(Err(e)) => Err(e),
            None => Ok(()),
        }
    };

    // 1. MAX - 1 -> Should pass check (return Truncated because no payload)
    let res = check_size((MAX_PAYLOAD_SIZE - 1) as u32);
    match res {
        Err(WalError::Truncated { .. }) => {}
        _ => panic!("Expected Truncated for MAX - 1, got {:?}", res),
    }

    // 2. MAX -> Should pass check
    let res = check_size(MAX_PAYLOAD_SIZE as u32);
    match res {
        Err(WalError::Truncated { .. }) => {}
        _ => panic!("Expected Truncated for MAX, got {:?}", res),
    }

    // 3. MAX + 1 -> Should fail check
    let res = check_size((MAX_PAYLOAD_SIZE + 1) as u32);
    match res {
        Err(WalError::PayloadTooLarge { .. }) => {}
        _ => panic!("Expected PayloadTooLarge for MAX + 1, got {:?}", res),
    }
}

proptest! {
    // The Boss Fight requires extensive testing
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn prop_wal_clean_shutdown(
        payloads in prop::collection::vec(any::<Vec<u8>>(), 0..100)
    ) {
        // 1. Write
        let mut buffer = Vec::new();
        let mut appender = WalAppender::new(&mut buffer);
        for payload in &payloads {
            appender.append(payload).unwrap();
        }

        // 2. Read
        let cursor = Cursor::new(buffer);
        let iterator = WalIterator::new(cursor);

        let mut recovered_payloads = Vec::new();
        for item in iterator {
            let (_, payload) = item.unwrap();
            recovered_payloads.push(payload);
        }

        // 3. Assert
        assert_eq!(payloads, recovered_payloads);
    }

    #[test]
    fn prop_wal_torn_write(
        payloads in prop::collection::vec(any::<Vec<u8>>(), 1..50), // At least 1 to have something to cut
        cut_point_strategy in any::<prop::sample::Index>()
    ) {
         // 1. Write full WAL
        let mut buffer = Vec::new();
        let mut appender = WalAppender::new(&mut buffer);
        let mut entry_boundaries = Vec::new(); // To track valid cutoff points
        let mut current_len = 0;

        for payload in &payloads {
            appender.append(payload).unwrap();
            current_len += 16 + payload.len() + 4; // Header(16) + Payload + CRC(4)
            entry_boundaries.push(current_len);
        }

        // 2. Truncate
        let cut_point = cut_point_strategy.index(buffer.len() + 1); // +1 to include full length
        let mut truncated_buffer = buffer.clone();
        truncated_buffer.truncate(cut_point);

        // 3. Read back
        let cursor = Cursor::new(truncated_buffer);
        let iterator = WalIterator::new(cursor);

        let mut recovered_payloads = Vec::new();
        let mut error_encountered = None;

        for item in iterator {
            match item {
                Ok((_, payload)) => recovered_payloads.push(payload),
                Err(e) => {
                    error_encountered = Some(e);
                    break;
                }
            }
        }

        // 4. Assertions

        // A. Prefix Property: Recovered must be a prefix of Original
        assert!(payloads.starts_with(&recovered_payloads), "Recovered {:?} is not a prefix of Original {:?}", recovered_payloads, payloads);

        // B. Safety / Error Correctness
        let is_boundary = entry_boundaries.contains(&cut_point) || cut_point == 0;

        if is_boundary {
            // If cut at boundary, we expect Clean Shutdown (no error), just fewer items
            assert!(error_encountered.is_none(), "Expected clean shutdown at boundary {}, got error {:?}", cut_point, error_encountered);

            // The number of recovered items should be exactly the number of FULL entries before the cut
            let expected_count = entry_boundaries.iter().filter(|&&b| b <= cut_point).count();
            assert_eq!(recovered_payloads.len(), expected_count, "Boundary cut at {}: expected {} items, got {}", cut_point, expected_count, recovered_payloads.len());
        } else {
            // If cut in middle, we expect Truncated error
            match error_encountered {
                Some(WalError::Truncated { .. }) => {
                    // Good
                },
                Some(e) => panic!("Expected Truncated error at offset {}, got {:?}", cut_point, e),
                None => panic!("Expected Truncated error at offset {}, got success with {} items", cut_point, recovered_payloads.len()),
            }

            // The number of recovered items should be exactly the number of FULL entries before the cut
            let expected_count = entry_boundaries.iter().filter(|&&b| b <= cut_point).count();
            assert_eq!(recovered_payloads.len(), expected_count, "Torn write at {}: expected {} valid items, got {}", cut_point, expected_count, recovered_payloads.len());
        }
    }
}
