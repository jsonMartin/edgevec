# Week 22, Day 4: WASM Boundary & TypeScript API Design

**Date:** 2025-12-20
**Sprint:** Week 22 (v0.5.0 Phase)
**Day Theme:** JavaScript/TypeScript API Design
**Status:** PLANNED

---

## Task W22.4: WASM Boundary & TypeScript API Design

**Priority:** HIGH (P1)
**Estimated Effort:** 6 hours (3x rule: 2h optimistic × 3 = 6h)
**Status:** PLANNED
**Depends On:** W22.1, W22.2, W22.3
**Blocks:** W22.5

---

### Context

Day 4 designs the JavaScript/TypeScript API for filtering that is ergonomic, type-safe, and efficient across the WASM boundary.

**Strategic Importance:**
- TypeScript API is the primary interface for browser users
- API ergonomics directly impact developer adoption
- WASM boundary design affects bundle size (<50KB for filter module)

**Reference Documents:**
- `docs/architecture/FILTERING_SYNTAX.md` (Day 1)
- `docs/architecture/FILTER_EVALUATOR.md` (Day 2)
- `docs/architecture/FILTER_STRATEGY.md` (Day 3)
- `pkg/edgevec.d.ts` (existing TypeScript definitions)

---

### Objective

Create `docs/architecture/FILTERING_WASM_API.md` with:
1. TypeScript interface definitions (10+ types)
2. API design comparison (string vs builder pattern)
3. WASM serialization format specification
4. Error handling across WASM boundary
5. Bundle size impact estimation

---

### Technical Approach

#### 1. API Design Options

**Option A: String-based (SQL-like)**
```typescript
// Simple, familiar syntax
const results = await index.search(query, 10, {
  filter: 'category = "gpu" AND price < 1000'
});
```

**Pros:** Familiar to SQL users, compact, easy to construct dynamically
**Cons:** No compile-time checking, parse errors at runtime

**Option B: Builder Pattern**
```typescript
// Type-safe, IDE autocomplete
const filter = Filter.and(
  Filter.eq('category', 'gpu'),
  Filter.lt('price', 1000)
);
const results = await index.search(query, 10, { filter });
```

**Pros:** Type-safe, IDE support, no parse errors
**Cons:** Verbose, harder to read complex queries

**Option C: Hybrid (RECOMMENDED)**
```typescript
// String-based for simple queries
const results1 = await index.search(query, 10, {
  filter: 'category = "gpu"'
});

// Builder for complex/dynamic queries
const filter = new FilterBuilder()
  .where('category').eq('gpu')
  .and('price').lt(1000)
  .build();
const results2 = await index.search(query, 10, { filter });
```

**Rationale:** Best of both worlds - simplicity for common cases, type safety when needed

#### 2. TypeScript Type Definitions

```typescript
// ═══════════════════════════════════════════════════════════════════
// CORE TYPES (4)
// ═══════════════════════════════════════════════════════════════════

/** Metadata value types matching Rust MetadataValue */
type MetadataValue =
  | string
  | number
  | boolean
  | string[];

/** Search options with filtering support */
interface SearchOptions {
  /** Filter expression (string or Filter object) */
  filter?: string | Filter;

  /** Filter strategy selection */
  strategy?: FilterStrategy;

  /** Oversample factor for hybrid strategy */
  oversampleFactor?: number;

  /** Include metadata in results */
  includeMetadata?: boolean;
}

/** Filter strategy enum */
type FilterStrategy = 'pre' | 'post' | 'hybrid' | 'auto';

/** Search result with optional metadata */
interface SearchResult {
  /** Vector ID */
  id: number;

  /** Similarity score (0-1, higher is more similar) */
  score: number;

  /** Metadata if requested */
  metadata?: Record<string, MetadataValue>;
}

// ═══════════════════════════════════════════════════════════════════
// FILTER TYPES (4)
// ═══════════════════════════════════════════════════════════════════

/** Compiled filter expression */
interface Filter {
  /** Internal representation (opaque to user) */
  readonly _compiled: Uint8Array;

  /** Original string representation */
  toString(): string;
}

/** Filter builder for programmatic construction */
interface FilterBuilder {
  /** Start a field condition */
  where(field: string): FieldCondition;

  /** Combine with AND */
  and(field: string): FieldCondition;

  /** Combine with OR */
  or(field: string): FieldCondition;

  /** Build the final filter */
  build(): Filter;
}

/** Field condition builder */
interface FieldCondition {
  /** Equal to value */
  eq(value: MetadataValue): FilterBuilder;

  /** Not equal to value */
  ne(value: MetadataValue): FilterBuilder;

  /** Less than value */
  lt(value: number): FilterBuilder;

  /** Less than or equal */
  le(value: number): FilterBuilder;

  /** Greater than value */
  gt(value: number): FilterBuilder;

  /** Greater than or equal */
  ge(value: number): FilterBuilder;

  /** Contains substring */
  contains(value: string): FilterBuilder;

  /** Starts with prefix */
  startsWith(value: string): FilterBuilder;

  /** Ends with suffix */
  endsWith(value: string): FilterBuilder;

  /** Value in array */
  in(values: MetadataValue[]): FilterBuilder;

  /** Value not in array */
  notIn(values: MetadataValue[]): FilterBuilder;

  /** Field is null/undefined */
  isNull(): FilterBuilder;

  /** Field is not null */
  isNotNull(): FilterBuilder;
}

// ═══════════════════════════════════════════════════════════════════
// ERROR TYPES (5)
// ═══════════════════════════════════════════════════════════════════

/** Filter error codes */
enum FilterErrorCode {
  /** Invalid filter syntax */
  SYNTAX_ERROR = 'SYNTAX_ERROR',

  /** Type mismatch in comparison */
  TYPE_MISMATCH = 'TYPE_MISMATCH',

  /** Unknown metadata field */
  UNKNOWN_FIELD = 'UNKNOWN_FIELD',

  /** Integer overflow */
  OVERFLOW = 'OVERFLOW',

  /** Array operation on non-array */
  INVALID_ARRAY = 'INVALID_ARRAY',
}

/** Filter exception with rich error info */
interface FilterException extends Error {
  /** Error code for programmatic handling */
  code: FilterErrorCode;

  /** Position in filter string (if applicable) */
  position?: { line: number; column: number };

  /** Suggestion for fixing the error */
  suggestion?: string;
}

/** Error handling policy */
interface ErrorPolicy {
  /** How to handle unknown keys */
  unknownKeys: 'strict' | 'lenient';

  /** How to handle type mismatches */
  typeMismatch: 'error' | 'coerce';

  /** Whether empty results are warnings */
  emptyResults: 'ok' | 'warn';
}
```

#### 3. WASM Serialization Format

**Filter Expression Transfer:**

```
┌─────────────────────────────────────────────────────────┐
│  WASM BOUNDARY SERIALIZATION                            │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Option 1: JSON (Simple, larger)                        │
│  ──────────────────────────────────                     │
│  JS: filter.toJSON() → '{"op":"and","left":...}'        │
│  Rust: serde_json::from_str(json)                       │
│  Size: ~200-500 bytes per query                         │
│                                                         │
│  Option 2: Binary (Compact, faster)                     │
│  ──────────────────────────────────                     │
│  JS: filter._compiled (Uint8Array)                      │
│  Rust: bincode::deserialize(bytes)                      │
│  Size: ~50-150 bytes per query                          │
│                                                         │
│  RECOMMENDED: JSON for MVP, Binary for optimization     │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**JSON Schema for Filter:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "op": { "const": "eq" },
        "field": { "type": "string" },
        "value": {}
      }
    },
    {
      "type": "object",
      "properties": {
        "op": { "const": "and" },
        "left": { "$ref": "#" },
        "right": { "$ref": "#" }
      }
    }
  ]
}
```

#### 4. Error Handling Across WASM

```typescript
// Error mapping from Rust to JavaScript
try {
  const results = await index.search(query, 10, {
    filter: 'invalid syntax here!'
  });
} catch (e) {
  if (e instanceof FilterException) {
    console.log(e.code);       // 'SYNTAX_ERROR'
    console.log(e.position);   // { line: 1, column: 8 }
    console.log(e.suggestion); // "Did you mean 'category = \"value\"'?"
  }
}
```

**Error Propagation:**
```
┌─────────────────────────────────────────────────────────┐
│  Rust                  │  WASM Boundary  │  JavaScript  │
├────────────────────────┼─────────────────┼──────────────┤
│  FilterError::Syntax   │  → JsValue      │  FilterException │
│  FilterError::Type     │  → JsValue      │  FilterException │
│  panic!()              │  → RuntimeError │  Error (generic) │
└────────────────────────┴─────────────────┴──────────────┘
```

---

### Deliverables

1. **`docs/architecture/FILTERING_WASM_API.md`** containing:
   - Complete TypeScript interface definitions (10+ types)
   - API design comparison table
   - WASM serialization format specification
   - Error handling specification
   - Example code snippets (5+)
   - Bundle size estimation

---

### Acceptance Criteria

**CRITICAL (Must Pass):**
- [ ] TypeScript interface fully specified with 10+ type definitions
- [ ] API ergonomics validated with 5+ example code snippets
- [ ] WASM serialization format documented (JSON schema)
- [ ] Error types mapped to JavaScript exceptions (5+ error types)

**MAJOR (Should Pass):**
- [ ] Bundle size impact estimated <50KB for filter module
- [ ] Total WASM bundle still <500KB after filter additions
- [ ] Error handling policy documented
- [ ] All Rust code examples pass `cargo fmt --check`

---

### Example Code Snippets (Required)

```typescript
// Example 1: Simple string filter
const results = await index.search(query, 10, {
  filter: 'category = "electronics"'
});

// Example 2: Numeric range
const results = await index.search(query, 10, {
  filter: 'price >= 100 AND price < 500'
});

// Example 3: String contains
const results = await index.search(query, 10, {
  filter: 'title CONTAINS "NVIDIA"'
});

// Example 4: Builder pattern
const filter = new FilterBuilder()
  .where('category').eq('gpu')
  .and('price').lt(1000)
  .build();
const results = await index.search(query, 10, { filter });

// Example 5: Error handling
try {
  const results = await index.search(query, 10, {
    filter: 'invalid!'
  });
} catch (e) {
  if (e.code === FilterErrorCode.SYNTAX_ERROR) {
    console.log(`Syntax error at column ${e.position?.column}`);
  }
}
```

---

### Bundle Size Estimation

| Component | Estimated Size | Notes |
|:----------|:---------------|:------|
| Filter parser (pest) | ~15KB | Grammar + parser code |
| Filter evaluator | ~10KB | AST + evaluation |
| WASM bindings | ~5KB | wasm-bindgen glue |
| Error types | ~2KB | Error enums |
| **Total Filter Module** | **~32KB** | Well under 50KB budget |
| Current EdgeVec | 248KB | Pre-filtering |
| **New Total** | **~280KB** | Well under 500KB limit |

---

### Implementation Checklist

- [ ] Create `docs/architecture/FILTERING_WASM_API.md`
- [ ] Define all TypeScript interfaces (10+)
- [ ] Write 5+ example code snippets
- [ ] Document WASM serialization format
- [ ] Specify error type mapping
- [ ] Create bundle size estimation
- [ ] Design FilterBuilder API
- [ ] Document error handling policy

---

### Dependencies

**Blocks:**
- W22.5 (Test strategy needs API design)

**Blocked By:**
- W22.1 (Syntax for string filter)
- W22.2 (AST for builder pattern)
- W22.3 (Strategy options for SearchOptions)

---

### Verification Method

**Day 4 is COMPLETE when:**

1. `docs/architecture/FILTERING_WASM_API.md` exists
2. 10+ TypeScript type definitions documented
3. 5+ example code snippets included
4. Bundle size estimation <50KB

---

### Estimated Timeline

| Phase | Time | Cumulative |
|:------|:-----|:-----------|
| Review Days 1-3 | 0.5h | 0.5h |
| TypeScript types | 1.5h | 2h |
| API comparison | 0.5h | 2.5h |
| Example snippets | 1h | 3.5h |
| WASM serialization | 1h | 4.5h |
| Error handling | 0.5h | 5h |
| Bundle estimation | 0.5h | 5.5h |
| Buffer | 0.5h | 6h |

---

### Hostile Review Checkpoint

**End of Day 4:** Submit for `/review` with:
- `docs/architecture/FILTERING_WASM_API.md`

**Expected Review Focus:**
- API ergonomics and consistency
- TypeScript type completeness
- WASM boundary efficiency
- Error handling comprehensiveness
- Bundle size within budget

---

**Task Owner:** WASM_SPECIALIST / META_ARCHITECT
**Review Required:** HOSTILE_REVIEWER
**Next Task:** W22.5 (Test Strategy & FILTERING_API.md Finalization)

---

*"A good API is invisible. A great API is inevitable."*
