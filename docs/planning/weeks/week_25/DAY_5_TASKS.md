# Week 25 Day 5: RFC-002 Metadata Storage — Design

**Date:** 2025-12-24 (flexible — holiday consideration)
**Focus:** Design integrated metadata storage architecture
**Estimated Duration:** 5-6 hours

---

## Tasks

### W25.5.1: Metadata Storage Requirements Analysis

**Objective:** Define requirements for integrated metadata storage.

**Acceptance Criteria:**
- [ ] Document current metadata pattern (external Map)
- [ ] List user pain points with current approach
- [ ] Define target API for v0.6.0
- [ ] Identify storage constraints (memory, persistence)

**Deliverables:**
- `docs/rfcs/RFC-002_REQUIREMENTS.md`

**Dependencies:** None

**Estimated Duration:** 1.5 hours

**Agent:** META_ARCHITECT

**Current Pattern (v0.5.0):**
```javascript
// User must manage metadata externally
const metadata = {};
const id = db.insert(vector);
metadata[id] = { category: "books", price: 29.99 };

// Filter requires manual mapping
const results = db.search(query, 10);
const filtered = results.filter(r =>
  Filter.parse('price < 50').evaluate(metadata[r.id])
);
```

**Target Pattern (v0.6.0):**
```javascript
// Metadata stored with vector
const id = db.insertWithMetadata(vector, { category: "books", price: 29.99 });

// Integrated filtered search
const results = db.searchFiltered(query, 'price < 50', 10);
```

---

### W25.5.2: Storage Architecture Options

**Objective:** Evaluate storage architectures for metadata.

**Acceptance Criteria:**
- [ ] Option A: Inline storage (metadata in HnswNode)
- [ ] Option B: Sidecar storage (separate metadata array)
- [ ] Option C: Hybrid (hot/cold separation)
- [ ] Compare: Memory overhead, access patterns, persistence impact
- [ ] Recommend preferred approach

**Deliverables:**
- `docs/rfcs/RFC-002_ARCHITECTURE_OPTIONS.md`

**Dependencies:** W25.5.1

**Estimated Duration:** 2 hours

**Agent:** META_ARCHITECT

**Comparison Matrix:**
| Criterion | Inline | Sidecar | Hybrid |
|:----------|:-------|:--------|:-------|
| Memory Overhead | | | |
| Cache Locality | | | |
| Persistence Complexity | | | |
| Filter Performance | | | |
| Migration Path | | | |

---

### W25.5.3: Persistence Format Design

**Objective:** Design persistence format for metadata.

**Acceptance Criteria:**
- [ ] Define metadata serialization format (JSON, MessagePack, custom)
- [ ] Design header changes for v0.4 format
- [ ] Plan migration from v0.3 snapshots
- [ ] Document checksum strategy

**Deliverables:**
- Persistence format specification in RFC-002

**Dependencies:** W25.5.2

**Estimated Duration:** 1.5 hours

**Agent:** META_ARCHITECT

**Format Considerations:**
```
v0.4 Header Extension:
+0x40: metadata_offset (u64)
+0x48: metadata_size (u64)
+0x50: metadata_format (u8) — 0=None, 1=JSON, 2=MessagePack
```

---

### W25.5.4: Draft RFC-002

**Objective:** Write complete RFC-002 document.

**Acceptance Criteria:**
- [ ] Problem statement
- [ ] Proposed solution
- [ ] API changes
- [ ] Memory impact analysis
- [ ] Persistence format changes
- [ ] Migration strategy
- [ ] Alternatives considered
- [ ] Open questions

**Deliverables:**
- `docs/rfcs/RFC-002_METADATA_STORAGE.md`

**Dependencies:** W25.5.1, W25.5.2, W25.5.3

**Estimated Duration:** 1.5 hours

**Agent:** META_ARCHITECT

**RFC Template:**
```markdown
# RFC-002: Integrated Metadata Storage

**Status:** [PROPOSED]
**Author:** META_ARCHITECT
**Date:** 2025-12-24

## Summary
[One paragraph]

## Motivation
[Why is this needed?]

## Detailed Design
[Technical details]

## API Changes
[New/modified APIs]

## Memory Impact
[Calculations]

## Persistence Format
[Format specification]

## Migration
[How to migrate existing indexes]

## Alternatives Considered
[What else was evaluated]

## Open Questions
[Unresolved issues]
```

---

## Day 5 Checklist

- [ ] W25.5.1: Requirements documented
- [ ] W25.5.2: Architecture options evaluated
- [ ] W25.5.3: Persistence format designed
- [ ] W25.5.4: RFC-002 draft complete

## Day 5 Exit Criteria

- RFC-002 draft ready for HOSTILE_REVIEWER
- All architectural decisions documented
- Memory impact calculated

---

*Agent: META_ARCHITECT*
*Status: [PROPOSED]*
