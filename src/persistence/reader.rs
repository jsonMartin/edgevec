use super::PersistenceError;
use crate::hnsw::HnswConfig;
use crate::persistence::FileHeader;
use crc32fast::Hasher;

/// Reads and validates the index header from a byte slice.
///
/// # Arguments
///
/// * `data` - The byte slice containing the index data.
///
/// # Returns
///
/// The deserialized `HnswConfig` if successful, or a `PersistenceError`.
///
/// # Errors
///
/// Returns `Err` if:
/// - Buffer is smaller than 64 bytes
/// - Magic number is invalid
/// - Checksum mismatch
/// - Version is unsupported
pub fn read_index_header(data: &[u8]) -> Result<HnswConfig, PersistenceError> {
    let header = read_file_header(data)?;

    // Construct Config
    Ok(HnswConfig {
        m: header.hnsw_m,
        m0: header.hnsw_m0,
        ef_construction: 200, // Default, not stored in header
        ef_search: 50,        // Default, not stored in header
        dimensions: header.dimensions,
        metric: 0, // Default to L2
        _reserved: [0; 2],
    })
}

/// Reads and validates the raw `FileHeader` from a byte slice.
///
/// # Arguments
///
/// * `data` - The byte slice.
///
/// # Returns
///
/// The deserialized `FileHeader`.
///
/// # Errors
///
/// Returns `Err` if:
/// - Buffer is smaller than 64 bytes
/// - Magic number is invalid
/// - Checksum mismatch
/// - Version is unsupported
pub fn read_file_header(data: &[u8]) -> Result<FileHeader, PersistenceError> {
    if data.len() < 64 {
        return Err(PersistenceError::BufferTooSmall {
            expected: 64,
            actual: data.len(),
        });
    }

    // Check Magic
    let magic: [u8; 4] = data[0..4]
        .try_into()
        .map_err(|_| PersistenceError::BufferTooSmall {
            expected: 4,
            actual: data.len(),
        })?;

    if magic != FileHeader::MAGIC {
        return Err(PersistenceError::InvalidMagic {
            expected: FileHeader::MAGIC,
            actual: magic,
        });
    }

    // Check CRC
    let stored_crc = u32::from_le_bytes(data[44..48].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 48,
            actual: data.len(),
        }
    })?);

    // Create a mutable copy of the header bytes to zero out CRC for calculation
    let mut header_bytes = [0u8; 64];
    header_bytes.copy_from_slice(&data[0..64]);
    header_bytes[44..48].fill(0); // Zero out CRC field

    let mut hasher = Hasher::new();
    hasher.update(&header_bytes);
    let calculated_crc = hasher.finalize();

    if stored_crc != calculated_crc {
        return Err(PersistenceError::ChecksumMismatch {
            expected: stored_crc,
            actual: calculated_crc,
        });
    }

    // Read Version
    let version_major = data[4];
    let version_minor = data[5];

    if version_major != FileHeader::VERSION_MAJOR {
        return Err(PersistenceError::UnsupportedVersion(
            version_major,
            version_minor,
        ));
    }

    // Extract Fields
    let flags = u16::from_le_bytes(data[6..8].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 8,
            actual: data.len(),
        }
    })?);
    let vector_count = u64::from_le_bytes(data[8..16].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 16,
            actual: data.len(),
        }
    })?);
    let index_offset = u64::from_le_bytes(data[16..24].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 24,
            actual: data.len(),
        }
    })?);
    let metadata_offset = u64::from_le_bytes(data[24..32].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 32,
            actual: data.len(),
        }
    })?);
    let rng_seed = u64::from_le_bytes(data[32..40].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 40,
            actual: data.len(),
        }
    })?);
    let dimensions = u32::from_le_bytes(data[40..44].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 44,
            actual: data.len(),
        }
    })?);
    // header_crc is stored_crc
    let hnsw_m = u32::from_le_bytes(data[48..52].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 52,
            actual: data.len(),
        }
    })?);
    let hnsw_m0 = u32::from_le_bytes(data[52..56].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 56,
            actual: data.len(),
        }
    })?);
    let data_crc = u32::from_le_bytes(data[56..60].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 60,
            actual: data.len(),
        }
    })?);
    // v0.3: deleted_count (was 'reserved' in v0.1/v0.2, always 0)
    let deleted_count = u32::from_le_bytes(data[60..64].try_into().map_err(|_| {
        PersistenceError::BufferTooSmall {
            expected: 64,
            actual: data.len(),
        }
    })?);

    Ok(FileHeader {
        magic,
        version_major,
        version_minor,
        flags,
        vector_count,
        index_offset,
        metadata_offset,
        rng_seed,
        dimensions,
        header_crc: stored_crc,
        hnsw_m,
        hnsw_m0,
        data_crc,
        deleted_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::writer::write_empty_index;

    #[test]
    fn test_read_valid_header() {
        let config = HnswConfig::new(128);
        let bytes = write_empty_index(&config);

        let read_config = read_index_header(&bytes).unwrap();

        assert_eq!(read_config.dimensions, 128);
        assert_eq!(read_config.m, 12);
        assert_eq!(read_config.m0, 24);
    }

    #[test]
    fn test_read_invalid_magic() {
        let mut bytes = [0u8; 64];
        bytes[0] = 0x00; // Break magic

        let result = read_index_header(&bytes);
        assert!(matches!(result, Err(PersistenceError::InvalidMagic { .. })));
    }

    #[test]
    fn test_read_invalid_crc() {
        let config = HnswConfig::new(128);
        let mut bytes = write_empty_index(&config);

        // Corrupt a byte in the header (e.g., dimensions)
        bytes[40] = 0xFF;

        let result = read_index_header(&bytes);
        assert!(matches!(
            result,
            Err(PersistenceError::ChecksumMismatch { .. })
        ));
    }
}
