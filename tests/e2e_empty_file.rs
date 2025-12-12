use edgevec::hnsw::HnswConfig;
use edgevec::persistence::{read_index_header, write_empty_index, PersistenceError};
use std::io::{Read, Write};
use tempfile::NamedTempFile;

#[test]
fn test_e2e_empty_file_lifecycle() {
    // 1. Setup Config
    let config = HnswConfig {
        m: 16,
        m0: 32,
        ef_construction: 100,
        ef_search: 50,
        dimensions: 128,
        metric: 0,
        _reserved: [0; 2],
    };

    // 2. Write to Memory
    let bytes = write_empty_index(&config);
    assert_eq!(bytes.len(), 64, "Empty index must be exactly header size");

    // 3. Write to Disk (using tempfile)
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(&bytes)
        .expect("Failed to write to temp file");
    temp_file.flush().expect("Failed to flush file");

    // 4. Verify File Size
    let file_size = temp_file
        .as_file()
        .metadata()
        .expect("Failed to get metadata")
        .len();
    assert_eq!(file_size, 64, "File size on disk must match header size");

    // 5. Read back from Disk
    let mut file_content = Vec::new();
    {
        // Re-open purely for reading to verify path access, or just seek to 0
        // NamedTempFile handles the path automatically
        let mut file = std::fs::File::open(temp_file.path()).expect("Failed to open temp file");
        file.read_to_end(&mut file_content)
            .expect("Failed to read temp file");
    }

    // 6. Parse Header
    let read_config = read_index_header(&file_content).expect("Failed to parse valid header");

    // 7. Verify Config Matches
    assert_eq!(read_config.dimensions, config.dimensions);
    assert_eq!(read_config.m, config.m);
    assert_eq!(read_config.m0, config.m0);

    // 8. Manual Hex Inspection (Magic Number)
    assert_eq!(
        &file_content[0..4],
        &[0x45, 0x56, 0x45, 0x43],
        "Magic number mismatch"
    ); // E V E C

    // 9. Verify Corruption (CRC32)
    {
        // Corrupt dimension byte (offset 40)
        let mut corrupted_content = file_content.clone();
        corrupted_content[40] ^= 0xFF;

        let result = read_index_header(&corrupted_content);
        match result {
            Err(PersistenceError::ChecksumMismatch { .. }) => (), // Expected
            Err(e) => panic!("Expected ChecksumMismatch, got {:?}", e),
            Ok(_) => panic!("Expected error for corrupted file, got success"),
        }
    }

    // Cleanup handled automatically by NamedTempFile drop
}
