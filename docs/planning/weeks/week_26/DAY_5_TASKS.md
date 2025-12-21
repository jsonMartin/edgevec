# Week 26 Day 5: Persistence Read/Write + Migration

**Date:** 2025-12-25
**Focus:** v0.4 snapshot format implementation and backward compatibility
**Estimated Duration:** 10 hours
**Phase:** RFC-002 Core Metadata (Phase 1)

---

## Tasks

### W26.5.1: Update write_snapshot for v0.4

**Objective:** Write metadata section to snapshot file per RFC-002 Persistence Format §4.2.

**Acceptance Criteria:**
- [ ] `version_minor` set to 4 in FileHeader
- [ ] `HAS_METADATA` flag (bit 2) set if metadata non-empty
- [ ] Metadata section appended after tombstone bitvec
- [ ] Metadata section format: MetadataSectionHeader (16 bytes) + serialized data
- [ ] CRC calculated and stored in MetadataSectionHeader
- [ ] Empty metadata → no metadata section, HAS_METADATA flag NOT set
- [ ] Unit tests for write with/without metadata

**Files:**
- `src/persistence/snapshot.rs` (primary — update write_snapshot)
- `tests/persistence_v04.rs` (new file — v0.4 format tests)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**Implementation (from RFC-002 Persistence Format §4.2):**
```rust
pub fn write_snapshot(
    index: &HnswIndex,
    storage: &VectorStorage,
    backend: &mut dyn StorageBackend,
) -> Result<(), PersistenceError> {
    // Build header
    let mut header = FileHeader::new(storage.dimensions());
    header.version_minor = 4;  // v0.4 format
    header.vector_count = storage.len() as u64;
    header.deleted_count = index.deleted_count as u32;
    // ... other header fields ...

    // Check if metadata exists
    let has_metadata = !index.metadata.is_empty();
    if has_metadata {
        header.flags |= Flags::HAS_METADATA;
    }

    // Calculate offsets
    let vector_data_size = storage.len() * storage.dimensions() * 4;
    header.index_offset = (64 + vector_data_size) as u64;  // After header + vectors

    let hnsw_size = calculate_hnsw_size(index);
    header.tombstone_offset = header.index_offset + hnsw_size as u64;

    let tombstone_size = (index.nodes.len() + 7) / 8;

    // Serialize metadata section if present
    let metadata_bytes = if has_metadata {
        Some(serialize_metadata_section(&index.metadata)?)
    } else {
        None
    };

    // Calculate data CRC (all data after header)
    let data_crc = calculate_data_crc(storage, index, metadata_bytes.as_deref());
    header.data_crc = data_crc;

    // Calculate header CRC
    header.header_crc = header.calculate_crc();

    // Write atomically
    let mut buffer = Vec::new();

    // 1. Write header (64 bytes)
    buffer.extend_from_slice(bytemuck::bytes_of(&header));

    // 2. Write vector data
    buffer.extend_from_slice(storage.as_bytes());

    // 3. Write HNSW index
    write_hnsw_to_buffer(&mut buffer, index)?;

    // 4. Write tombstone bitvec
    write_tombstones_to_buffer(&mut buffer, index)?;

    // 5. Write metadata section (if present)
    if let Some(meta_bytes) = metadata_bytes {
        buffer.extend_from_slice(&meta_bytes);
    }

    // Atomic write
    backend.write_all(&buffer)?;

    Ok(())
}

fn serialize_metadata_section(store: &MetadataStore) -> Result<Vec<u8>, PersistenceError> {
    // Serialize metadata to postcard
    let serialized = store.to_postcard()?;
    let crc = crc32fast::hash(&serialized);

    // Build section header
    let header = MetadataSectionHeader::new_postcard(
        serialized.len() as u32,
        crc,
    );

    // Combine header + data
    let mut result = Vec::with_capacity(16 + serialized.len());
    result.extend_from_slice(bytemuck::bytes_of(&header));
    result.extend_from_slice(&serialized);

    Ok(result)
}
```

**Dependencies:**
- W26.4.1 (MetadataSectionHeader)
- W26.4.2 (MetadataStore::to_postcard)

---

### W26.5.2: Update read_snapshot for v0.4

**Objective:** Read metadata section from v0.4 snapshot files per RFC-002 Persistence Format §4.1.

**Acceptance Criteria:**
- [ ] Detects v0.3 vs v0.4 format via `version_minor`
- [ ] v0.3 files: loads successfully with empty MetadataStore
- [ ] v0.4 files without HAS_METADATA: loads with empty MetadataStore
- [ ] v0.4 files with HAS_METADATA: loads metadata section
- [ ] Validates MetadataSectionHeader magic bytes ("META")
- [ ] Validates CRC before deserializing
- [ ] Returns `PersistenceError::Corrupted` on CRC mismatch
- [ ] Unit tests for each scenario

**Files:**
- `src/persistence/snapshot.rs` (primary — update read_snapshot)
- `tests/persistence_v04.rs` (extend with read tests)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**Implementation (from RFC-002 Persistence Format §4.1):**
```rust
pub fn read_snapshot(
    backend: &dyn StorageBackend,
) -> Result<(HnswIndex, VectorStorage), PersistenceError> {
    // Read and validate header
    let header_bytes = backend.read(0, 64)?;
    let header = FileHeader::from_bytes(&header_bytes)?;
    header.validate_magic()?;
    header.validate_crc()?;

    // Read vector data
    let storage = read_vector_storage(backend, &header)?;

    // Read HNSW index
    let mut index = read_hnsw_index(backend, &header)?;

    // Read tombstone bitvec
    read_tombstones(backend, &header, &mut index)?;

    // Check for metadata section
    if header.version_minor >= 4 && (header.flags & Flags::HAS_METADATA != 0) {
        // Calculate metadata section offset
        let tombstone_size = (header.vector_count as usize + 7) / 8;
        let metadata_offset = header.tombstone_offset as usize + tombstone_size;

        // Load metadata section
        let metadata = load_metadata_section(backend, metadata_offset)?;
        index.metadata = metadata;
    } else {
        // v0.3 or v0.4 without metadata
        index.metadata = MetadataStore::new();
    }

    Ok((index, storage))
}

fn load_metadata_section(
    backend: &dyn StorageBackend,
    offset: usize,
) -> Result<MetadataStore, PersistenceError> {
    // Read section header (16 bytes)
    let header_bytes = backend.read(offset, 16)?;
    let header: MetadataSectionHeader = *bytemuck::from_bytes(&header_bytes);

    // Validate magic
    if header.magic != MetadataSectionHeader::MAGIC {
        return Err(PersistenceError::Corrupted(
            "Invalid metadata section magic".into()
        ));
    }

    // Validate version
    if header.version > MetadataSectionHeader::VERSION {
        return Err(PersistenceError::UnsupportedVersion(
            format!("Metadata version {} not supported", header.version)
        ));
    }

    // Read serialized data
    let data = backend.read(offset + 16, header.size as usize)?;

    // Validate CRC
    let actual_crc = crc32fast::hash(&data);
    if actual_crc != header.crc {
        return Err(PersistenceError::Corrupted(
            format!(
                "Metadata CRC mismatch: expected {:#x}, got {:#x}",
                header.crc, actual_crc
            )
        ));
    }

    // Deserialize based on format
    match header.format {
        MetadataSectionHeader::FORMAT_POSTCARD => {
            MetadataStore::from_postcard(&data).map_err(|e| {
                PersistenceError::Corrupted(format!("Postcard decode failed: {}", e))
            })
        }
        MetadataSectionHeader::FORMAT_JSON => {
            MetadataStore::from_json(&data).map_err(|e| {
                PersistenceError::Corrupted(format!("JSON decode failed: {}", e))
            })
        }
        _ => Err(PersistenceError::Corrupted(
            format!("Unknown metadata format: {}", header.format)
        )),
    }
}
```

**Dependencies:**
- W26.4.1 (MetadataSectionHeader)
- W26.4.2 (MetadataStore::from_postcard)
- W26.5.1 (write_snapshot for test fixtures)

---

### W26.5.3: v0.3 → v0.4 migration tests

**Objective:** Verify backward compatibility with v0.3 files per RFC-002 Persistence Format §4.3.

**Acceptance Criteria:**
- [ ] Load v0.3 file: verify succeeds with empty metadata
- [ ] Save as v0.4: verify HAS_METADATA flag NOT set (empty metadata)
- [ ] Reload v0.4: verify empty metadata
- [ ] Add metadata, save, reload: verify metadata present
- [ ] Verify v0.3 reader fails gracefully on v0.4 (version check)
- [ ] Integration test with real v0.3 fixture file

**Files:**
- `tests/migration_v03_v04.rs` (new file — migration tests)
- `tests/fixtures/` (directory for test fixtures if needed)

**Estimated Duration:** 2 hours

**Agent:** TEST_ENGINEER

**Test Cases:**
```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a v0.3 snapshot (for testing)
    fn create_v03_snapshot() -> Vec<u8> {
        // Create a minimal v0.3 format snapshot
        // This tests backward compatibility
        let mut header = FileHeader::new(128);
        header.version_minor = 3;  // v0.3
        header.vector_count = 10;
        // ... populate other fields ...

        // Build snapshot bytes without metadata section
        let mut buffer = Vec::new();
        buffer.extend_from_slice(bytemuck::bytes_of(&header));
        // ... add vector data, hnsw, tombstones ...

        buffer
    }

    #[test]
    fn load_v03_returns_empty_metadata() {
        let v03_bytes = create_v03_snapshot();
        let backend = MemoryBackend::from_bytes(&v03_bytes);

        let (index, _storage) = read_snapshot(&backend).unwrap();

        // v0.3 files have no metadata
        assert!(index.metadata.is_empty());
    }

    #[test]
    fn save_v04_without_metadata_no_flag() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.edgevec");

        // Create index without metadata
        let mut storage = VectorStorage::new(128);
        let mut index = HnswIndex::new(HnswConfig::default());

        storage.push(&[0.0; 128]).unwrap();
        index.insert(&mut storage, &[0.0; 128]).unwrap();

        // Save as v0.4
        let backend = FileBackend::create(&path).unwrap();
        write_snapshot(&index, &storage, &backend).unwrap();

        // Read back header
        let header_bytes = std::fs::read(&path).unwrap();
        let header: FileHeader = *bytemuck::from_bytes(&header_bytes[..64]);

        // Should be v0.4 but WITHOUT HAS_METADATA flag
        assert_eq!(header.version_minor, 4);
        assert_eq!(header.flags & Flags::HAS_METADATA, 0);
    }

    #[test]
    fn save_v04_with_metadata_sets_flag() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.edgevec");

        // Create index WITH metadata
        let mut storage = VectorStorage::new(128);
        let mut index = HnswIndex::new(HnswConfig::default());

        let id = index.insert_with_metadata(
            &mut storage,
            &[0.0; 128],
            [("key".into(), MetadataValue::String("value".into()))].into()
        ).unwrap();

        // Save as v0.4
        let backend = FileBackend::create(&path).unwrap();
        write_snapshot(&index, &storage, &backend).unwrap();

        // Read back header
        let header_bytes = std::fs::read(&path).unwrap();
        let header: FileHeader = *bytemuck::from_bytes(&header_bytes[..64]);

        // Should have HAS_METADATA flag
        assert_eq!(header.version_minor, 4);
        assert_ne!(header.flags & Flags::HAS_METADATA, 0);
    }

    #[test]
    fn roundtrip_with_metadata() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.edgevec");

        // Create index with metadata
        let mut storage = VectorStorage::new(128);
        let mut index = HnswIndex::new(HnswConfig::default());

        let id = index.insert_with_metadata(
            &mut storage,
            &[1.0; 128],
            [
                ("category".into(), MetadataValue::String("books".into())),
                ("price".into(), MetadataValue::Float(29.99)),
            ].into()
        ).unwrap();

        // Save
        let backend = FileBackend::create(&path).unwrap();
        write_snapshot(&index, &storage, &backend).unwrap();

        // Reload
        let backend = FileBackend::open(&path).unwrap();
        let (loaded_index, _) = read_snapshot(&backend).unwrap();

        // Verify metadata preserved
        let meta = loaded_index.get_metadata(id).unwrap();
        assert_eq!(
            meta.get("category"),
            Some(&MetadataValue::String("books".into()))
        );
        assert_eq!(
            meta.get("price"),
            Some(&MetadataValue::Float(29.99))
        );
    }

    #[test]
    fn v03_to_v04_migration() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.edgevec");

        // 1. Create v0.3 snapshot
        let v03_bytes = create_v03_snapshot();
        std::fs::write(&path, &v03_bytes).unwrap();

        // 2. Load as v0.4 reader
        let backend = FileBackend::open(&path).unwrap();
        let (mut index, storage) = read_snapshot(&backend).unwrap();

        // 3. Verify empty metadata
        assert!(index.metadata.is_empty());

        // 4. Add metadata
        index.metadata.insert(0, "migrated", MetadataValue::Boolean(true));

        // 5. Save as v0.4
        let backend = FileBackend::create(&path).unwrap();
        write_snapshot(&index, &storage, &backend).unwrap();

        // 6. Reload and verify
        let backend = FileBackend::open(&path).unwrap();
        let (loaded, _) = read_snapshot(&backend).unwrap();

        assert_eq!(
            loaded.metadata.get(0, "migrated"),
            Some(&MetadataValue::Boolean(true))
        );
    }

    #[test]
    fn corrupted_metadata_crc_fails() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.edgevec");

        // Create valid snapshot with metadata
        let mut storage = VectorStorage::new(128);
        let mut index = HnswIndex::new(HnswConfig::default());
        index.insert_with_metadata(
            &mut storage,
            &[0.0; 128],
            [("key".into(), MetadataValue::String("value".into()))].into()
        ).unwrap();

        let backend = FileBackend::create(&path).unwrap();
        write_snapshot(&index, &storage, &backend).unwrap();

        // Corrupt the metadata section (flip a bit in the serialized data)
        let mut bytes = std::fs::read(&path).unwrap();
        let last_byte = bytes.len() - 1;
        bytes[last_byte] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();

        // Attempt to load
        let backend = FileBackend::open(&path).unwrap();
        let result = read_snapshot(&backend);

        // Should fail with CRC error
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CRC"));
    }
}
```

**Dependencies:**
- W26.5.1 (write_snapshot v0.4)
- W26.5.2 (read_snapshot v0.4)

---

## Day 5 Checklist

- [x] W26.5.1: write_snapshot updated for v0.4
- [x] W26.5.2: read_snapshot updated for v0.4
- [x] W26.5.3: Migration tests pass
- [x] All existing tests pass (`cargo test`)
- [x] All new tests pass (`cargo test persistence_v04 migration`)
- [x] Clippy clean (`cargo clippy -- -D warnings`)
- [x] Formatted (`cargo fmt --check`)

## Day 5 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| v0.3 → v0.4 migration works | Migration tests pass |
| Metadata persists correctly | Round-trip tests pass |
| CRC validation works | Corruption tests pass |

## Week 26 Completion Summary

After completing Day 5, Week 26 delivers:

**Core Metadata API:**
- `insert_with_metadata()` — atomic vector + metadata insert
- `get_metadata()` — retrieve metadata by ID
- `soft_delete()` — automatic metadata cleanup
- `compact()` — metadata compaction
- `search_filtered()` — post-filter with adaptive overfetch

**Persistence v0.4:**
- MetadataSectionHeader (16 bytes)
- Postcard serialization with CRC32
- v0.3 backward compatibility
- v0.3 → v0.4 transparent migration

**Test Coverage:**
- `tests/metadata_insert.rs`
- `tests/metadata_delete.rs`
- `tests/metadata_compact.rs`
- `tests/metadata_search.rs`
- `tests/metadata_serialize.rs`
- `tests/selectivity.rs`
- `tests/persistence_v04.rs`
- `tests/migration_v03_v04.rs`

**Status:** APPROVED (2025-12-21)

**Next:** Week 27 — Binary Quantization implementation

---

*Agent: RUST_ENGINEER + TEST_ENGINEER*
*Status: [PROPOSED]*
