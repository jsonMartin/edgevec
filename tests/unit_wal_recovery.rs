use edgevec::persistence::entry::WalEntry;
use edgevec::persistence::storage::StorageBackend;
use edgevec::persistence::wal::{WalAppender, WalError, WalIterator, CRC_SIZE, WAL_HEADER_SIZE};
use edgevec::persistence::PersistenceError;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Mock backend for testing WAL with full control over the buffer.
#[derive(Debug, Clone)]
struct MockBackend {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl MockBackend {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl StorageBackend for MockBackend {
    fn append(&mut self, data: &[u8]) -> Result<(), PersistenceError> {
        let mut buf = self.buffer.lock().unwrap();
        buf.extend_from_slice(data);
        Ok(())
    }

    fn read(&self) -> Result<Vec<u8>, PersistenceError> {
        let buf = self.buffer.lock().unwrap();
        Ok(buf.clone())
    }

    fn atomic_write(&self, _key: &str, data: &[u8]) -> Result<(), PersistenceError> {
        let mut buf = self.buffer.lock().unwrap();
        buf.clear();
        buf.extend_from_slice(data);
        Ok(())
    }
}

#[test]
fn test_wal_constants_verification() {
    assert_eq!(
        WAL_HEADER_SIZE, 16,
        "WAL_HEADER_SIZE must be 16 to match WalEntry layout"
    );
    assert_eq!(CRC_SIZE, 4, "CRC_SIZE must be 4");

    // Also verify WalEntry size matches header size
    use std::mem::size_of;
    assert_eq!(
        size_of::<WalEntry>(),
        WAL_HEADER_SIZE,
        "WalEntry struct size must match WAL_HEADER_SIZE"
    );
}

#[test]
fn test_wal_verify_with_mock() {
    // 1. Setup
    let backend = MockBackend::new();
    // Keep a clone to inspect buffer later
    let backend_ref = backend.clone();

    let mut appender = WalAppender::new(Box::new(backend), 0);

    // 2. Append 100 entries
    let count = 100;
    for i in 0..count {
        let payload = (i as u32).to_le_bytes();
        appender.append(0, &payload).expect("append failed");
    }

    // 3. Inspect Buffer Directly
    let buffer = backend_ref.buffer.lock().unwrap();
    let mut cursor = Cursor::new(&*buffer);

    // WalIterator reads from a Reader, so we can verify using the iterator too,
    // but the task asks to "Assert: Mock buffer contains expected data".
    // We can manually parse or use WalIterator to parse and verify CRCs.
    // The iterator *does* verify CRCs.

    let iterator = WalIterator::new(&mut cursor);

    let mut read_count = 0;
    for (i, result) in iterator.enumerate() {
        let (entry, payload) = result.expect("Entry should be valid");

        // Verify Content
        assert_eq!(entry.sequence, i as u64);
        assert_eq!(entry.entry_type, 0);
        assert_eq!(payload, (i as u32).to_le_bytes());

        read_count += 1;
    }
    assert_eq!(read_count, count);
}

#[test]
fn test_wal_corruption_handling() {
    // 1. Setup
    let backend = MockBackend::new();
    let backend_ref = backend.clone();

    let mut appender = WalAppender::new(Box::new(backend), 0);

    // Write one valid entry
    let payload = b"valid_entry";
    appender.append(0, payload).expect("append failed");

    // 2. Corrupt the buffer manually
    {
        let mut buf = backend_ref.buffer.lock().unwrap();
        // The format is: Header(16) + Payload + CRC(4)
        // Entry header is 16 bytes. Payload "valid_entry" is 11 bytes. CRC is 4 bytes.
        // Total = 16 + 11 + 4 = 31 bytes.

        // Let's corrupt the CRC (last 4 bytes)
        let len = buf.len();
        if len > 0 {
            buf[len - 1] ^= 0xFF; // Flip last byte of CRC
        }
    }

    // 3. Attempt to read
    let buffer = backend_ref.buffer.lock().unwrap();
    let cursor = Cursor::new(&*buffer);
    let mut iterator = WalIterator::new(cursor);

    let result = iterator.next();

    // 4. Assert Error
    match result {
        Some(Err(WalError::ChecksumMismatch { .. })) => {
            // Expected
        }
        res => panic!("Expected ChecksumMismatch, got {:?}", res),
    }
}

#[test]
fn test_wal_truncation_handling() {
    // 1. Setup
    let backend = MockBackend::new();
    let backend_ref = backend.clone();

    let mut appender = WalAppender::new(Box::new(backend), 0);

    // Write one valid entry
    appender.append(0, b"valid").expect("append failed");

    // 2. Truncate the buffer (remove last byte of CRC)
    {
        let mut buf = backend_ref.buffer.lock().unwrap();
        buf.pop();
    }

    // 3. Attempt to read
    let buffer = backend_ref.buffer.lock().unwrap();
    let cursor = Cursor::new(&*buffer);
    let mut iterator = WalIterator::new(cursor);

    let result = iterator.next();

    // 4. Assert Truncated Error
    match result {
        Some(Err(WalError::Truncated { .. })) => {
            // Expected
        }
        res => panic!("Expected Truncated error, got {:?}", res),
    }
}
