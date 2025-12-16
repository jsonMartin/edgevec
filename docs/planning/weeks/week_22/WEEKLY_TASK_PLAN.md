# Week 22: Filtering Architecture Design Sprint

**Week Number:** 22
**Sprint Theme:** FILTERING_API Architecture & Design
**Status:** PLANNED
**Priority:** P0 CRITICAL
**Estimated Duration:** 2025-12-17 → 2025-12-22 (with buffer to 2025-12-24)
**Git Branch:** `feature/w22-filtering-architecture`

---

## Pre-Sprint Audit

### Prerequisites Verified

| Prerequisite | Status | Evidence |
|:-------------|:-------|:---------|
| GATE_W21_COMPLETE.md exists | PASS | [.claude/GATE_W21_COMPLETE.md](../../../.claude/GATE_W21_COMPLETE.md) |
| Metadata Schema FROZEN | PASS | [docs/schemas/METADATA_SCHEMA_V1.md](../../schemas/METADATA_SCHEMA_V1.md) |
| V0.5.0 Roadmap approved | PASS | [docs/planning/V0.5.0_STRATEGIC_ROADMAP.md](../V0.5.0_STRATEGIC_ROADMAP.md) |
| 305/305 tests passing | PASS | `cargo test` verified |
| WASM bundle <500KB | PASS | 248KB actual |

### Foundation Analysis

**Available Metadata Types (FROZEN):**
```
String      - Text up to 64KB
Integer     - i64 (JS safe: +-2^53)
Float       - f64 finite values
Boolean     - true/false
StringArray - Up to 1024 strings
```

**Available Operations to Filter On:**
- Equality: `=`, `!=`
- Comparison: `<`, `<=`, `>`, `>=`
- String: `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`
- Array: `IN`, `NOT IN`, `ANY`, `ALL`
- Logical: `AND`, `OR`, `NOT`
- Existence: `IS NULL`, `IS NOT NULL`

---

## Competitive Intelligence (Industry Research)

### Vector Database Filtering Approaches Analyzed

Research conducted on 2025-12-17 covering major vector database filtering APIs.

#### 1. Pinecone (MongoDB-style JSON)

**Syntax Style:** JSON with MongoDB operators (`$eq`, `$gt`, `$in`, `$and`, `$or`)

```json
{
  "filter": {
    "$and": [
      {"genre": {"$eq": "comedy"}},
      {"year": {"$gte": 2020}}
    ]
  }
}
```

**Operators:** `$eq`, `$ne`, `$gt`, `$gte`, `$lt`, `$lte`, `$in`, `$nin`, `$exists`, `$and`, `$or`

**Key Insight:** Pinecone reports filtering often IMPROVES throughput (1.2x-1.5x) with integrated filtering due to reduced candidate set.

**Limitation:** No null values supported - must remove key instead.

#### 2. Milvus (SQL-like String)

**Syntax Style:** SQL-like string expressions

```python
filter='product["price"] < 1850 AND category == "electronics"'
```

**Operators:** `==`, `!=`, `>`, `<`, `>=`, `<=`, `like`, `LIKE`, `in`, `not in`, `and`, `or`, `not`

**EBNF Grammar Documented:** Yes - Milvus has formal EBNF for boolean expressions

**Key Insight:** Supports both `&&`/`and` and `||`/`or` syntax variants. Case-insensitive keywords.

**Advanced Features:** `json_contains()`, `array_contains_any()`, `array_length()` functions.

#### 3. Qdrant (Structured JSON with Clauses)

**Syntax Style:** Nested JSON with `must`, `should`, `must_not` clauses

```json
{
  "filter": {
    "must": [
      {"key": "category", "match": {"value": "laptop"}},
      {"key": "price", "range": {"lte": 1000}}
    ]
  }
}
```

**Operators:** `match`, `range`, `geo_radius`, `values_count`, `is_empty`, `is_null`, `has_id`

**Key Insight:** Qdrant pioneered "Filterable HNSW" - graph augmentation that maintains connectivity under filtering. Adds intra-category links so filtered nodes don't break traversal.

**Performance:** Query planner automatically switches from HNSW to payload index for low cardinality filters.

#### 4. Weaviate (GraphQL)

**Syntax Style:** GraphQL with `where` clause

```graphql
{
  Get {
    Article(where: {
      operator: And,
      operands: [{
        path: ["wordCount"],
        operator: GreaterThan,
        valueInt: 1000
      }]
    }) {
      title
    }
  }
}
```

**Operators:** `Equal`, `NotEqual`, `GreaterThan`, `LessThan`, `Like`, `WithinGeoRange`, `And`, `Or`

**Key Insight:** Implements ACORN algorithm for filtered HNSW - multi-hop approach that evaluates neighborhood candidates.

### Competitive Analysis Summary

| Feature | Pinecone | Milvus | Qdrant | Weaviate | **EdgeVec Target** |
|:--------|:---------|:-------|:-------|:---------|:-------------------|
| Syntax Style | JSON/MongoDB | SQL-like | Structured JSON | GraphQL | **SQL-like + Builder** |
| Formal Grammar | No | Yes (EBNF) | No | No | **Yes (EBNF)** |
| Null Support | No | Yes | Yes | Yes | **Yes** |
| Array Ops | `$in`, `$nin` | `in`, `array_contains` | `values_count` | `ContainsAny` | **IN, NOT IN, ANY, ALL** |
| String Ops | No | `like` | `match` | `Like` | **CONTAINS, STARTS_WITH, ENDS_WITH** |
| Filter Strategy | Integrated | Post-filter | Filterable HNSW | ACORN | **Hybrid (Qdrant-inspired)** |

### Key Decisions from Research

1. **Syntax:** SQL-like (Milvus pattern) - most familiar to developers, has formal EBNF
2. **Null Handling:** Support `IS NULL` / `IS NOT NULL` (unlike Pinecone)
3. **Filter Strategy:** Hybrid approach inspired by Qdrant's filterable HNSW
4. **API:** Dual API - string-based + programmatic builder (best of Pinecone + Milvus)

---

## Sprint Objective

Design a production-grade filtering API that:
1. Integrates with HNSW search seamlessly
2. Supports all 5 metadata types
3. Achieves <10ms P99 latency with filtering at 100k × 384-dim vectors
4. Works identically in Rust and WASM
5. Has formally specified query syntax (EBNF)

**CRITICAL CONSTRAINT:** This is a DESIGN SPRINT. NO implementation code.

---

## Day-by-Day Engineering Tasks

### Day 1 (W22.1): Query Syntax Design & EBNF Grammar

**Task ID:** W22.1
**Priority:** P0 CRITICAL
**Effort:** 8 hours
**Type:** Architecture/Design
**Depends On:** GATE_W21_COMPLETE.md
**Blocks:** All other W22 tasks

#### Objective

Define the formal query syntax using EBNF grammar. This is the contract that the parser, evaluator, and all documentation will be built against.

#### Deliverables

1. **`docs/architecture/FILTERING_SYNTAX.md`**
   - Complete EBNF grammar specification
   - Operator precedence table
   - Escape sequences for strings
   - Reserved keywords list
   - Error message catalog

2. **Query Examples (at least 20)**
   - Simple equality: `category = "electronics"`
   - Numeric range: `price >= 100 AND price < 500`
   - String operations: `title CONTAINS "NVIDIA"`
   - Array membership: `tags IN ["gpu", "cuda"]`
   - Complex nested: `(category = "gpu" OR category = "tpu") AND price < 1000`
   - Boolean: `is_active = true`
   - Null handling: `description IS NOT NULL`

3. **Type Coercion Rules**
   - What happens when comparing Integer to Float?
   - String to StringArray in `IN` operator?
   - Document all implicit/explicit conversions

#### Technical Decisions Required

| Decision | Options | Impact |
|:---------|:--------|:-------|
| Syntax style | SQL-like vs JavaScript-like | User experience |
| Case sensitivity | Case-insensitive keywords | Parser complexity |
| String quotes | Single vs double vs both | Escape handling |
| Null semantics | SQL NULL vs JSON null | Query behavior |

#### Acceptance Criteria (MEASURABLE)

- [ ] EBNF grammar contains minimum 30 rules covering all operators
- [ ] All 5 metadata types have at least 3 operators defined each
- [ ] Operator precedence table with 6+ levels documented
- [ ] 20+ example queries with expected AST output (JSON format)
- [ ] Edge cases documented: empty strings, MAX_INT, MIN_INT, NaN rejection, 64KB string
- [ ] Grammar can be validated with a parser generator (pest)
- [ ] **BINARY CHECK**: Grammar file parses without errors via `pest_meta::validate`
- [ ] **BINARY CHECK**: Passes `cargo fmt --check` on any example code
- [ ] All vector references specify dimensionality (100k × 384-dim)

#### Verification Method

```bash
# Grammar MUST parse without errors
pest_meta::validate(grammar_file) == Ok(())

# Grammar complexity ceiling
- Max rules: 50
- Max nesting depth: 5 levels
- Max alternations per rule: 7
```

#### Competitive Analysis Verification Methodology

**Sources (MUST document for each competitor):**
| Competitor | Primary Source | Secondary Source | Live Test |
|:-----------|:---------------|:-----------------|:----------|
| Pinecone | Official Docs | GitHub Issues | API Playground |
| Milvus | Official Docs | Source Code | Local Docker |
| Qdrant | Official Docs | Source Code | Local Docker |
| Weaviate | Official Docs | GraphQL Schema | Local Docker |

**Verification Checklist:**
- [ ] Feature documented from official docs (link required)
- [ ] Cross-referenced with second source (GitHub/code)
- [ ] Live API test executed where possible
- [ ] Performance claims verified against independent benchmarks (not marketing)

---

### Day 2 (W22.2): Filter Evaluator Architecture

**Task ID:** W22.2
**Priority:** P0 CRITICAL
**Effort:** 8 hours
**Type:** Architecture/Design
**Depends On:** W22.1
**Blocks:** W22.3, W22.4

#### Objective

Design the filter evaluation engine that will execute parsed queries against metadata. This is the core algorithmic challenge.

#### Deliverables

1. **`docs/architecture/FILTER_EVALUATOR.md`**
   - AST node types (enum design)
   - Evaluation algorithm (recursive tree walk vs stack machine)
   - Short-circuit evaluation strategy
   - Memory allocation strategy (zero-copy where possible)
   - Error handling during evaluation

2. **Data Structure Designs**
   ```rust
   // Proposed AST structure (for documentation only)
   pub enum FilterExpr {
       // Literals
       Literal(MetadataValue),

       // Field access
       Field(String),

       // Comparisons
       Eq(Box<FilterExpr>, Box<FilterExpr>),
       Ne(Box<FilterExpr>, Box<FilterExpr>),
       Lt(Box<FilterExpr>, Box<FilterExpr>),
       Le(Box<FilterExpr>, Box<FilterExpr>),
       Gt(Box<FilterExpr>, Box<FilterExpr>),
       Ge(Box<FilterExpr>, Box<FilterExpr>),

       // String operations
       Contains(Box<FilterExpr>, Box<FilterExpr>),
       StartsWith(Box<FilterExpr>, Box<FilterExpr>),
       EndsWith(Box<FilterExpr>, Box<FilterExpr>),

       // Array operations
       In(Box<FilterExpr>, Box<FilterExpr>),
       NotIn(Box<FilterExpr>, Box<FilterExpr>),

       // Logical operations
       And(Box<FilterExpr>, Box<FilterExpr>),
       Or(Box<FilterExpr>, Box<FilterExpr>),
       Not(Box<FilterExpr>),

       // Null checks
       IsNull(Box<FilterExpr>),
       IsNotNull(Box<FilterExpr>),
   }
   ```

3. **Performance Analysis**
   - Complexity analysis for each operator
   - Expected evaluation time for common query patterns
   - Memory overhead per filter expression

#### Technical Decisions Required

| Decision | Options | Recommendation |
|:---------|:--------|:---------------|
| AST representation | Enum vs trait object | Enum (monomorphization) |
| Evaluation strategy | Recursive vs iterative | Recursive (clarity) |
| String comparison | Byte vs Unicode | Unicode-aware |
| Short-circuit | Lazy vs eager | Lazy AND/OR |

#### Acceptance Criteria (MEASURABLE)

- [ ] Complete AST enum design with 15+ variant types documented
- [ ] Evaluation algorithm pseudo-code with O(n) complexity annotations
- [ ] Short-circuit behavior formally specified (AND/OR early exit conditions)
- [ ] Memory overhead calculated per node type (bytes per variant)
- [ ] **BINARY CHECK**: Total AST overhead for typical query <1KB
- [ ] Error types cataloged with 10+ distinct error messages
- [ ] Complexity analysis for all operators (O(1) for comparisons, O(n) for array ops)
- [ ] **Preliminary Memory Budget**:
  - Estimate AST node size for typical query: <200 bytes
  - Define memory ceiling that triggers design changes: 10KB per query max

#### Verification Checklist

```
Evaluator Design Verification:
- [ ] Each operator has documented time complexity
- [ ] Each operator has documented space complexity
- [ ] Short-circuit test cases provided (5+ examples)
- [ ] Error propagation path documented
- [ ] Unicode string comparison strategy defined
```

---

### Day 3 (W22.3): Pre-Filter vs Post-Filter Strategy

**Task ID:** W22.3
**Priority:** P0 CRITICAL
**Effort:** 6 hours
**Type:** Architecture/Design
**Depends On:** W22.2
**Blocks:** W22.4

#### Objective

Determine the optimal strategy for integrating filtering with HNSW search. This is the most critical performance decision of the sprint.

#### Industry Benchmark Data (2025)

From competitive research on Pinecone, Qdrant, Milvus, Weaviate:

| Engine | Filter Approach | QPS (Unfiltered) | QPS (10% Filter) | Latency P99 |
|:-------|:----------------|:-----------------|:-----------------|:------------|
| Pinecone | Integrated | ~800 | ~600 | <10ms |
| Zilliz/Milvus | Integrated | ~750 | ~700 | <15ms |
| Qdrant | Filterable HNSW | ~700 | ~750 | <12ms |
| LanceDB | Post-filter | ~500 | ~300 | 200-300ms |
| PGVector | Post-filter | ~400 | ~250 | 150-200ms |

**Key Finding:** Engines with in-algorithm filtering (Qdrant, Pinecone) not only preserve recall—they get FASTER under filters due to reduced workload.

#### The Core Trade-off

| Strategy | Description | Pros | Cons |
|:---------|:------------|:-----|:-----|
| **Pre-Filter** | Filter vectors before HNSW traversal | Accurate k results | May break HNSW connectivity |
| **Post-Filter** | Run HNSW, then filter results | HNSW integrity preserved | May return <k results |
| **Hybrid** | Pre-filter with oversampling | Best of both | Complex implementation |
| **Filterable HNSW** | In-graph filtering during traversal | Best recall + speed | Requires graph augmentation |

#### The Qdrant Insight (Critical)

Qdrant's "Filterable HNSW" approach:
1. Adds extra intra-category links to the HNSW graph
2. Maintains connectivity even when nodes are filtered out
3. Query planner switches to payload index for very selective filters (<1% match)
4. Result: Filtering can IMPROVE performance, not degrade it

#### Deliverables

1. **`docs/architecture/FILTER_STRATEGY.md`**
   - Decision matrix for all strategies
   - Performance modeling for each approach
   - Recommended strategy with justification
   - Edge case handling (all vectors filtered out)
   - Fallback behavior specification

2. **Performance Modeling**
   ```
   Scenario: 100k × 384-dim vectors, 50% pass filter, k=10

   Pre-Filter:
   - Filter time: O(n) = 100k evaluations
   - HNSW search: O(log(50k)) on filtered set
   - Total: filter_time + search_time
   - Risk: HNSW graph may be disconnected

   Post-Filter:
   - HNSW search: O(log(100k))
   - Return: ef_search candidates
   - Filter: O(ef_search) evaluations
   - Risk: May return fewer than k results

   Hybrid (Recommended):
   - HNSW with inline filtering
   - Oversample by 2x-5x
   - Filter during candidate generation
   - Guaranteed k results (if exist)
   ```

3. **API Design for Strategy Selection**
   ```rust
   pub enum FilterStrategy {
       PreFilter,      // Filter first, then search
       PostFilter,     // Search first, then filter
       Hybrid(f32),    // Oversample factor (e.g., 2.0)
       Auto,           // Let EdgeVec decide based on selectivity
   }
   ```

#### Acceptance Criteria (MEASURABLE)

- [ ] All four strategies analyzed with complexity (Pre, Post, Hybrid, Filterable HNSW)
- [ ] **BINARY CHECK**: Performance model shows <10ms P99 at 100k for all tiers
- [ ] Recommended strategy documented with quantitative rationale
- [ ] Edge cases handled: empty result set, all filtered, 0% selectivity, 100% selectivity
- [ ] API design for strategy selection with 4 enum variants
- [ ] Selectivity estimation approach documented with threshold values

#### Filter Strategy Decision Matrix (REQUIRED)

```
Strategy Selection Criteria:
┌─────────────────┬────────────────┬─────────────────────────────────┐
│ Selectivity     │ Strategy       │ Rationale                       │
├─────────────────┼────────────────┼─────────────────────────────────┤
│ <10% pass       │ Post-filter    │ Bitset overhead > search speedup│
│ 10-50% pass     │ Hybrid         │ Pre-filter + HNSW on subset     │
│ >50% pass       │ Pre-filter     │ Bitset scan faster than full KNN│
│ Unknown         │ Auto           │ Measure first 100 queries, adapt│
└─────────────────┴────────────────┴─────────────────────────────────┘
```

#### Performance Tiers (REQUIRED)

| Tier | Filter Type | Example | P99 Target |
|:-----|:------------|:--------|:-----------|
| Tier 1 | Simple equality | `$key = value` | <2ms |
| Tier 2 | Range | `$key BETWEEN x y` | <5ms |
| Tier 3 | Complex AND/OR | `($a AND $b) OR $c` | <10ms |
| Tier 4 | Worst-case | 3+ clauses + negations | <20ms |

---

### Day 4 (W22.4): WASM Boundary & TypeScript API Design

**Task ID:** W22.4
**Priority:** P1 HIGH
**Effort:** 6 hours
**Type:** Architecture/Design
**Depends On:** W22.1, W22.2, W22.3
**Blocks:** W22.5

#### Objective

Design the JavaScript/TypeScript API for filtering that is ergonomic, type-safe, and efficient across the WASM boundary.

#### Deliverables

1. **`docs/architecture/FILTERING_WASM_API.md`**
   - TypeScript interface definitions
   - String-based vs builder pattern API comparison
   - Serialization format for filter expressions
   - Error handling across WASM boundary

2. **API Design Options**

   **Option A: String-based (SQL-like)**
   ```typescript
   const results = index.search(query, 10, {
     filter: 'category = "gpu" AND price < 1000'
   });
   ```

   **Option B: Builder Pattern**
   ```typescript
   const filter = Filter.and(
     Filter.eq('category', 'gpu'),
     Filter.lt('price', 1000)
   );
   const results = index.search(query, 10, { filter });
   ```

   **Option C: Hybrid (Recommended)**
   ```typescript
   // String-based for simple queries
   const results1 = index.search(query, 10, {
     filter: 'category = "gpu"'
   });

   // Builder for complex/dynamic queries
   const filter = new FilterBuilder()
     .where('category').eq('gpu')
     .and('price').lt(1000)
     .build();
   const results2 = index.search(query, 10, { filter });
   ```

3. **TypeScript Type Definitions**
   ```typescript
   interface SearchOptions {
     filter?: string | Filter;
     strategy?: 'pre' | 'post' | 'hybrid' | 'auto';
     oversampleFactor?: number;
   }

   interface SearchResult {
     id: number;
     score: number;
     metadata?: Record<string, MetadataValue>;
   }
   ```

#### Acceptance Criteria (MEASURABLE)

- [ ] TypeScript interface fully specified with 10+ type definitions
- [ ] API ergonomics validated with 5+ example code snippets
- [ ] WASM serialization format documented (JSON schema or binary format)
- [ ] Error types mapped to JavaScript exceptions (5+ error types)
- [ ] **BINARY CHECK**: Bundle size impact estimated <50KB for filter module
- [ ] **BINARY CHECK**: Verify total WASM bundle still <500KB after filter additions
- [ ] **BINARY CHECK**: Passes `cargo fmt --check` on any example code
- [ ] Error handling spec complete (see below)

#### Error Handling Specification (REQUIRED)

```typescript
// Error Type Mapping
enum FilterError {
  SYNTAX_ERROR,       // Invalid filter syntax
  TYPE_MISMATCH,      // $age = "invalid" (string vs integer)
  UNKNOWN_KEY,        // Key doesn't exist in metadata
  OVERFLOW,           // Integer overflow in comparison
  INVALID_ARRAY,      // Array operation on non-array
}

// Error Handling Behavior
interface ErrorPolicy {
  unknownKeys: 'strict' | 'lenient';  // strict throws, lenient treats as false
  typeMismatch: 'error' | 'coerce';   // error throws, coerce attempts conversion
  emptyResults: 'ok' | 'warn';        // empty array is valid, optional warning
}

// Error Message Format
interface FilterException extends Error {
  code: FilterError;
  position?: { line: number; column: number };
  suggestion?: string;  // "Did you mean 'category'?"
}
```

---

### Day 5 (W22.5): Test Strategy & FILTERING_API.md Finalization

**Task ID:** W22.5
**Priority:** P0 CRITICAL
**Effort:** 8 hours
**Type:** Architecture/Documentation
**Depends On:** W22.1, W22.2, W22.3, W22.4
**Blocks:** Week 23 implementation

#### Objective

Create the comprehensive test strategy and finalize the unified FILTERING_API.md architecture document for hostile review.

#### Deliverables

1. **`docs/architecture/FILTER_TEST_STRATEGY.md`**
   - Unit test plan (parser, evaluator, integration)
   - Property test invariants
   - Fuzz testing targets
   - Performance benchmark specifications
   - WASM-specific test plan

2. **Test Categories**

   | Category | Tests | Coverage Target |
   |:---------|:------|:----------------|
   | Parser unit | 100+ | All grammar rules |
   | Evaluator unit | 50+ | All operators |
   | Type coercion | 25+ | All type pairs |
   | Integration | 20+ | End-to-end search |
   | Property tests | 1000+ inputs | Invariants |
   | Fuzz tests | 10M+ inputs | Crash resistance |
   | Performance | 10 benchmarks | <10ms P99 |
   | WASM | 30+ | All JS API paths |

3. **Property Test Invariants (EXHAUSTIVE - 12 REQUIRED)**

   ```rust
   // ═══════════════════════════════════════════════════════════════════
   // FILTER PROPERTY TEST INVARIANTS (ALL MUST BE TESTED)
   // ═══════════════════════════════════════════════════════════════════

   // LOGICAL INVARIANTS (6)
   // 1. Double negation: NOT NOT expr ≡ expr
   prop_double_negation: ∀ filter. NOT(NOT(filter)) == filter

   // 2. De Morgan (AND): NOT (a AND b) ≡ (NOT a) OR (NOT b)
   prop_de_morgan_and: ∀ a, b. NOT(a AND b) == (NOT a) OR (NOT b)

   // 3. De Morgan (OR): NOT (a OR b) ≡ (NOT a) AND (NOT b)
   prop_de_morgan_or: ∀ a, b. NOT(a OR b) == (NOT a) AND (NOT b)

   // 4. Idempotence: a AND a ≡ a, a OR a ≡ a
   prop_idempotence: ∀ filter. (filter AND filter) == filter AND (filter OR filter) == filter

   // 5. Commutativity: a AND b ≡ b AND a, a OR b ≡ b OR a
   prop_commutativity: ∀ a, b. (a AND b) == (b AND a) AND (a OR b) == (b OR a)

   // 6. Associativity: (a AND b) AND c ≡ a AND (b AND c)
   prop_associativity: ∀ a, b, c. ((a AND b) AND c) == (a AND (b AND c))

   // IDENTITY INVARIANTS (3)
   // 7. Empty filter identity: Empty filter returns all vectors
   prop_empty_filter: filter(∅) == all_vectors

   // 8. TRUE identity: filter(TRUE) returns all vectors
   prop_true_identity: filter(TRUE) == all_vectors

   // 9. FALSE identity: filter(FALSE) returns empty set
   prop_false_identity: filter(FALSE) == ∅

   // CONTRADICTION INVARIANTS (2)
   // 10. Contradictory filter: Opposite filters = empty set
   prop_contradiction: ∀ filter. filter AND NOT(filter) == ∅

   // 11. Tautology: filter OR NOT(filter) returns all vectors
   prop_tautology: ∀ filter. filter OR NOT(filter) == all_vectors

   // RANGE EQUIVALENCE (1)
   // 12. Range equivalence: Different expressions with same semantics
   prop_range_equivalence: ($age > 5 AND $age < 10) == ($age BETWEEN 6 9)

   // ═══════════════════════════════════════════════════════════════════
   // ADDITIONAL INVARIANTS (Non-logical, 5 required)
   // ═══════════════════════════════════════════════════════════════════

   // 13. Metadata preservation: Filtering doesn't modify metadata
   prop_metadata_preservation: ∀ v. metadata(filter(v)) == metadata(v)

   // 14. Index stability: Filter results consistent across index rebuilds
   prop_index_stability: ∀ filter. filter(index_v1) == filter(rebuild(index_v1))

   // 15. Order independence: Filter order doesn't affect results
   prop_order_independence: filter_then_search == search_then_filter (same result set)

   // 16. Subset guarantee: filter(a AND b) ⊆ filter(a)
   prop_subset_guarantee: ∀ a, b. filter(a AND b) ⊆ filter(a)

   // 17. Superset guarantee: filter(a) ⊆ filter(a OR b)
   prop_superset_guarantee: ∀ a, b. filter(a) ⊆ filter(a OR b)
   ```

   **Test Generation Target:** Each invariant × 100 random inputs = 1,700+ property tests

4. **`docs/architecture/FILTERING_API.md`** (Master Document)
   - Consolidates all Day 1-4 documents
   - Executive summary
   - Complete API specification
   - Implementation roadmap for Week 23
   - Risk analysis

#### Acceptance Criteria (MEASURABLE)

- [ ] Test strategy covers all 6 components (parser, evaluator, integration, WASM, property, fuzz)
- [ ] **BINARY CHECK**: All 17 property test invariants formally specified
- [ ] Fuzz test targets identified (5+ fuzzing entry points)
- [ ] Performance benchmarks defined for all 4 tiers with thresholds
- [ ] **BINARY CHECK**: FILTERING_API.md >5,000 words and comprehensive
- [ ] Week 23 implementation tasks derived from architecture (10+ tasks)
- [ ] WASM boundary tests explicitly included (10+ tests)

#### Test Plan Coverage (DERIVED, NOT ARBITRARY)

```
Test Plan Coverage Breakdown:
┌────────────────────────┬───────┬─────────────────────────────────────┐
│ Category               │ Count │ Derivation                          │
├────────────────────────┼───────┼─────────────────────────────────────┤
│ Grammar rules          │ 30-50 │ 1 test per EBNF rule                │
│ Operators (happy path) │ 15    │ 1 test per operator                 │
│ Operators (edge cases) │ 30    │ 2 edge cases per operator           │
│ Operators (errors)     │ 15    │ 1 error case per operator           │
│ Combinations           │ 20    │ Sample of 2^N combinations          │
│ Integration            │ 10    │ 1 test per HNSW integration point   │
│ Performance tiers      │ 4     │ 1 test per tier                     │
│ Property invariants    │ 17    │ All 17 invariants                   │
│ Property random inputs │ 1700  │ 17 invariants × 100 inputs each     │
│ Fuzz entry points      │ 5     │ Parser, evaluator, WASM boundary    │
│ WASM boundary          │ 10    │ Serialization, errors, memory       │
├────────────────────────┼───────┼─────────────────────────────────────┤
│ TOTAL                  │ 1856+ │ Coverage-derived (not arbitrary)    │
└────────────────────────┴───────┴─────────────────────────────────────┘
```

---

## Sprint Success Criteria

| Criterion | Threshold | Verification |
|:----------|:----------|:-------------|
| EBNF grammar complete | Unambiguous, parseable | pest/peg validation |
| All operators specified | 15+ operators | Document review |
| Strategy decision made | One of pre/post/hybrid | Documented rationale |
| Performance validated | <10ms modeled | Complexity analysis |
| TypeScript API designed | Full type coverage | TS compile check |
| Test strategy complete | 200+ tests planned | Test matrix review |
| FILTERING_API.md approved | Hostile review pass | /review command |

---

## Risk Analysis

| Risk | Probability | Impact | Mitigation |
|:-----|:------------|:-------|:-----------|
| Grammar ambiguity | Medium | High | Multiple reviewers |
| Performance budget exceeded | Low | Critical | Pre-modeling |
| WASM boundary complexity | Medium | Medium | Existing patterns |
| Scope creep into implementation | Medium | High | Strict enforcement |

---

## Constraints

### HARD CONSTRAINTS (IMMUTABLE)

1. **NO IMPLEMENTATION CODE** - Design documents only
2. **Metadata Schema v1.0 is FROZEN** - Cannot change types
3. **<10ms P99 at 100k × 384-dim vectors** - Performance budget
4. **WASM bundle <500KB** - Size budget (currently 248KB, filter module <50KB)
5. **All tasks <16 hours** - Decomposition required
6. **All Rust code must pass `cargo fmt --check`** - Formatting standard

### SOFT CONSTRAINTS (NEGOTIABLE)

1. SQL-like syntax (can be adjusted based on review)
2. Builder pattern for JS (can be string-only)
3. Hybrid filter strategy (can be post-filter if simpler)

---

## Dependencies

### External Dependencies

None - this is a pure design sprint.

### Internal Dependencies

```
W22.1 (Syntax) ──┬──> W22.2 (Evaluator) ──┬──> W22.5 (Finalization)
                 │                        │
                 └──> W22.3 (Strategy) ───┘
                 │
                 └──> W22.4 (WASM API) ───┘
```

---

## Verification Method

Week 22 is **COMPLETE** when:

1. `docs/architecture/FILTERING_API.md` exists and is comprehensive
2. EBNF grammar is formally specified and parseable
3. All technical decisions are documented with rationale
4. Test strategy covers 200+ planned tests
5. Performance budget is validated via modeling
6. HOSTILE_REVIEWER approves FILTERING_API.md
7. `.claude/GATE_W22_COMPLETE.md` is created

---

## Hostile Review Checkpoint

**End of Day 5:** Submit via `/review FILTERING_API.md`

**Expected Review Focus:**
- Grammar completeness and unambiguity
- Performance budget feasibility
- API ergonomics
- Test coverage adequacy
- Implementation feasibility for Week 23

---

## Handoff to Week 23

**Week 23 Theme:** Filtering Implementation

**Prerequisites:**
- FILTERING_API.md approved by HOSTILE_REVIEWER
- All design documents complete (FILTERING_SYNTAX.md, FILTER_EVALUATOR.md, FILTER_STRATEGY.md, FILTERING_WASM_API.md, FILTER_TEST_STRATEGY.md)

**Week 23 Scope (From Roadmap - 10+ Tasks):**
1. Query parser implementation using pest (12h)
2. Filter evaluator implementation (12h)
3. HNSW search integration with filtering (9h)
4. WASM bindings for filter API (6h)
5. Parser unit tests (4h)
6. Evaluator unit tests (5h)
7. Property tests for 17 invariants (6h)
8. Integration tests with HNSW (4h)
9. WASM boundary tests (3h)
10. API documentation (3h)
11. Performance benchmarks validation (4h)

**Total Estimated Effort:** 68 hours (with 3x padding: 204 hours = ~5 week-days)

**Identify Next Sprint (Week 24):**
Week 24 will focus on Advanced Query Features OR Performance Optimization based on Week 23 outcomes. Specific topic selection and 1-paragraph scope definition will be created at end of Week 23 (full gap analysis deferred to Week 24 planning).

---

## Approval

```
+---------------------------------------------------------------------+
|   PLANNER: WEEK 22 TASK PLAN                                        |
|                                                                     |
|   Theme: Filtering Architecture Design Sprint                       |
|   Days: 5                                                           |
|   Total Effort: 36 hours                                            |
|   Type: Design Only (NO CODE)                                       |
|                                                                     |
|   Status: PENDING HOSTILE_REVIEWER APPROVAL                         |
|                                                                     |
+---------------------------------------------------------------------+
```

---

**Task Owner:** META_ARCHITECT / PLANNER
**Review Required:** HOSTILE_REVIEWER
**Next Phase:** Week 23 (Filtering Implementation)

---

*"Architecture is not about guessing. It's about knowing before you build."*

---

## Research Sources

Industry research conducted 2025-12-17:

### Pinecone
- [Pinecone Metadata Filtering](https://docs.pinecone.io/docs/metadata-filtering)
- [Accurate and Efficient Metadata Filtering](https://www.pinecone.io/research/accurate-and-efficient-metadata-filtering-in-pinecones-serverless-vector-database/)
- [The Missing WHERE Clause in Vector Search](https://www.pinecone.io/learn/vector-search-filtering/)

### Milvus
- [Milvus Filtering Explained](https://milvus.io/docs/boolean.md)
- [Boolean Expression Rules](https://milvus.io/docs/v2.3.x/boolean.md)
- [Generating Query Filter Expressions](https://milvus.io/docs/generating_milvus_query_filter_expressions.md)

### Qdrant
- [Qdrant Filtering](https://qdrant.tech/documentation/concepts/filtering/)
- [Complete Guide to Filtering in Vector Search](https://qdrant.tech/articles/vector-search-filtering/)
- [Vector Database Benchmarks](https://qdrant.tech/benchmarks/)

### Weaviate
- [Weaviate Conditional Filters](https://weaviate.io/developers/weaviate/api/graphql/filters)
- [Weaviate Search Filters](https://weaviate.io/developers/weaviate/search/filters)
- [Filtering Concepts](https://weaviate.io/developers/weaviate/concepts/filtering)

### General Vector Search Filtering
- [Pre-filtering vs Post-filtering in Vector Search](https://apxml.com/courses/advanced-vector-search-llms/chapter-2-optimizing-vector-search-performance/advanced-filtering-strategies)
- [The Achilles Heel of Vector Search: Filters](https://yudhiesh.github.io/2025/05/09/the-achilles-heel-of-vector-search-filters/)
- [All About Filtered Vector Search](https://www.myscale.com/blog/filtered-vector-search-in-myscale/)
