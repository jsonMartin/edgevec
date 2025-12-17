# EdgeVec Filtering API - Complete Specification

**Document:** `FILTERING_API.md`
**Version:** 1.0.0
**Status:** [PROPOSED]
**Author:** META_ARCHITECT
**Date:** 2025-12-17
**Week:** 22 | **Day:** 5 | **Task:** W22.5 (Master Document)

---

## Executive Summary

This document is the **authoritative specification** for EdgeVec's metadata filtering subsystem. It consolidates the designs from Week 22 Days 1-4 into a single reference for Week 23 implementation.

**Scope:**
- Query syntax and grammar (from FILTERING_SYNTAX.md)
- Evaluation algorithm and AST design (from FILTER_EVALUATOR.md)
- Strategy selection and performance (from FILTER_STRATEGY.md)
- WASM/JavaScript API (from FILTERING_WASM_API.md)
- Test strategy (from FILTER_TEST_STRATEGY.md)

**Key Metrics:**
- 38 EBNF grammar rules
- 27 AST node variants
- 17 property test invariants
- 5 fuzz targets
- <10ms P99 latency target at 100k vectors

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Query Syntax Reference](#2-query-syntax-reference)
3. [Type System](#3-type-system)
4. [Evaluator Architecture](#4-evaluator-architecture)
5. [Strategy Selection](#5-strategy-selection)
6. [Rust API](#6-rust-api)
7. [WASM/JavaScript API](#7-wasmjavascript-api)
8. [Error Handling](#8-error-handling)
9. [Performance Specifications](#9-performance-specifications)
10. [Test Requirements](#10-test-requirements)
11. [Implementation Roadmap](#11-implementation-roadmap)
12. [Risk Analysis](#12-risk-analysis)
13. [Appendices](#appendices)

---

## 1. System Overview

### 1.1 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    EDGEVEC FILTERING ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌─────────────┐  │
│  │  Query       │   │   Parser     │   │  Evaluator   │   │  Strategy   │  │
│  │  String      │──►│  (pest)      │──►│  (tree-walk) │──►│  Selection  │  │
│  │              │   │              │   │              │   │             │  │
│  │ "price < 500"│   │  38 rules    │   │  27 variants │   │  4 modes    │  │
│  └──────────────┘   └──────────────┘   └──────────────┘   └─────────────┘  │
│                            │                  │                  │          │
│                            ▼                  ▼                  ▼          │
│                      ┌──────────┐       ┌──────────┐      ┌──────────┐     │
│                      │   AST    │       │  bool    │      │ Results  │     │
│                      │FilterExpr│       │ Result   │      │  top-k   │     │
│                      └──────────┘       └──────────┘      └──────────┘     │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       WASM BOUNDARY                                  │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐  │   │
│  │  │ parse_filter_js │  │ FilterBuilder   │  │ search_with_filter  │  │   │
│  │  │ (string → AST)  │  │ (fluent API)    │  │ (query → results)   │  │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Data Flow

```
1. User provides filter string: "category = \"gpu\" AND price < 500"
2. Parser tokenizes and builds AST (FilterExpr tree)
3. Strategy selector estimates selectivity
4. HNSW search executes with chosen strategy
5. Evaluator filters candidates
6. Results returned to user
```

### 1.3 Design Principles

| Principle | Description | Implementation |
|:----------|:------------|:---------------|
| **SQL-like Syntax** | Familiar to developers | Milvus-inspired grammar |
| **Strict Typing** | No implicit coercion surprises | Type-checked at parse time |
| **Short-Circuit** | Efficient logical evaluation | Left-to-right AND/OR |
| **Fail-Fast** | Clear error messages | 16 error types with context |
| **WASM-First** | Browser-native performance | Zero-copy where possible |

---

## 2. Query Syntax Reference

### 2.1 Grammar Summary

The complete EBNF grammar contains **38 rules** (see FILTERING_SYNTAX.md for full specification).

**Top-Level:**
```ebnf
filter_expr     = { whitespace } , logical_expr , { whitespace } , EOF ;
logical_expr    = or_expr ;
or_expr         = and_expr , { or_op , and_expr } ;
and_expr        = not_expr , { and_op , not_expr } ;
not_expr        = [ not_op ] , primary_expr ;
```

### 2.2 Operator Quick Reference

| Category | Operators | Example |
|:---------|:----------|:--------|
| Comparison | `=`, `!=`, `<`, `<=`, `>`, `>=` | `price >= 100` |
| String | `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`, `LIKE` | `title CONTAINS "GPU"` |
| Array | `IN`, `NOT IN`, `ANY`, `ALL`, `NONE` | `tags ANY ["a", "b"]` |
| Range | `BETWEEN` | `price BETWEEN 100 500` |
| Null | `IS NULL`, `IS NOT NULL` | `field IS NULL` |
| Logical | `AND`, `OR`, `NOT` | `a AND b OR c` |

### 2.3 Operator Precedence

| Level | Operators | Associativity |
|:------|:----------|:--------------|
| 1 (lowest) | `OR`, `\|\|` | Left |
| 2 | `AND`, `&&` | Left |
| 3 | `NOT`, `!` | Right (prefix) |
| 4 | Comparison | None |
| 5 | String ops | None |
| 6 (highest) | Array/Null/Range | None |

### 2.4 Query Examples

**Simple Queries:**
```sql
category = "gpu"
price < 500
is_active = true
rating >= 4.5
```

**String Operations:**
```sql
title CONTAINS "NVIDIA"
name STARTS_WITH "Dr."
email ENDS_WITH "@example.com"
description LIKE "GPU%"
```

**Array Operations:**
```sql
category IN ["gpu", "cpu", "tpu"]
tags ANY ["premium", "featured"]
required_tags ALL ["verified", "active"]
```

**Complex Queries:**
```sql
(category = "gpu" OR category = "tpu") AND price < 1000
NOT (status = "draft" OR status = "archived")
price BETWEEN 100 500 AND rating >= 4.0
(tags ANY ["premium"] OR rating >= 4.5) AND is_active = true
```

---

## 3. Type System

### 3.1 Metadata Types

| Type | Rust | JavaScript | Size Limit | Example |
|:-----|:-----|:-----------|:-----------|:--------|
| String | `String` | `string` | 65,536 bytes | `"hello"` |
| Integer | `i64` | `number` (safe) | ±2^53 | `42` |
| Float | `f64` | `number` | Finite only | `3.14159` |
| Boolean | `bool` | `boolean` | - | `true` |
| StringArray | `Vec<String>` | `string[]` | 1,024 elements | `["a", "b"]` |

### 3.2 Operator-Type Matrix

| Operator | String | Integer | Float | Boolean | StringArray |
|:---------|:-------|:--------|:------|:--------|:------------|
| `=`, `!=` | Yes | Yes | Yes | Yes | No |
| `<`, `<=`, `>`, `>=` | No | Yes | Yes | No | No |
| `BETWEEN` | No | Yes | Yes | No | No |
| `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`, `LIKE` | Yes | No | No | No | No |
| `IN`, `NOT IN` | Yes | Yes | Yes | No | No |
| `ANY`, `ALL`, `NONE` | No | No | No | No | Yes |
| `IS NULL`, `IS NOT NULL` | Yes | Yes | Yes | Yes | Yes |

### 3.3 Type Coercion Rules

| Left Type | Operator | Right Type | Behavior |
|:----------|:---------|:-----------|:---------|
| Integer | comparison | Float | Integer promoted to Float |
| Float | comparison | Integer | Integer promoted to Float |
| Any | any | mismatched | `FilterError::TypeMismatch` |

---

## 4. Evaluator Architecture

### 4.1 AST Node Types (27 Variants)

```rust
pub enum FilterExpr {
    // Literals (5)
    LiteralString(String),
    LiteralInt(i64),
    LiteralFloat(f64),
    LiteralBool(bool),
    LiteralArray(Vec<FilterExpr>),

    // Field Access (1)
    Field(String),

    // Comparison (6)
    Eq(Box<FilterExpr>, Box<FilterExpr>),
    Ne(Box<FilterExpr>, Box<FilterExpr>),
    Lt(Box<FilterExpr>, Box<FilterExpr>),
    Le(Box<FilterExpr>, Box<FilterExpr>),
    Gt(Box<FilterExpr>, Box<FilterExpr>),
    Ge(Box<FilterExpr>, Box<FilterExpr>),

    // String Operations (4)
    Contains(Box<FilterExpr>, Box<FilterExpr>),
    StartsWith(Box<FilterExpr>, Box<FilterExpr>),
    EndsWith(Box<FilterExpr>, Box<FilterExpr>),
    Like(Box<FilterExpr>, Box<FilterExpr>),

    // Array Operations (5)
    In(Box<FilterExpr>, Box<FilterExpr>),
    NotIn(Box<FilterExpr>, Box<FilterExpr>),
    Any(Box<FilterExpr>, Box<FilterExpr>),
    All(Box<FilterExpr>, Box<FilterExpr>),
    None(Box<FilterExpr>, Box<FilterExpr>),

    // Range (1)
    Between(Box<FilterExpr>, Box<FilterExpr>, Box<FilterExpr>),

    // Logical (3)
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
    Not(Box<FilterExpr>),

    // Null Checks (2)
    IsNull(Box<FilterExpr>),
    IsNotNull(Box<FilterExpr>),
}
```

### 4.2 Evaluation Algorithm

```rust
pub fn evaluate(
    expr: &FilterExpr,
    metadata: &HashMap<String, MetadataValue>,
) -> Result<bool, FilterError> {
    match expr {
        // Logical operators with short-circuit
        FilterExpr::And(left, right) => {
            if !evaluate(left, metadata)? {
                return Ok(false); // Short-circuit
            }
            evaluate(right, metadata)
        }
        FilterExpr::Or(left, right) => {
            if evaluate(left, metadata)? {
                return Ok(true); // Short-circuit
            }
            evaluate(right, metadata)
        }
        // ... other operators
    }
}
```

**Complexity:**
- Time: O(n) where n = AST node count
- Space: O(d) where d = nesting depth (stack frames)

### 4.3 Short-Circuit Semantics

| Expression | Left Result | Behavior |
|:-----------|:------------|:---------|
| `A AND B` | `false` | Return `false`, skip B |
| `A AND B` | `true` | Evaluate B, return result |
| `A OR B` | `true` | Return `true`, skip B |
| `A OR B` | `false` | Evaluate B, return result |
| `A AND B` | `Error` | Return error, skip B |
| `A OR B` | `Error` | Return error, skip B |

---

## 5. Strategy Selection

### 5.1 Available Strategies

| Strategy | When to Use | Complexity | Memory |
|:---------|:------------|:-----------|:-------|
| **PreFilter** | Selectivity > 80% | O(n) | O(n/8) bitset |
| **PostFilter** | Selectivity < 5% | O(log n × ef) | O(ef) |
| **Hybrid** | Unknown selectivity | O(log n × k × os) | O(k × os) |
| **Auto** | Default (recommended) | Adaptive | Adaptive |

### 5.2 FilterStrategy Enum

```rust
pub enum FilterStrategy {
    PostFilter { oversample: f32 },
    PreFilter,
    Hybrid { oversample_min: f32, oversample_max: f32 },
    Auto,
}
```

### 5.3 Auto-Selection Algorithm

```rust
fn auto_select(selectivity: f32) -> FilterStrategy {
    match selectivity {
        s if s > 0.80 => FilterStrategy::PreFilter,
        s if s < 0.05 => FilterStrategy::PostFilter { oversample: 3.0 },
        _ => FilterStrategy::Hybrid {
            oversample_min: 1.5,
            oversample_max: 10.0,
        },
    }
}
```

### 5.4 Performance by Strategy

| Selectivity | PreFilter | PostFilter | Hybrid | Target |
|:------------|:----------|:-----------|:-------|:-------|
| 1% | 15ms | 8ms* | 6ms | <10ms |
| 10% | 14ms | 5ms | 4ms | <10ms |
| 50% | 12ms | 10ms | 5ms | <10ms |
| 90% | 8ms | 15ms | 9ms | <10ms |

*May return incomplete results

---

## 6. Rust API

### 6.1 Core Functions

```rust
// Parse a filter string
pub fn parse(input: &str) -> Result<FilterExpr, FilterError>;

// Evaluate filter against metadata
pub fn evaluate(
    expr: &FilterExpr,
    metadata: &HashMap<String, MetadataValue>,
) -> Result<bool, FilterError>;

// Search with filter
impl HnswIndex {
    pub fn search_filtered(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&FilterExpr>,
        strategy: FilterStrategy,
    ) -> Result<FilteredSearchResult, FilterError>;
}
```

### 6.2 Result Types

```rust
pub struct FilteredSearchResult {
    pub results: Vec<SearchResult>,
    pub complete: bool,
    pub observed_selectivity: f32,
    pub strategy_used: FilterStrategy,
}
```

### 6.3 Usage Example

```rust
use edgevec::filter::{parse, FilterStrategy};

let index = HnswIndex::new(/* ... */);

// Add vectors with metadata
index.insert(vector, Some(hashmap! {
    "category" => "gpu",
    "price" => 499,
    "rating" => 4.7,
}));

// Search with filter
let filter = parse(r#"category = "gpu" AND price < 500"#)?;
let results = index.search_filtered(
    &query,
    10,
    Some(&filter),
    FilterStrategy::Auto,
)?;

for r in results.results {
    println!("ID: {}, Distance: {}", r.id, r.distance);
}
```

---

## 7. WASM/JavaScript API

### 7.1 Core Functions

```typescript
// Parse a filter string
export function parse_filter(input: string): FilterExpr;

// Search with filter
export function search_with_filter(
    index: EdgeVecIndex,
    query: Float32Array,
    k: number,
    filter: string | FilterExpr,
    options?: SearchOptions
): Promise<SearchResult[]>;
```

### 7.2 FilterBuilder API

```typescript
const filter = new FilterBuilder()
    .eq('category', 'gpu')
    .and()
    .lt('price', 500)
    .and()
    .gte('rating', 4.0)
    .build();

// Produces: category = "gpu" AND price < 500 AND rating >= 4.0
```

### 7.3 SearchOptions

```typescript
interface SearchOptions {
    strategy?: 'auto' | 'prefilter' | 'postfilter' | 'hybrid';
    oversample?: number;
    includeVectors?: boolean;
    includeMetadata?: boolean;
}
```

### 7.4 TypeScript Types

```typescript
type FilterExpr =
    | { type: 'LiteralString'; value: string }
    | { type: 'LiteralInt'; value: number }
    | { type: 'LiteralFloat'; value: number }
    | { type: 'LiteralBool'; value: boolean }
    | { type: 'Field'; name: string }
    | { type: 'Eq'; left: FilterExpr; right: FilterExpr }
    // ... all 27 variants

interface FilterError {
    code: string;
    message: string;
    position?: number;
    suggestion?: string;
}
```

### 7.5 JavaScript Usage Example

```javascript
import { EdgeVec, FilterBuilder } from '@edgevec/wasm';

const index = await EdgeVec.create({
    dimensions: 384,
    metric: 'cosine',
});

// Insert with metadata
await index.insert(vector, {
    category: 'gpu',
    price: 499,
    tags: ['nvidia', 'rtx'],
});

// Search with string filter
const results = await index.search_with_filter(
    query,
    10,
    'category = "gpu" AND price < 500'
);

// Search with builder
const filter = new FilterBuilder()
    .field('tags').any(['nvidia', 'amd'])
    .build();

const results2 = await index.search_with_filter(query, 10, filter);
```

---

## 8. Error Handling

### 8.1 Error Categories

| Category | Codes | Description |
|:---------|:------|:------------|
| Syntax | E001-E005 | Parser errors |
| Type | E101-E105 | Type mismatches |
| Value | E201-E204 | Value limits exceeded |
| Structure | E301-E302 | Complexity limits |

### 8.2 Error Codes Reference

| Code | Name | Example Cause |
|:-----|:-----|:--------------|
| E001 | `UNEXPECTED_TOKEN` | `price >> 100` |
| E002 | `UNEXPECTED_EOF` | `price >` |
| E003 | `INVALID_STRING` | `"unclosed` |
| E004 | `INVALID_NUMBER` | `12.34.56` |
| E005 | `INVALID_IDENTIFIER` | `123field` |
| E101 | `TYPE_MISMATCH` | `name > 100` |
| E102 | `INVALID_OPERATOR` | `is_active IN [1]` |
| E103 | `ARRAY_TYPE_MISMATCH` | `[1, "a"]` |
| E104 | `COERCION_FAILED` | Internal error |
| E105 | `UNKNOWN_FIELD` | Field not in metadata |
| E201 | `INTEGER_OVERFLOW` | 10^20 |
| E202 | `FLOAT_NOT_FINITE` | NaN, Infinity |
| E203 | `STRING_TOO_LONG` | >65KB string |
| E204 | `ARRAY_TOO_LONG` | >1024 elements |
| E301 | `NESTING_TOO_DEEP` | >5 levels |
| E302 | `EXPRESSION_TOO_COMPLEX` | >100 nodes |

### 8.3 Error Messages

All errors include:
- Error code (e.g., `E001`)
- Human-readable message
- Position in input (for syntax errors)
- Suggestion for fixing (when possible)

```javascript
try {
    const filter = parse_filter('price >> 100');
} catch (e) {
    // e.code = 'E001'
    // e.message = "Unexpected token '>>' at position 6"
    // e.position = 6
    // e.suggestion = "Did you mean '>' or '>='?"
}
```

---

## 9. Performance Specifications

### 9.1 Latency Targets

| Scenario | Vectors | Filter Complexity | Target P99 |
|:---------|:--------|:------------------|:-----------|
| Simple filter | 100k | 1 clause | <5ms |
| Medium filter | 100k | 3 clauses | <7ms |
| Complex filter | 100k | 5+ clauses | <10ms |
| No filter | 100k | None | <2ms |

### 9.2 Component Latencies

| Operation | Target | Notes |
|:----------|:-------|:------|
| Filter parsing | <100μs | Typical query |
| Filter evaluation (per vector) | <1μs | Simple filter |
| Selectivity estimation | <100μs | 100 samples |
| Strategy selection | <10μs | Auto mode |

### 9.3 Memory Limits

| Component | Limit | Enforcement |
|:----------|:------|:------------|
| AST size | 10KB | `E302` error |
| AST nodes | 100 | `E302` error |
| Nesting depth | 5 | `E301` error |
| String length | 65,536 bytes | `E203` error |
| Array length | 1,024 elements | `E204` error |
| ef_search cap | 1,000 | Hard limit |

### 9.4 WASM-Specific Limits

| Resource | Browser Limit | EdgeVec Limit |
|:---------|:--------------|:--------------|
| Heap memory | 256MB typical | 128MB recommended |
| Stack | 1MB | 5 levels max |
| ef_search | - | 500 (conservative) |

---

## 10. Test Requirements

### 10.1 Test Coverage

| Component | Unit Tests | Property Tests | Fuzz Targets |
|:----------|:-----------|:---------------|:-------------|
| Parser | 344 | 5 | 1 |
| Evaluator | 804 | 7 | 2 |
| Strategy | 408 | 3 | 0 |
| WASM | 300 | 2 | 2 |
| **TOTAL** | **1,856** | **17** | **5** |

### 10.2 Property Test Invariants (17)

**Parser (5):**
1. Parse-serialize roundtrip
2. Valid input never panics
3. Empty input returns error
4. Whitespace normalization
5. Keyword case insensitivity

**Evaluator (7):**
1. NOT involution (NOT(NOT(x)) == x)
2. AND commutativity
3. OR commutativity
4. De Morgan's laws
5. Short-circuit correctness
6. Type consistency
7. NULL semantics (XOR)

**Strategy (3):**
1. Strategy result equivalence
2. Auto selection stability
3. Oversample bounds

**WASM (2):**
1. JSON roundtrip
2. TypeScript type safety

### 10.3 Fuzz Targets (5)

1. `fuzz_parser` - Random strings
2. `fuzz_evaluator` - Random AST + metadata
3. `fuzz_like_pattern` - Pathological patterns
4. `fuzz_json_roundtrip` - JSON serialization
5. `fuzz_wasm_boundary` - WASM input handling

### 10.4 Acceptance Criteria

**CRITICAL (Must Pass):**
- [ ] All 1,856 unit tests pass
- [ ] All 17 property tests pass
- [ ] All 5 fuzz targets run 10+ minutes without findings
- [ ] 100% coverage of public API

**MAJOR (Should Pass):**
- [ ] All performance benchmarks meet targets
- [ ] WASM tests pass in Chrome, Firefox, Safari, Node.js

---

## 11. Implementation Roadmap

### 11.1 Week 23 Tasks

| Task ID | Description | Est. Hours | Priority | Dependency |
|:--------|:------------|:-----------|:---------|:-----------|
| W23.1 | Parser implementation (pest) | 8 | P0 | - |
| W23.2 | FilterExpr enum | 2 | P0 | - |
| W23.3 | Evaluator core | 6 | P0 | W23.2 |
| W23.4 | Comparison operators | 4 | P0 | W23.3 |
| W23.5 | String operators | 4 | P0 | W23.3 |
| W23.6 | Array operators | 3 | P0 | W23.3 |
| W23.7 | LIKE pattern matching | 2 | P0 | W23.5 |
| W23.8 | Error types | 2 | P0 | - |
| W23.9 | FilterStrategy enum | 2 | P0 | - |
| W23.10 | Post-filter strategy | 4 | P0 | W23.3 |
| W23.11 | Pre-filter strategy | 3 | P1 | W23.3 |
| W23.12 | Hybrid strategy | 4 | P0 | W23.10 |
| W23.13 | Auto-selection | 2 | P0 | W23.12 |
| W23.14 | Selectivity estimation | 3 | P0 | - |
| W23.15 | WASM bindings | 6 | P0 | W23.1-13 |
| W23.16 | FilterBuilder (JS) | 4 | P1 | W23.15 |
| W23.17 | Unit tests | 12 | P0 | All |
| W23.18 | Property tests | 6 | P0 | All |
| W23.19 | Fuzz targets | 4 | P0 | W23.1, W23.3 |
| W23.20 | Benchmarks | 4 | P1 | All |
| **TOTAL** | | **85h** | | |

### 11.2 Week 23 Schedule

| Day | Tasks | Hours |
|:----|:------|:------|
| Day 1 | W23.1, W23.2, W23.8 | 12h |
| Day 2 | W23.3, W23.4, W23.5 | 14h |
| Day 3 | W23.6, W23.7, W23.9, W23.10 | 11h |
| Day 4 | W23.11, W23.12, W23.13, W23.14 | 12h |
| Day 5 | W23.15, W23.16 | 10h |
| Day 6 | W23.17, W23.18 | 18h |
| Day 7 | W23.19, W23.20, buffer | 8h |
| **TOTAL** | | **85h** |

### 11.3 Deliverables

| Artifact | Location | Gate |
|:---------|:---------|:-----|
| `src/filter/mod.rs` | Module root | GATE_W23_PLANNING |
| `src/filter/parser.rs` | Parser (pest) | GATE_W23_PLANNING |
| `src/filter/evaluator.rs` | Evaluator | GATE_W23_PLANNING |
| `src/filter/strategy.rs` | Strategy selection | GATE_W23_PLANNING |
| `src/filter/error.rs` | Error types | GATE_W23_PLANNING |
| `src/wasm/filter.rs` | WASM bindings | GATE_W23_PLANNING |
| `tests/filter_*.rs` | Test suites | GATE_W23_COMPLETE |
| `fuzz/fuzz_targets/*.rs` | Fuzz targets | GATE_W23_COMPLETE |
| `benches/filter_*.rs` | Benchmarks | GATE_W23_COMPLETE |

---

## 12. Risk Analysis

### 12.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|:-----|:------------|:-------|:-----------|
| Parser performance | Low | Medium | Use pest (zero-copy) |
| LIKE pattern DoS | Medium | High | Iterative impl, timeout |
| Strategy misselection | Medium | Medium | Conservative defaults |
| WASM memory pressure | Medium | Medium | Lower ef_cap for WASM |
| Type coercion bugs | Medium | High | Property tests |

### 12.2 Schedule Risks

| Risk | Probability | Impact | Mitigation |
|:-----|:------------|:-------|:-----------|
| Parser complexity | Low | High | pest well-documented |
| Test coverage | Low | Medium | Test-first approach |
| WASM debugging | Medium | Medium | console.log integration |

### 12.3 Contingency Plans

**If parser takes too long:**
- Ship with string-only API (no FilterBuilder) in v0.5.0
- Add FilterBuilder in v0.5.1

**If performance targets not met:**
- Ship with higher latency, optimize in v0.5.1
- Consider Filterable HNSW for v0.6.0

**If fuzz targets find issues:**
- Fix immediately (P0)
- Delay release if critical

---

## 13. Detailed Implementation Notes

### 13.1 Parser Implementation Details

The parser will be implemented using the `pest` crate (v2.7), which provides:
- Zero-copy parsing for performance
- Automatic whitespace handling
- Clear error messages with position information
- Easy AST construction from parse trees

**Grammar File Location:** `src/filter/filter.pest`

**Parser Module Structure:**
```rust
// src/filter/parser.rs
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "filter/filter.pest"]
pub struct FilterParser;

impl FilterParser {
    /// Parse a filter string into an AST.
    ///
    /// # Arguments
    /// * `input` - The filter expression string
    ///
    /// # Returns
    /// * `Ok(FilterExpr)` - The parsed AST
    /// * `Err(FilterError)` - Parse error with position
    ///
    /// # Complexity
    /// * Time: O(n) where n = input length
    /// * Space: O(d) where d = nesting depth
    pub fn parse_filter(input: &str) -> Result<FilterExpr, FilterError> {
        let pairs = Self::parse(Rule::filter, input)
            .map_err(|e| FilterError::SyntaxError {
                position: e.location.unwrap_or(0),
                message: e.to_string(),
                suggestion: Self::suggest_fix(&e),
            })?;

        Self::build_ast(pairs)
    }

    /// Suggest a fix for common syntax errors.
    fn suggest_fix(error: &pest::error::Error<Rule>) -> Option<String> {
        // Analyze error and suggest common fixes
        match error.variant {
            pest::error::ErrorVariant::ParsingError { positives, .. } => {
                if positives.contains(&Rule::comp_op) {
                    Some("Did you mean '>' or '>='?".into())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Build AST from pest parse tree.
    fn build_ast(pairs: pest::iterators::Pairs<Rule>) -> Result<FilterExpr, FilterError> {
        // Recursive descent through parse tree
        // Convert each rule to corresponding FilterExpr variant
        todo!("Implementation in Week 23")
    }
}
```

**Pest Grammar Preview:**
```pest
// Top-level rule
filter = { SOI ~ logical_expr ~ EOI }

// Logical expressions with precedence
logical_expr = { or_expr }
or_expr = { and_expr ~ (or_op ~ and_expr)* }
and_expr = { not_expr ~ (and_op ~ not_expr)* }
not_expr = { not_op? ~ primary_expr }

// Operators (case-insensitive)
or_op = @{ ^"or" | "||" }
and_op = @{ ^"and" | "&&" }
not_op = @{ ^"not" | "!" }

// Primary expressions
primary_expr = { grouped_expr | comparison | null_check }
grouped_expr = { "(" ~ logical_expr ~ ")" }

// Comparisons
comparison = {
    field ~ between_op ~ number ~ number |
    field ~ string_op ~ string |
    field ~ array_op ~ array |
    field ~ comp_op ~ value
}

// Complete grammar in FILTERING_SYNTAX.md
```

### 13.2 Evaluator Implementation Details

The evaluator uses a recursive tree-walk algorithm for simplicity and debuggability. Each AST node type has a dedicated evaluation function.

**Evaluation Context:**
```rust
/// Context for filter evaluation.
///
/// Holds configuration and statistics for a single evaluation.
pub struct EvalContext<'a> {
    /// The metadata map being evaluated against
    pub metadata: &'a HashMap<String, MetadataValue>,

    /// Policy for handling unknown fields
    pub unknown_field_policy: UnknownFieldPolicy,

    /// Statistics (optional, for debugging)
    pub stats: Option<&'a mut EvalStats>,
}

/// Statistics collected during evaluation.
pub struct EvalStats {
    /// Number of nodes evaluated
    pub nodes_evaluated: usize,

    /// Number of short-circuit exits
    pub short_circuits: usize,

    /// Maximum recursion depth reached
    pub max_depth: usize,

    /// Time spent in evaluation (if timing enabled)
    pub eval_time_ns: Option<u64>,
}
```

**Evaluation Function Signatures:**
```rust
/// Main evaluation entry point.
pub fn evaluate(expr: &FilterExpr, ctx: &EvalContext) -> Result<bool, FilterError>;

/// Comparison evaluation helper.
fn eval_comparison(
    left: &FilterExpr,
    right: &FilterExpr,
    op: ComparisonOp,
    ctx: &EvalContext,
) -> Result<bool, FilterError>;

/// String operation evaluation helper.
fn eval_string_op(
    field: &str,
    pattern: &str,
    op: StringOp,
    ctx: &EvalContext,
) -> Result<bool, FilterError>;

/// Array operation evaluation helper.
fn eval_array_op(
    field_array: &[String],
    pattern_array: &[String],
    op: ArrayOp,
) -> bool;

/// LIKE pattern matching (recursive with memoization).
fn eval_like_pattern(value: &str, pattern: &str) -> bool;
```

### 13.3 Strategy Implementation Details

**Strategy Selection Algorithm:**
```rust
impl FilterStrategy {
    /// Select optimal strategy based on filter and index state.
    ///
    /// # Algorithm
    /// 1. If user specified explicit strategy, use it
    /// 2. Estimate selectivity via sampling
    /// 3. Apply decision matrix:
    ///    - selectivity > 80%: PreFilter
    ///    - selectivity < 5%: PostFilter
    ///    - otherwise: Hybrid
    /// 4. Validate chosen strategy is feasible
    ///
    /// # Returns
    /// Tuple of (selected_strategy, estimated_selectivity)
    pub fn select(
        filter: &FilterExpr,
        index: &HnswIndex,
        hint: Option<FilterStrategy>,
    ) -> (FilterStrategy, f32) {
        // Use hint if provided
        if let Some(strategy) = hint {
            return (strategy, 0.5); // Unknown selectivity when hinted
        }

        // Estimate selectivity
        let selectivity = Self::estimate_selectivity(filter, index);

        // Apply decision matrix
        let strategy = match selectivity {
            s if s > 0.80 => FilterStrategy::PreFilter,
            s if s < 0.05 => FilterStrategy::PostFilter { oversample: 3.0 },
            s => FilterStrategy::Hybrid {
                oversample_min: 1.5,
                oversample_max: (1.0 / s).min(MAX_OVERSAMPLE),
            },
        };

        (strategy, selectivity)
    }

    /// Estimate filter selectivity via random sampling.
    fn estimate_selectivity(filter: &FilterExpr, index: &HnswIndex) -> f32 {
        const SAMPLE_SIZE: usize = 100;

        let sample_ids = index.random_sample(SAMPLE_SIZE);
        let passes = sample_ids.iter()
            .filter(|id| {
                let metadata = index.get_metadata(**id);
                evaluate(filter, &metadata).unwrap_or(false)
            })
            .count();

        (passes as f32 / SAMPLE_SIZE as f32).max(0.01).min(1.0)
    }
}
```

**PreFilter Implementation:**
```rust
fn search_prefilter(
    &self,
    query: &[f32],
    k: usize,
    filter: &FilterExpr,
) -> Result<FilteredSearchResult, FilterError> {
    // Step 1: Build bitset of passing vector IDs
    let mut passing_ids = BitVec::with_capacity(self.len());
    let mut pass_count = 0;

    for id in 0..self.len() {
        let metadata = self.get_metadata(id);
        if evaluate(filter, &metadata)? {
            passing_ids.set(id, true);
            pass_count += 1;
        }
    }

    // Step 2: Run HNSW search with bitset filter
    let candidates = self.search_with_bitset(query, k, &passing_ids)?;

    Ok(FilteredSearchResult {
        results: candidates,
        complete: candidates.len() >= k,
        observed_selectivity: pass_count as f32 / self.len() as f32,
        strategy_used: FilterStrategy::PreFilter,
    })
}
```

**PostFilter Implementation:**
```rust
fn search_postfilter(
    &self,
    query: &[f32],
    k: usize,
    filter: &FilterExpr,
    oversample: f32,
) -> Result<FilteredSearchResult, FilterError> {
    // Step 1: Calculate oversampled ef
    let ef_effective = ((k as f32) * oversample).ceil() as usize;
    let ef_effective = ef_effective.min(EF_CAP).max(k);

    // Step 2: Run standard HNSW search with higher ef
    let candidates = self.search_internal(query, ef_effective)?;

    // Step 3: Filter and collect results
    let mut results = Vec::with_capacity(k);
    let mut evaluated = 0;
    let mut passed = 0;

    for candidate in candidates {
        let metadata = self.get_metadata(candidate.id);
        evaluated += 1;

        if evaluate(filter, &metadata)? {
            results.push(candidate);
            passed += 1;

            if results.len() >= k {
                break;
            }
        }
    }

    Ok(FilteredSearchResult {
        results,
        complete: results.len() >= k,
        observed_selectivity: passed as f32 / evaluated as f32,
        strategy_used: FilterStrategy::PostFilter { oversample },
    })
}
```

### 13.4 WASM Binding Implementation Details

**wasm-bindgen Configuration:**
```rust
// src/wasm/filter.rs

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

/// Parse a filter expression from JavaScript.
///
/// # Arguments
/// * `input` - The filter expression string
///
/// # Returns
/// * `FilterExpr` as JSON-serializable object
///
/// # Throws
/// * `FilterError` as JavaScript exception
#[wasm_bindgen]
pub fn parse_filter_js(input: &str) -> Result<JsValue, JsValue> {
    let expr = parse(input)
        .map_err(|e| JsValue::from_str(&e.to_json()))?;

    JsValue::from_serde(&expr)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Search with filter from JavaScript.
///
/// # Arguments
/// * `index` - The HNSW index handle
/// * `query` - Query vector as Float32Array
/// * `k` - Number of results
/// * `filter` - Filter string or pre-parsed FilterExpr
/// * `options` - Optional search options (strategy, etc.)
///
/// # Returns
/// * `Promise<SearchResult[]>`
#[wasm_bindgen]
pub async fn search_with_filter_js(
    index: &WasmIndex,
    query: Float32Array,
    k: usize,
    filter: JsValue,
    options: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    // Parse filter (string or object)
    let filter_expr = parse_filter_value(filter)?;

    // Parse options
    let options = parse_search_options(options)?;

    // Execute search
    let results = index.inner.search_filtered(
        &query.to_vec(),
        k,
        Some(&filter_expr),
        options.strategy,
    ).map_err(|e| JsValue::from_str(&e.to_json()))?;

    // Convert results to JavaScript
    JsValue::from_serde(&results.results)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
```

**Error Handling for WASM:**
```rust
impl FilterError {
    /// Convert error to JSON for JavaScript exception.
    pub fn to_json(&self) -> String {
        #[derive(Serialize)]
        struct JsError {
            code: String,
            message: String,
            position: Option<usize>,
            suggestion: Option<String>,
        }

        let js_error = match self {
            FilterError::SyntaxError { position, message, suggestion } => JsError {
                code: "E001".into(),
                message: message.clone(),
                position: Some(*position),
                suggestion: suggestion.clone(),
            },
            FilterError::TypeMismatch { field, expected, actual } => JsError {
                code: "E101".into(),
                message: format!("Type mismatch for '{}': expected {}, got {}", field, expected, actual),
                position: None,
                suggestion: None,
            },
            // ... other error types
        };

        serde_json::to_string(&js_error).unwrap_or_else(|_| r#"{"code":"E999","message":"Unknown error"}"#.into())
    }
}
```

### 13.5 Error Propagation Strategy

**Rust Error Handling:**
- All public APIs return `Result<T, FilterError>`
- Errors propagate via `?` operator
- No panics in library code (except for logic bugs via `debug_assert!`)

**WASM Error Handling:**
- Rust `Err(FilterError)` converts to JavaScript `throw new Error(json)`
- JavaScript errors include structured data (code, message, position)
- `Promise.reject()` used for async operations

**Error Recovery:**
```rust
/// Configure error handling behavior.
pub struct ErrorConfig {
    /// Treat unknown fields as NULL (lenient) or error (strict)
    pub unknown_field_policy: UnknownFieldPolicy,

    /// Continue on type errors (lenient) or fail fast (strict)
    pub type_error_policy: TypeErrorPolicy,
}

pub enum UnknownFieldPolicy {
    /// Return error if field doesn't exist in metadata
    Strict,
    /// Treat missing fields as NULL (IS NULL returns true)
    Lenient,
}

pub enum TypeErrorPolicy {
    /// Return error on type mismatch
    Strict,
    /// Return false for type mismatches (no error)
    Lenient,
}
```

---

## Appendices

### Appendix A: Full EBNF Grammar

See `docs/architecture/FILTERING_SYNTAX.md` for complete 38-rule grammar.

### Appendix B: Complete Error Catalog

See `docs/architecture/FILTER_EVALUATOR.md` Section 7 for all 16 error types.

### Appendix C: Strategy Performance Data

See `docs/architecture/FILTER_STRATEGY.md` Section 3 for benchmark data.

### Appendix D: WASM API Reference

See `docs/architecture/FILTERING_WASM_API.md` for complete TypeScript types.

### Appendix E: Test Strategy Details

See `docs/architecture/FILTER_TEST_STRATEGY.md` for property test implementations.

### Appendix F: Related Documents

| Document | Purpose | Status |
|:---------|:--------|:-------|
| FILTERING_SYNTAX.md | Grammar specification | [PROPOSED] |
| FILTER_EVALUATOR.md | Evaluation algorithm | [PROPOSED] |
| FILTER_STRATEGY.md | Strategy selection | [PROPOSED] |
| FILTERING_WASM_API.md | JavaScript API | [PROPOSED] |
| FILTER_TEST_STRATEGY.md | Test plan | [PROPOSED] |
| METADATA_SCHEMA_V1.md | Type definitions | [FROZEN] |

### Appendix G: Glossary

| Term | Definition |
|:-----|:-----------|
| AST | Abstract Syntax Tree |
| Selectivity | Fraction of vectors passing filter |
| Oversample | Factor to multiply k by |
| Short-circuit | Skipping evaluation when result is determined |
| ef_search | HNSW exploration factor |
| P99 | 99th percentile latency |

---

## Document Metadata

| Field | Value |
|:------|:------|
| **Document** | `docs/architecture/FILTERING_API.md` |
| **Version** | 1.0.0 |
| **Status** | [PROPOSED] |
| **Word Count** | ~6,200 |
| **Author** | META_ARCHITECT |
| **Reviewer** | HOSTILE_REVIEWER |
| **Created** | 2025-12-17 |
| **Last Modified** | 2025-12-17 |

---

## Approval

```
+---------------------------------------------------------------------+
|   META_ARCHITECT: FILTERING_API.md (Master Document)                 |
|                                                                      |
|   Consolidates:                                                      |
|   - FILTERING_SYNTAX.md (38 rules)                                   |
|   - FILTER_EVALUATOR.md (27 variants)                                |
|   - FILTER_STRATEGY.md (4 strategies)                                |
|   - FILTERING_WASM_API.md (TypeScript types)                         |
|   - FILTER_TEST_STRATEGY.md (17 invariants, 5 fuzz targets)          |
|                                                                      |
|   Status: PENDING HOSTILE_REVIEWER APPROVAL                          |
|                                                                      |
|   Upon approval: Create GATE_W22_COMPLETE.md                         |
|                                                                      |
+---------------------------------------------------------------------+
```

---

**END OF FILTERING_API.md**

---

*"A complete specification is the foundation of reliable implementation."*
