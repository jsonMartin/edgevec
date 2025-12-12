use bytemuck::{Pod, Zeroable};
use core::mem::{align_of, size_of};
use thiserror::Error;

/// Magic number: "EVEC" = [0x45, 0x56, 0x45, 0x43]
pub const MAGIC: [u8; 4] = *b"EVEC";

/// Current major version
pub const VERSION_MAJOR: u8 = 0;

/// Current minor version
pub const VERSION_MINOR: u8 = 1;

/// File header for .evec index files.
///
/// # Layout
///
/// Total size: 64 bytes
/// Alignment: 8 bytes
///
/// # Invariants
///
/// - `magic` must be "EVEC"
/// - `version_major` must match current version
///
/// # Thread Safety
///
/// This type is `Send + Sync` as it is a POD struct.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct FileHeader {
    /// Magic number: "EVEC" = [0x45, 0x56, 0x45, 0x43]
    pub magic: [u8; 4], // 0

    /// Format version major part.
    pub version_major: u8, // 4

    /// Format version minor part.
    pub version_minor: u8, // 5

    /// Flags (bit 0: compressed, bit 1: quantized, etc.)
    pub flags: u16, // 6

    /// Number of vectors in file
    pub vector_count: u64, // 8

    /// Byte offset to index section
    pub index_offset: u64, // 16

    /// Byte offset to metadata section (0 if none)
    pub metadata_offset: u64, // 24

    /// RNG Seed for deterministic replay
    pub rng_seed: u64, // 32

    /// Vector dimensionality
    pub dimensions: u32, // 40

    /// CRC32 of header bytes
    pub header_crc: u32, // 44

    /// HNSW M parameter
    pub hnsw_m: u32, // 48

    /// HNSW M0 parameter
    pub hnsw_m0: u32, // 52

    /// CRC32 of data payload
    pub data_crc: u32, // 56

    /// Reserved for future use
    pub reserved: u32, // 60
}

// Static assertions for size and alignment
const _: () = assert!(size_of::<FileHeader>() == 64);
const _: () = assert!(align_of::<FileHeader>() == 8);

/// Errors that can occur during header parsing.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HeaderError {
    /// Invalid magic number.
    #[error("invalid magic number: expected 'EVEC', got {0:?}")]
    InvalidMagic([u8; 4]),

    /// Unsupported version.
    #[error("unsupported version: {0}.{1}")]
    UnsupportedVersion(u8, u8),

    /// Checksum mismatch.
    #[error("checksum mismatch: expected {expected:#x}, got {actual:#x}")]
    ChecksumMismatch {
        /// Expected checksum (from header)
        expected: u32,
        /// Actual calculated checksum
        actual: u32,
    },

    /// Buffer too short.
    #[error("buffer too short: expected 64 bytes, got {0}")]
    BufferTooShort(usize),

    /// Buffer is not 8-byte aligned.
    #[error("buffer is not 8-byte aligned")]
    UnalignedBuffer,
}

impl FileHeader {
    /// The expected magic bytes "EVEC".
    pub const MAGIC: [u8; 4] = MAGIC;
    /// The current major version.
    pub const VERSION_MAJOR: u8 = VERSION_MAJOR;
    /// The current minor version.
    pub const VERSION_MINOR: u8 = VERSION_MINOR;

    /// Creates a new `FileHeader` with default values.
    #[must_use]
    pub fn new(dimensions: u32) -> Self {
        let mut header = Self {
            magic: MAGIC,
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            flags: 0,
            vector_count: 0,
            index_offset: 0,
            metadata_offset: 0,
            rng_seed: 0,
            dimensions,
            header_crc: 0,
            hnsw_m: 16,
            hnsw_m0: 32,
            data_crc: 0,
            reserved: 0,
        };
        header.update_checksum();
        header
    }

    /// Returns the byte representation of the header.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 64] {
        bytemuck::cast_ref(self)
    }

    /// Parses a `FileHeader` from bytes.
    ///
    /// # Requirements
    ///
    /// - `bytes` must be at least 64 bytes
    /// - `bytes` must be 8-byte aligned
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - Buffer is less than 64 bytes (`BufferTooShort`)
    /// - Buffer is not 8-byte aligned (`UnalignedBuffer`)
    /// - Magic number is invalid (`InvalidMagic`)
    /// - Version is unsupported (`UnsupportedVersion`)
    /// - Checksum mismatch (`ChecksumMismatch`)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HeaderError> {
        if bytes.len() < 64 {
            return Err(HeaderError::BufferTooShort(bytes.len()));
        }

        // SAFETY: Length is checked above; alignment is validated via `try_from_bytes`.
        //
        // This function is safe because:
        // - Buffer length is verified to be exactly 64 bytes
        // - FileHeader is Pod + Zeroable (all bit patterns valid)
        // - bytemuck::try_from_bytes validates alignment
        let header = *bytemuck::try_from_bytes::<FileHeader>(&bytes[..64])
            .map_err(|_| HeaderError::UnalignedBuffer)?;

        if header.magic != MAGIC {
            return Err(HeaderError::InvalidMagic(header.magic));
        }

        if header.version_major != VERSION_MAJOR {
            return Err(HeaderError::UnsupportedVersion(
                header.version_major,
                header.version_minor,
            ));
        }

        // Verify checksum
        let mut verify_header = header;
        verify_header.header_crc = 0;
        let calculated_crc = crc32fast::hash(verify_header.as_bytes());

        if header.header_crc != calculated_crc {
            return Err(HeaderError::ChecksumMismatch {
                expected: header.header_crc,
                actual: calculated_crc,
            });
        }

        Ok(header)
    }

    /// Updates the checksum based on current fields.
    pub fn update_checksum(&mut self) {
        self.header_crc = 0;
        self.header_crc = crc32fast::hash(self.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_layout() {
        assert_eq!(size_of::<FileHeader>(), 64);
        assert_eq!(core::mem::align_of::<FileHeader>(), 8);
    }

    #[test]
    fn test_new_header_validity() {
        let header = FileHeader::new(128);
        assert_eq!(header.magic, MAGIC);
        assert_eq!(header.dimensions, 128);

        // Checksum should be set
        assert_ne!(header.header_crc, 0);

        // Should be valid
        let bytes = header.as_bytes();
        let decoded = FileHeader::from_bytes(bytes).unwrap();
        assert_eq!(decoded.dimensions, 128);
    }

    #[test]
    fn test_invalid_magic() {
        let mut header = FileHeader::new(128);
        header.magic = [0x00, 0x00, 0x00, 0x00];
        header.update_checksum(); // Recalculate checksum so that's not the error

        let bytes = header.as_bytes();
        let result = FileHeader::from_bytes(bytes);
        assert!(matches!(result, Err(HeaderError::InvalidMagic(_))));
    }

    #[test]
    fn test_checksum_mismatch() {
        let mut header = FileHeader::new(128);
        // Corrupt a field without updating checksum
        header.dimensions = 256;

        let bytes = header.as_bytes();
        let result = FileHeader::from_bytes(bytes);
        assert!(matches!(result, Err(HeaderError::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_unaligned_buffer_rejected() {
        let header = FileHeader::new(64);
        let mut buf = Vec::with_capacity(65);
        buf.push(0); // create an offset to force misalignment
        buf.extend_from_slice(header.as_bytes());

        let slice = &buf[1..65];
        let result = FileHeader::from_bytes(slice);
        assert!(matches!(result, Err(HeaderError::UnalignedBuffer)));
    }
}
