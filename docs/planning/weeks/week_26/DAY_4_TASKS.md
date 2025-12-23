# Week 26 Day 4: Persistence v0.4 Format

**Date:** 2025-12-24
**Focus:** MetadataSectionHeader struct and Postcard serialization
**Estimated Duration:** 6 hours
**Phase:** RFC-002 Core Metadata (Phase 1)

---

## Tasks

### W26.4.1: Add MetadataSectionHeader struct

**Objective:** Define metadata section header per RFC-002 Persistence Format.

**Acceptance Criteria:**
- [ ] `MetadataSectionHeader` struct (16 bytes) defined
- [ ] `Pod + Zeroable` derives for bytemuck zero-copy
- [ ] Compile-time size/alignment asserts:
  - `size_of::<MetadataSectionHeader>() == 16`
  - `align_of::<MetadataSectionHeader>() == 4`
- [ ] Constants defined:
  - `METADATA_MAGIC: [u8; 4] = *b"META"`
  - `METADATA_VERSION: u16 = 1`
  - `FORMAT_POSTCARD: u8 = 1`
  - `FORMAT_JSON: u8 = 2`
- [ ] `Flags::HAS_METADATA` constant added (`1 << 2`)

**Files:**
- `src/persistence/header.rs` (primary — add MetadataSectionHeader)
- `src/persistence/mod.rs` (add exports if needed)

**Estimated Duration:** 2 hours

**Agent:** RUST_ENGINEER

**Struct Definition (from RFC-002 Persistence Format §3.1):**
```rust
/// Metadata section header (16 bytes)
/// Placed after tombstone bitvec when HAS_METADATA flag is set.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct MetadataSectionHeader {
    /// Magic: "META" = [0x4D, 0x45, 0x54, 0x41]
    pub magic: [u8; 4],

    /// Section format version (currently 1)
    pub version: u16,

    /// Serialization format: 1=Postcard, 2=JSON
    pub format: u8,

    /// Reserved for future use (must be 0)
    pub reserved: u8,

    /// Size of serialized metadata in bytes
    pub size: u32,

    /// CRC32 of serialized metadata bytes
    pub crc: u32,
}

impl MetadataSectionHeader {
    pub const MAGIC: [u8; 4] = *b"META";
    pub const VERSION: u16 = 1;
    pub const FORMAT_POSTCARD: u8 = 1;
    pub const FORMAT_JSON: u8 = 2;

    /// Create new header for Postcard-serialized metadata
    pub fn new_postcard(size: u32, crc: u32) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            format: Self::FORMAT_POSTCARD,
            reserved: 0,
            size,
            crc,
        }
    }

    /// Validate magic bytes
    pub fn validate_magic(&self) -> Result<(), PersistenceError> {
        if self.magic != Self::MAGIC {
            return Err(PersistenceError::InvalidMagic);
        }
        Ok(())
    }

    /// Validate version is supported
    pub fn validate_version(&self) -> Result<(), PersistenceError> {
        if self.version > Self::VERSION {
            return Err(PersistenceError::UnsupportedVersion(self.version));
        }
        Ok(())
    }
}

// Compile-time verification
const _: () = assert!(std::mem::size_of::<MetadataSectionHeader>() == 16);
const _: () = assert!(std::mem::align_of::<MetadataSectionHeader>() == 4);
```

**Flag Addition:**
```rust
pub mod Flags {
    pub const COMPRESSED: u16 = 1 << 0;     // Data is compressed
    pub const QUANTIZED: u16 = 1 << 1;      // Vectors are quantized
    pub const HAS_METADATA: u16 = 1 << 2;   // NEW: MetadataStore present
}
```

**Dependencies:** None

---

### W26.4.2: Implement Postcard serialization

**Objective:** Serialize/deserialize MetadataStore with Postcard per RFC-002 §3.2.

**Acceptance Criteria:**
- [ ] `MetadataStore::to_postcard() -> Result<Vec<u8>, SerializationError>` implemented
- [ ] `MetadataStore::from_postcard(bytes: &[u8]) -> Result<Self, SerializationError>` implemented
- [ ] Round-trip tests pass (serialize → deserialize → equals original)
- [ ] CRC32 calculation implemented for data integrity
- [ ] Error types for serialization failures defined
- [ ] Optional: JSON fallback methods for debugging

**Files:**
- `src/metadata/serialize.rs` (new file — serialization implementation)
- `src/metadata/mod.rs` (add module export)
- `src/error.rs` (add serialization error variants)
- `tests/metadata_serialize.rs` (new file — serialization tests)
- `Cargo.toml` (add `postcard` dependency if not present)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**API Implementation (from RFC-002 Persistence Format §3.2):**
```rust
use postcard;
use serde::{Serialize, Deserialize};
use crc32fast;

impl MetadataStore {
    /// Serialize to postcard bytes (binary, compact)
    pub fn to_postcard(&self) -> Result<Vec<u8>, SerializationError> {
        postcard::to_allocvec(self).map_err(|e| {
            SerializationError::PostcardEncode(e.to_string())
        })
    }

    /// Deserialize from postcard bytes
    pub fn from_postcard(bytes: &[u8]) -> Result<Self, SerializationError> {
        postcard::from_bytes(bytes).map_err(|e| {
            SerializationError::PostcardDecode(e.to_string())
        })
    }

    /// Serialize to JSON bytes (for debugging/interop)
    pub fn to_json(&self) -> Result<Vec<u8>, SerializationError> {
        serde_json::to_vec(self).map_err(|e| {
            SerializationError::JsonEncode(e.to_string())
        })
    }

    /// Deserialize from JSON bytes
    pub fn from_json(bytes: &[u8]) -> Result<Self, SerializationError> {
        serde_json::from_slice(bytes).map_err(|e| {
            SerializationError::JsonDecode(e.to_string())
        })
    }

    /// Calculate CRC32 for serialized bytes
    pub fn calculate_crc(bytes: &[u8]) -> u32 {
        crc32fast::hash(bytes)
    }

    /// Verify CRC32 matches expected value
    pub fn verify_crc(bytes: &[u8], expected: u32) -> Result<(), SerializationError> {
        let actual = Self::calculate_crc(bytes);
        if actual != expected {
            return Err(SerializationError::CrcMismatch { expected, actual });
        }
        Ok(())
    }
}
```

**Error Types:**
```rust
#[derive(Debug, Error)]
pub enum SerializationError {
    #[error("Postcard encode failed: {0}")]
    PostcardEncode(String),

    #[error("Postcard decode failed: {0}")]
    PostcardDecode(String),

    #[error("JSON encode failed: {0}")]
    JsonEncode(String),

    #[error("JSON decode failed: {0}")]
    JsonDecode(String),

    #[error("CRC mismatch: expected {expected:#x}, got {actual:#x}")]
    CrcMismatch { expected: u32, actual: u32 },
}
```

**Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postcard_roundtrip_empty() {
        let store = MetadataStore::new();
        let bytes = store.to_postcard().unwrap();
        let restored = MetadataStore::from_postcard(&bytes).unwrap();
        assert!(restored.is_empty());
    }

    #[test]
    fn postcard_roundtrip_with_data() {
        let mut store = MetadataStore::new();
        store.insert(1, "key", MetadataValue::String("value".into()));
        store.insert(1, "count", MetadataValue::Integer(42));
        store.insert(2, "price", MetadataValue::Float(29.99));

        let bytes = store.to_postcard().unwrap();
        let restored = MetadataStore::from_postcard(&bytes).unwrap();

        assert_eq!(store.get(1, "key"), restored.get(1, "key"));
        assert_eq!(store.get(1, "count"), restored.get(1, "count"));
        assert_eq!(store.get(2, "price"), restored.get(2, "price"));
    }

    #[test]
    fn crc_validates_correctly() {
        let store = MetadataStore::new();
        let bytes = store.to_postcard().unwrap();
        let crc = MetadataStore::calculate_crc(&bytes);

        // Valid CRC should pass
        assert!(MetadataStore::verify_crc(&bytes, crc).is_ok());

        // Invalid CRC should fail
        assert!(MetadataStore::verify_crc(&bytes, crc + 1).is_err());
    }

    #[test]
    fn json_roundtrip() {
        let mut store = MetadataStore::new();
        store.insert(1, "key", MetadataValue::String("value".into()));

        let bytes = store.to_json().unwrap();
        let restored = MetadataStore::from_json(&bytes).unwrap();

        assert_eq!(store.get(1, "key"), restored.get(1, "key"));
    }
}
```

**Cargo.toml Addition:**
```toml
[dependencies]
postcard = { version = "1.0", features = ["alloc"] }
crc32fast = "1.3"
```

**Dependencies:** W26.4.1 (MetadataSectionHeader for constants)

---

## Day 4 Checklist

- [x] W26.4.1: MetadataSectionHeader struct defined (16 bytes)
- [x] W26.4.2: Postcard serialization implemented
- [x] W26.4.2: CRC32 validation implemented
- [x] All existing tests pass (`cargo test`)
- [x] New tests pass (`cargo test metadata_serialize`)
- [x] Clippy clean (`cargo clippy -- -D warnings`)
- [x] Formatted (`cargo fmt --check`)

## Day 4 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| MetadataSectionHeader is 16 bytes | Compile-time assert |
| Round-trip serialization works | Unit tests pass |
| CRC validation works | Unit tests pass |

## Day 4 Handoff

After completing Day 4:

**Artifacts Generated:**
- Modified `src/persistence/header.rs` (MetadataSectionHeader)
- New `src/metadata/serialize.rs`
- Modified `src/metadata/mod.rs` (module export)
- Modified `src/error.rs` (SerializationError)
- New `tests/metadata_serialize.rs`
- Modified `Cargo.toml` (postcard, crc32fast dependencies)

**Status:** APPROVED (2025-12-21)

**Next:** Day 5 — Persistence read/write + v0.3→v0.4 migration

---

*Agent: RUST_ENGINEER*
*Status: [APPROVED]*
