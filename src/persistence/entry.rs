use bytemuck::{Pod, Zeroable};

/// Write-ahead log entry format.
///
/// # Size
/// 16 bytes header + payload
///
/// # Encoding
/// All multi-byte integers are little-endian.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct WalEntry {
    /// Sequence number (monotonically increasing)
    pub sequence: u64, // offset 0

    /// Entry type (0=insert, 1=delete, 2=checkpoint)
    pub entry_type: u8, // offset 8

    /// Padding
    pub _pad: [u8; 3], // offset 9

    /// Payload length in bytes
    pub payload_len: u32, // offset 12
                          // Followed by:
                          // - payload: [u8; payload_len]
                          // - crc32: u32 (of entire entry including header)
}

impl WalEntry {
    /// Creates a new `WalEntry` with the given sequence number and payload length.
    #[must_use]
    pub fn new(sequence: u64, entry_type: u8, payload_len: u32) -> Self {
        Self {
            sequence,
            entry_type,
            _pad: [0; 3],
            payload_len,
        }
    }

    /// Returns the byte representation of the entry header.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 16] {
        bytemuck::cast_ref(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn test_wal_entry_layout() {
        assert_eq!(size_of::<WalEntry>(), 16);
        assert_eq!(align_of::<WalEntry>(), 8);
    }
}
