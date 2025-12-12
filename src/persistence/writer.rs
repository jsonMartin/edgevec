use super::header::FileHeader;
use crate::hnsw::HnswConfig;

/// Writes an empty index (header only) to a byte vector.
///
/// # Arguments
///
/// * `config` - The HNSW configuration to store in the header.
///
/// # Returns
///
/// A `Vec<u8>` containing the serialized 64-byte header.
#[must_use]
pub fn write_empty_index(config: &HnswConfig) -> Vec<u8> {
    let mut header = FileHeader::new(config.dimensions);

    // Update fields from config
    header.hnsw_m = config.m;
    header.hnsw_m0 = config.m0;
    header.vector_count = 0;
    header.index_offset = 64; // Immediately follows header

    // Calculate CRC
    header.update_checksum();

    header.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::MAGIC;

    #[test]
    fn test_write_empty_index() {
        let config = HnswConfig::new(128);
        let bytes = write_empty_index(&config);

        assert_eq!(bytes.len(), 64);
        assert_eq!(&bytes[0..4], &MAGIC);

        // Verify dimensions
        let dims = u32::from_le_bytes(bytes[40..44].try_into().unwrap());
        assert_eq!(dims, 128);

        // Verify CRC is set (non-zero for non-empty header)
        let crc = u32::from_le_bytes(bytes[44..48].try_into().unwrap());
        assert_ne!(crc, 0);

        // Verify it parses back correctly
        let header = FileHeader::from_bytes(&bytes).expect("Should parse valid header");
        assert_eq!(header.dimensions, 128);
        assert_eq!(header.hnsw_m, 12);
    }
}
