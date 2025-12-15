# EdgeVec Migration Guide

**Version:** 0.3.0
**Last Updated:** 2025-12-15

This guide covers migration between EdgeVec versions and file format versions.

---

## Quick Start: v0.2.x to v0.3.0

**Good news:** v0.3.0 is **fully backward compatible**. Your existing code works without changes.

**Persistence:** v0.2 snapshots are automatically migrated when loaded by v0.3.0.

**New features:** Soft Delete API and Compaction API (opt-in, no migration required).

---

## File Format Versions

| Version | Release | Features |
|:--------|:--------|:---------|
| v0.1 | Pre-alpha | Basic HNSW persistence |
| v0.2 | v0.2.0-alpha | Same as v0.1 |
| v0.3 | **v0.3.0** | Soft delete support, compaction |

---

## v0.2 to v0.3 Migration

### What Changed

**Header (64 bytes):**
- Offset 60-63: `reserved` (always 0) → `deleted_count` (u32)

**Node Structure (16 bytes):**
- Offset 15: `pad` (always 0) → `deleted` (u8)
  - 0 = live vector
  - 1 = deleted (tombstone)

### Migration Process

**Automatic (Default):**

Migration from v0.1/v0.2 to v0.3 is **fully automatic**. When you load an older format file:

1. EdgeVec detects the version mismatch
2. Since `reserved` and `pad` fields were always 0 in older formats:
   - `deleted_count = 0` (no deletions)
   - `deleted = 0` (all nodes live)
3. The index works immediately with soft delete support

**No user action required.**

### Example

```rust
use edgevec::persistence::{read_snapshot, write_snapshot};
use edgevec::persistence::storage::MemoryBackend;

// Load v0.1/v0.2 file — automatic migration
let (index, storage) = read_snapshot(&old_backend)?;

// Now you can use soft delete features
index.soft_delete(VectorId(1))?;
println!("Deleted count: {}", index.deleted_count());

// Save as v0.3 format
let mut new_backend = MemoryBackend::default();
write_snapshot(&index, &storage, &mut new_backend)?;
```

### Backward Compatibility

| Operation | v0.1/v0.2 File | v0.3 File |
|:----------|:--------------:|:---------:|
| Read by v0.3 code | YES (auto-migrated) | YES |
| Read by v0.1/v0.2 code | YES | **NO** |
| Write by v0.3 code | NO | YES |

> **WARNING: Version Downgrade Not Supported**
>
> v0.3 snapshots are **forward-incompatible**. Once a snapshot is written in v0.3 format:
>
> - **It cannot be loaded by v0.2.x or earlier**
> - Older versions will fail with "Unsupported version" error
> - **Always backup before upgrading**
>
> If you accidentally downgrade and need the data:
> 1. Reinstall v0.3.x
> 2. Export data to JSON/CSV
> 3. Re-import to older format (loses soft delete data)

**Important:** Once you save with v0.3 format, older EdgeVec versions cannot read the file. If you need backward compatibility:

1. Keep a backup of the original file before any soft-delete operations
2. Or re-export without soft delete data (requires custom export script)

---

## Detecting Format Version

```rust
use edgevec::persistence::{read_file_header, VERSION_MINOR, VERSION_MINOR_MIN};

let header = read_file_header(&data)?;

// Check version
println!("Format version: 0.{}", header.version_minor);

// Check if migration is needed
if header.needs_migration() {
    println!("File will be migrated from v0.{} to v0.{}",
             header.version_minor, VERSION_MINOR);
}

// Check soft delete support
if header.supports_soft_delete() {
    println!("Soft delete supported, deleted_count: {}", header.deleted_count);
}
```

---

## Troubleshooting

### "Unsupported version" error

This error means the file was created by a newer EdgeVec version than you're running. Update your EdgeVec dependency.

### "Checksum mismatch" error

The file is corrupted or was modified externally. Restore from backup.

### Deleted count mismatch warning

If you see a warning like:
```
Warning: snapshot deleted_count mismatch (header=X, actual=Y). Using actual.
```

This means the header's `deleted_count` doesn't match the actual count of deleted nodes. This can happen if:
- The snapshot was manually edited
- The file was partially corrupted

EdgeVec automatically corrects this by using the actual count.

---

## Version Constants

```rust
use edgevec::persistence::{VERSION_MAJOR, VERSION_MINOR, VERSION_MINOR_MIN};

// Current version
const VERSION_MAJOR: u8 = 0;  // 0.x releases
const VERSION_MINOR: u8 = 3;  // Current minor version

// Minimum supported for migration
const VERSION_MINOR_MIN: u8 = 1;  // Can read v0.1+
```

---

## Future Migrations

Future format changes will follow the same pattern:
1. Increment VERSION_MINOR
2. Add automatic migration from previous versions
3. Document changes in this file

Major version changes (1.0, 2.0) may require explicit migration tools.

---

## New API Summary (v0.3.0)

### Soft Delete (Rust)

```rust
// Delete a vector
let was_deleted = index.soft_delete(vector_id)?;

// Check deletion status
let is_deleted = index.is_deleted(vector_id)?;

// Statistics
let deleted = index.deleted_count();
let live = index.live_count();
let ratio = index.tombstone_ratio();
```

### Soft Delete (JavaScript)

```javascript
// Delete a vector
const wasDeleted = index.softDelete(vectorId);

// Check deletion status
const isDeleted = index.isDeleted(vectorId);

// Statistics
const deleted = index.deletedCount();
const live = index.liveCount();
const ratio = index.tombstoneRatio();
```

### Compaction (Rust)

```rust
// Check if compaction recommended
if index.needs_compaction() {
    let result = index.compact(&mut storage)?;
    println!("Removed {} tombstones", result.tombstones_removed);
}

// Configure threshold
index.set_compaction_threshold(0.4); // 40%
```

### Compaction (JavaScript)

```javascript
// Check if compaction recommended
if (index.needsCompaction()) {
    const result = index.compact();
    console.log(`Removed ${result.tombstones_removed} tombstones`);
}

// Configure threshold
index.setCompactionThreshold(0.4); // 40%
```

---

## See Also

- [API Reference](./API_REFERENCE.md) — Full API documentation
- [CHANGELOG](../CHANGELOG.md) — Version history
- [README](../README.md) — Quick start guide
