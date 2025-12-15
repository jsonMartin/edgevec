# Week 16 — Day 5 Tasks

**Date:** Day 5 of Week 16
**Focus:** Persistence Format v3
**Agent:** RUST_ENGINEER
**Status:** [REVISED]

---

## Day Objective

Update the snapshot persistence format to version 3 to persist the `deleted` field and `deleted_count`. Implement forward migration from v2 format.

**Success Criteria:**
- Snapshot format version 3 implemented
- `deleted` field serialized per node
- `deleted_count` in header
- v2 → v3 migration works
- CRC32 checksum still valid

---

## Tasks

### W16.5: Update Persistence Format to v3

**Priority:** P0 (Persistence)
**Estimate:** 6h (2h base × 3x)
**Agent:** RUST_ENGINEER
**Depends On:** W16.1, W16.2

#### Scope

- [ ] **AC16.5.1:** Snapshot header version bumped to 3
- [ ] **AC16.5.2:** `deleted_count` in header
- [ ] **AC16.5.3:** `deleted` field serialized per node
- [ ] **AC16.5.4:** v2 → v3 migration (set deleted=0)
- [ ] **AC16.5.5:** Backward compatibility documented
- [ ] **AC16.5.6:** CRC32 checksum still valid

#### Implementation Specification

**File:** `src/persistence/snapshot.rs` (or relevant persistence file)

##### Format Specification

```
=== EdgeVec Snapshot Format v3 ===

[Header] (32 bytes)
  magic:          [u8; 4]   // [0xED, 0x6E, 0x56, 0x45] "EdnVE"
  version:        u32       // 3 (bumped from 2)
  flags:          u32       // Bit 0: has_tombstones
  dimensions:     u32       // Vector dimensions
  node_count:     u64       // Total nodes (including deleted)
  deleted_count:  u64       // Count of deleted nodes (NEW)

[Config] (variable)
  m:              u32       // HNSW M parameter
  m0:             u32       // HNSW M0 parameter
  ef_construction: u32      // ef_construction parameter
  ef_search:      u32       // ef_search parameter

[Nodes] (16 bytes each)
  For each node:
    vector_id:      u64     // 8 bytes
    neighbor_offset: u32    // 4 bytes
    neighbor_len:   u16     // 2 bytes
    max_layer:      u8      // 1 byte
    deleted:        u8      // 1 byte (NEW - was padding)

[Neighbor Pool] (variable)
  (unchanged from v2)

[Entry Point] (8 bytes)
  entry_node_id:  Option<u32>

[Footer]
  crc32:          u32       // Checksum of all preceding data

=== End Format ===
```

##### Header Struct

```rust
/// Snapshot header version 3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotHeaderV3 {
    /// Magic bytes: [0xED, 0x6E, 0x56, 0x45]
    pub magic: [u8; 4],

    /// Format version (3)
    pub version: u32,

    /// Feature flags
    /// Bit 0: has_tombstones (1 if any deleted nodes)
    pub flags: u32,

    /// Vector dimensions
    pub dimensions: u32,

    /// Total node count (including deleted)
    pub node_count: u64,

    /// Count of deleted nodes (v3)
    pub deleted_count: u64,
}

impl SnapshotHeaderV3 {
    pub const MAGIC: [u8; 4] = [0xED, 0x6E, 0x56, 0x45];
    pub const VERSION: u32 = 3;

    /// Flag indicating tombstones present
    pub const FLAG_HAS_TOMBSTONES: u32 = 1 << 0;

    pub fn new(dimensions: u32, node_count: u64, deleted_count: u64) -> Self {
        let flags = if deleted_count > 0 {
            Self::FLAG_HAS_TOMBSTONES
        } else {
            0
        };

        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            flags,
            dimensions,
            node_count,
            deleted_count,
        }
    }

    pub fn has_tombstones(&self) -> bool {
        self.flags & Self::FLAG_HAS_TOMBSTONES != 0
    }
}
```

##### Serialization

```rust
impl HnswIndex {
    /// Save index to binary format (v3)
    pub fn save<W: Write>(&self, writer: &mut W) -> Result<(), PersistenceError> {
        let mut hasher = Crc32Hasher::new();

        // Write header
        let header = SnapshotHeaderV3::new(
            self.config.dimensions,
            self.nodes.len() as u64,
            self.deleted_count as u64,
        );
        let header_bytes = postcard::to_allocvec(&header)?;
        writer.write_all(&header_bytes)?;
        hasher.update(&header_bytes);

        // Write config
        let config_bytes = postcard::to_allocvec(&self.config)?;
        writer.write_all(&config_bytes)?;
        hasher.update(&config_bytes);

        // Write nodes (HnswNode is Pod, can write directly)
        for node in &self.nodes {
            let node_bytes = bytemuck::bytes_of(node);
            writer.write_all(node_bytes)?;
            hasher.update(node_bytes);
        }

        // Write neighbor pool
        let neighbor_bytes = &self.neighbors.buffer;
        let neighbor_len = neighbor_bytes.len() as u64;
        writer.write_all(&neighbor_len.to_le_bytes())?;
        hasher.update(&neighbor_len.to_le_bytes());
        writer.write_all(neighbor_bytes)?;
        hasher.update(neighbor_bytes);

        // Write entry point
        let entry_point = self.entry_point.map(|n| n.0);
        let entry_bytes = postcard::to_allocvec(&entry_point)?;
        writer.write_all(&entry_bytes)?;
        hasher.update(&entry_bytes);

        // Write CRC32
        let checksum = hasher.finish();
        writer.write_all(&checksum.to_le_bytes())?;

        Ok(())
    }
}
```

##### Deserialization with Migration

```rust
impl HnswIndex {
    /// Load index from binary format
    ///
    /// Supports v2 and v3 formats. v2 is migrated to v3 on load.
    pub fn load<R: Read>(reader: &mut R) -> Result<Self, PersistenceError> {
        // Read magic and version first
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != SnapshotHeaderV3::MAGIC {
            return Err(PersistenceError::InvalidMagic);
        }

        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);

        match version {
            2 => Self::load_v2(reader, magic, version),
            3 => Self::load_v3(reader, magic, version),
            _ => Err(PersistenceError::UnsupportedVersion(version)),
        }
    }

    /// Load v2 format and migrate to v3
    fn load_v2<R: Read>(
        reader: &mut R,
        magic: [u8; 4],
        version: u32,
    ) -> Result<Self, PersistenceError> {
        // Read v2 header (without deleted_count)
        // ... read flags, dimensions, node_count ...

        // Read nodes - v2 had 'pad' which was always 0
        // The 'pad' byte is now 'deleted', and since v2 had no deletes,
        // all nodes loaded will have deleted = 0 (correct!)

        // Load rest of format...
        let mut index = Self::load_common(reader, /* params */)?;

        // Migration: deleted_count was not tracked in v2
        // All v2 nodes have deleted = 0 (since pad was always 0)
        index.deleted_count = 0;

        // Log migration
        log::info!("Migrated snapshot from v2 to v3 format");

        Ok(index)
    }

    /// Load v3 format
    fn load_v3<R: Read>(
        reader: &mut R,
        magic: [u8; 4],
        version: u32,
    ) -> Result<Self, PersistenceError> {
        // Read full v3 header
        // ... read flags, dimensions, node_count, deleted_count ...

        // Read nodes (with deleted field)
        // Load rest of format...
        let mut index = Self::load_common(reader, /* params */)?;

        // Set deleted_count from header
        index.deleted_count = header.deleted_count as usize;

        // Verify deleted_count matches actual deleted nodes
        let actual_deleted = index.nodes.iter().filter(|n| n.deleted != 0).count();
        if actual_deleted != index.deleted_count {
            log::warn!(
                "Snapshot deleted_count mismatch: header={}, actual={}",
                index.deleted_count,
                actual_deleted
            );
            index.deleted_count = actual_deleted;
        }

        Ok(index)
    }
}
```

#### Migration Path

##### v2 → v3 (Forward Compatible)

| Field | v2 Value | v3 Value | Migration |
|:------|:---------|:---------|:----------|
| version | 2 | 3 | Set to 3 |
| deleted_count | N/A | 0 | Add field |
| node.pad | 0 | - | Rename to deleted |
| node.deleted | - | 0 | Same byte, different name |

**Key Insight:** Since v2's `pad` byte was always 0, and v0.3.0's `deleted = 0` means "live", the migration is automatic - no data transformation needed!

##### v3 → v2 (NOT Supported)

v3 snapshots with deleted nodes cannot be read by v0.2.x. Document this in release notes:

```markdown
## Breaking Changes

### Persistence Format

v0.3.0 introduces snapshot format v3. Snapshots saved with v0.3.0 cannot be
read by v0.2.x. Downgrade requires re-indexing from original vectors.

v0.2.x snapshots can be read by v0.3.0 and will be automatically migrated.
```

#### Test Cases

**File:** `tests/persistence_v3.rs` (new file)

```rust
use edgevec::hnsw::{HnswConfig, HnswIndex, VectorId};
use edgevec::storage::VectorStorage;
use std::io::Cursor;

#[test]
fn test_save_load_v3_roundtrip() {
    let config = HnswConfig::new(4);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    // Insert some vectors
    for i in 0..10 {
        index.insert(&[i as f32; 4], &mut storage).unwrap();
    }

    // Delete some
    index.delete(VectorId(1)).unwrap();
    index.delete(VectorId(3)).unwrap();

    // Save
    let mut buffer = Vec::new();
    index.save(&mut buffer).unwrap();

    // Load
    let mut cursor = Cursor::new(buffer);
    let loaded = HnswIndex::load(&mut cursor).unwrap();

    // Verify
    assert_eq!(loaded.node_count(), 10);
    assert_eq!(loaded.deleted_count(), 2);
    assert!(loaded.is_deleted(VectorId(1)).unwrap());
    assert!(loaded.is_deleted(VectorId(3)).unwrap());
    assert!(!loaded.is_deleted(VectorId(2)).unwrap());
}

#[test]
fn test_save_v3_has_deleted_count_in_header() {
    let config = HnswConfig::new(4);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    for i in 0..5 {
        index.insert(&[i as f32; 4], &mut storage).unwrap();
    }
    index.delete(VectorId(1)).unwrap();

    let mut buffer = Vec::new();
    index.save(&mut buffer).unwrap();

    // Check header (magic + version + flags + dimensions + node_count + deleted_count)
    // Magic: 4 bytes, version: 4 bytes, flags: 4 bytes, dimensions: 4 bytes
    // node_count: 8 bytes, deleted_count: 8 bytes
    // Total header: 32 bytes

    // Check version is 3
    let version = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    assert_eq!(version, 3);

    // Check flags has tombstone bit set
    let flags = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
    assert_eq!(flags & 1, 1); // has_tombstones
}

#[test]
fn test_load_v2_migration() {
    // Create a v2-format snapshot manually or use a fixture
    // For now, we'll test the migration path by creating a v2-like index

    // This test verifies that loading an old format sets deleted_count = 0
    // In practice, you'd have a v2 fixture file

    let config = HnswConfig::new(4);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    // Fresh index should have deleted_count = 0
    assert_eq!(index.deleted_count(), 0);

    // After save/load with no deletes, deleted_count should still be 0
    let mut buffer = Vec::new();
    index.save(&mut buffer).unwrap();

    let mut cursor = Cursor::new(buffer);
    let loaded = HnswIndex::load(&mut cursor).unwrap();
    assert_eq!(loaded.deleted_count(), 0);
}

#[test]
fn test_crc32_checksum_valid() {
    let config = HnswConfig::new(4);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    for i in 0..5 {
        index.insert(&[i as f32; 4], &mut storage).unwrap();
    }
    index.delete(VectorId(1)).unwrap();

    let mut buffer = Vec::new();
    index.save(&mut buffer).unwrap();

    // Corrupt one byte
    buffer[100] ^= 0xFF;

    // Load should fail with checksum error
    let mut cursor = Cursor::new(buffer);
    let result = HnswIndex::load(&mut cursor);
    assert!(result.is_err());
}

#[test]
fn test_deleted_nodes_persist_correctly() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    // Insert 100 vectors
    for i in 0..100 {
        let vec: Vec<f32> = (0..128).map(|j| (i * 128 + j) as f32).collect();
        index.insert(&vec, &mut storage).unwrap();
    }

    // Delete specific IDs
    let deleted_ids: Vec<u64> = vec![5, 15, 25, 35, 45, 55, 65, 75, 85, 95];
    for id in &deleted_ids {
        index.delete(VectorId(*id)).unwrap();
    }

    // Save and load
    let mut buffer = Vec::new();
    index.save(&mut buffer).unwrap();

    let mut cursor = Cursor::new(buffer);
    let loaded = HnswIndex::load(&mut cursor).unwrap();

    // Verify all deleted IDs are still deleted
    for id in &deleted_ids {
        assert!(
            loaded.is_deleted(VectorId(*id)).unwrap(),
            "ID {} should be deleted after load",
            id
        );
    }

    // Verify non-deleted IDs are not deleted
    for id in 1..=100u64 {
        if !deleted_ids.contains(&id) {
            assert!(
                !loaded.is_deleted(VectorId(id)).unwrap(),
                "ID {} should not be deleted after load",
                id
            );
        }
    }
}
```

#### Verification Commands

```bash
# Run persistence tests
cargo test persistence_v3

# Run all tests
cargo test --all

# Clippy check
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

---

## Day 5 Summary

**Total Effort:** 6h scheduled

**Deliverables:**
1. SnapshotHeaderV3 with deleted_count
2. Save method for v3 format
3. Load method with v2 migration
4. CRC32 checksum validation
5. Comprehensive tests

---

## Week 16 Completion Checklist

Before declaring Week 16 complete:

- [ ] W16.1: `HnswNode.deleted` field exists
- [ ] W16.2: delete(), is_deleted() work correctly
- [ ] W16.3: Search excludes deleted vectors
- [ ] W16.4: compact() removes all tombstones
- [ ] W16.5: Persistence v3 with migration
- [ ] All 373+ tests pass
- [ ] Clippy clean
- [ ] Documentation updated

---

## HOSTILE_REVIEWER Pre-Flight

Before end of day:

- [ ] Snapshot version is 3
- [ ] deleted_count in header
- [ ] deleted field persisted per node
- [ ] v2 → v3 migration works
- [ ] CRC32 checksum catches corruption
- [ ] All new tests pass
- [ ] Clippy clean

---

**Status:** [REVISED]
**Next:** `/rust-implement W16.5` then `/review WEEKLY_TASK_PLAN.md`
