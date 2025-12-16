# Metadata Schema v1.0 (FROZEN)

**Version:** 1.0.0
**Status:** FROZEN
**Frozen Date:** 2025-12-16
**Breaking Changes:** Require major version bump (v1.0.0)

---

## Schema Overview

EdgeVec metadata is a key-value store attached to vectors. Each vector
can have up to 64 metadata keys. This schema is FROZEN - any breaking
changes require a major version bump.

## Value Types

| Type | Rust Type | JSON Type | TypeScript Type | Constraints |
|:-----|:----------|:----------|:----------------|:------------|
| String | `String` | `string` | `string` | Max 65,536 bytes |
| Integer | `i64` | `number` | `number` | -2^63 to 2^63-1 (JS safe: +-2^53) |
| Float | `f64` | `number` | `number` | IEEE 754, finite values only |
| Boolean | `bool` | `boolean` | `boolean` | true/false |
| StringArray | `Vec<String>` | `string[]` | `string[]` | Max 1,024 elements |

## Key Constraints

| Constraint | Value | Rationale |
|:-----------|:------|:----------|
| Max keys per vector | 64 | Memory budget |
| Max key length | 256 bytes | Reasonable limit |
| Key format | `[a-zA-Z_][a-zA-Z0-9_]*` | JSON/WASM compatibility |

## JSON Serialization

Values are serialized with type tags for unambiguous deserialization:

```json
{"type": "string", "value": "hello"}
{"type": "integer", "value": 42}
{"type": "float", "value": 3.14159}
{"type": "boolean", "value": true}
{"type": "string_array", "value": ["a", "b", "c"]}
```

## Persistence Format

Metadata is stored in snapshot format with the vectors:

```
[4 bytes] Magic: "EVEC"
[4 bytes] Version: 3
[N bytes] Header (bincode)
[N bytes] Vectors (bincode)
[N bytes] Graph (bincode)
[N bytes] Metadata (bincode)
[4 bytes] CRC32 checksum
```

## API Surface (FROZEN)

### Rust API

```rust
/// Core metadata value enum
pub enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    StringArray(Vec<String>),
}

impl MetadataStore {
    /// Create a new empty metadata store
    pub fn new() -> Self;

    /// Insert a metadata key-value pair for a vector
    pub fn insert(&mut self, vector_id: u32, key: &str, value: MetadataValue) -> Result<(), MetadataError>;

    /// Get a metadata value for a vector
    pub fn get(&self, vector_id: u32, key: &str) -> Option<&MetadataValue>;

    /// Get all metadata for a vector
    pub fn get_all(&self, vector_id: u32) -> Option<&HashMap<String, MetadataValue>>;

    /// Update an existing metadata value
    pub fn update(&mut self, vector_id: u32, key: &str, value: MetadataValue) -> Result<(), MetadataError>;

    /// Delete a metadata key for a vector
    pub fn delete(&mut self, vector_id: u32, key: &str) -> Result<bool, MetadataError>;

    /// Delete all metadata for a vector
    pub fn delete_all(&mut self, vector_id: u32) -> bool;

    /// Check if a key exists for a vector
    pub fn has_key(&self, vector_id: u32, key: &str) -> bool;

    /// Get count of keys for a vector
    pub fn key_count(&self, vector_id: u32) -> usize;
}
```

### WASM/TypeScript API

```typescript
/**
 * JavaScript-friendly metadata value wrapper
 */
export class JsMetadataValue {
  // Factory methods
  static fromString(value: string): JsMetadataValue;
  static fromInteger(value: number): JsMetadataValue;
  static fromFloat(value: number): JsMetadataValue;
  static fromBoolean(value: boolean): JsMetadataValue;
  static fromStringArray(value: string[]): JsMetadataValue;

  // Type inspection
  getType(): 'string' | 'integer' | 'float' | 'boolean' | 'string_array';
  isString(): boolean;
  isInteger(): boolean;
  isFloat(): boolean;
  isBoolean(): boolean;
  isStringArray(): boolean;

  // Value extraction
  asString(): string | undefined;
  asInteger(): number | undefined;
  asFloat(): number | undefined;
  asBoolean(): boolean | undefined;
  asStringArray(): string[] | undefined;

  // JavaScript conversion
  toJS(): string | number | boolean | string[];
}

/**
 * EdgeVec index with metadata support
 */
export class EdgeVec {
  // Metadata operations
  setMetadata(vectorId: number, key: string, value: JsMetadataValue): void;
  getMetadata(vectorId: number, key: string): JsMetadataValue | undefined;
  getAllMetadata(vectorId: number): Record<string, any> | undefined;
  deleteMetadata(vectorId: number, key: string): boolean;
  deleteAllMetadata(vectorId: number): boolean;
  hasMetadata(vectorId: number, key: string): boolean;
  metadataKeyCount(vectorId: number): number;
}
```

## Validation Rules

### Key Validation
- Keys must be 1-256 bytes
- Keys must match pattern: `[a-zA-Z_][a-zA-Z0-9_]*`
- Keys are case-sensitive

### Value Validation
- **String:** UTF-8 encoded, max 65,536 bytes
- **Integer:** i64 range, JS safe integer range for WASM (+-2^53-1)
- **Float:** Must be finite (no NaN, Infinity, -Infinity)
- **Boolean:** true or false
- **StringArray:** Max 1,024 elements, each element follows String rules

### Error Conditions

| Error | Cause |
|:------|:------|
| `KeyTooLong` | Key exceeds 256 bytes |
| `InvalidKeyFormat` | Key contains invalid characters |
| `ValueTooLarge` | String exceeds 65,536 bytes |
| `TooManyKeys` | Vector has 64 keys already |
| `TooManyArrayElements` | StringArray has >1,024 elements |
| `VectorNotFound` | Vector ID doesn't exist |
| `KeyNotFound` | Key doesn't exist (for update) |

## Compatibility Guarantees

1. **Forward Compatibility:** v0.5.0+ can read v0.4.0 snapshots (empty metadata)
2. **Backward Compatibility:** v0.4.0 cannot read v0.5.0 snapshots (version check fails gracefully)
3. **API Stability:** All methods listed above are stable for v0.x lifetime

## Future Extensions (v2.0)

The following are candidates for schema v2.0 (requires major version bump):

- Additional value types (Date, Binary, Object)
- Nested metadata objects
- Type coercion/casting
- Computed/derived metadata
- Metadata indexes for filtering

---

## Freeze Declaration

```
+---------------------------------------------------------------------+
|                                                                     |
|   METADATA SCHEMA v1.0 IS NOW FROZEN                                |
|                                                                     |
|   Date: 2025-12-16                                                  |
|   Authority: HOSTILE_REVIEWER                                       |
|                                                                     |
|   Any breaking changes to:                                          |
|   - Value types (adding/removing/modifying)                         |
|   - Key constraints                                                 |
|   - Serialization format                                            |
|   - API method signatures                                           |
|                                                                     |
|   REQUIRE A MAJOR VERSION BUMP (v0.x -> v1.0)                       |
|                                                                     |
|   Non-breaking additions (new methods, relaxed constraints)         |
|   are permitted in minor versions.                                  |
|                                                                     |
+---------------------------------------------------------------------+
```

---

## Changelog

| Version | Date | Changes |
|:--------|:-----|:--------|
| 1.0.0 | 2025-12-16 | Initial frozen schema |

---

**Schema Owner:** EdgeVec Core Team
**Review Authority:** HOSTILE_REVIEWER
