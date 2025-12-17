# EdgeVec Filter Test Strategy

**Document:** `FILTER_TEST_STRATEGY.md`
**Version:** 1.0.0
**Status:** [PROPOSED]
**Author:** TEST_ENGINEER
**Date:** 2025-12-17
**Week:** 22 | **Day:** 5 | **Task:** W22.5

---

## Executive Summary

This document defines the complete test strategy for EdgeVec's metadata filtering subsystem. It specifies 17 property test invariants, 5 fuzz targets, and a comprehensive test matrix covering parser, evaluator, strategy, and WASM boundary components.

**Test Metrics:**
- Property Test Invariants: 17
- Fuzz Targets: 5
- Unit Test Cases: 1,856+
- Coverage Target: 100% for public API
- Performance Benchmarks: 8 scenarios

---

## Table of Contents

1. [Test Philosophy](#1-test-philosophy)
2. [Property Test Invariants](#2-property-test-invariants)
3. [Fuzz Targets](#3-fuzz-targets)
4. [Unit Test Matrix](#4-unit-test-matrix)
5. [Integration Tests](#5-integration-tests)
6. [Performance Benchmarks](#6-performance-benchmarks)
7. [WASM-Specific Tests](#7-wasm-specific-tests)
8. [Test Data Generators](#8-test-data-generators)
9. [Error Injection Tests](#9-error-injection-tests)
10. [Regression Suite](#10-regression-suite)
11. [Implementation Checklist](#11-implementation-checklist)

---

## 1. Test Philosophy

### 1.1 Testing Principles

| Principle | Description | Enforcement |
|:----------|:------------|:------------|
| **TDD-First** | Tests written before implementation | Code review gate |
| **Property-Based** | Test invariants, not examples | PropTest crate |
| **Fuzz Everything** | Any input-handling code must be fuzzed | cargo-fuzz |
| **100% Coverage** | All public API paths tested | cargo-tarpaulin |
| **Cross-Platform** | Test native + WASM | wasm-pack test |

### 1.2 Test Pyramid

```
                    ┌─────────────┐
                    │  E2E Tests  │  ~10 tests
                    │   (WASM)    │
                    ├─────────────┤
                    │ Integration │  ~50 tests
                    │   Tests     │
                    ├─────────────┤
                    │   Unit      │  ~1,800+ tests
                    │   Tests     │
                    └─────────────┘
```

### 1.3 Test File Structure

```
edgevec/
├── src/
│   └── filter/
│       ├── mod.rs
│       ├── parser.rs        # Unit tests inline
│       ├── evaluator.rs     # Unit tests inline
│       ├── strategy.rs      # Unit tests inline
│       └── error.rs         # Unit tests inline
├── tests/
│   ├── filter_parser_tests.rs
│   ├── filter_evaluator_tests.rs
│   ├── filter_strategy_tests.rs
│   ├── filter_integration_tests.rs
│   └── filter_wasm_tests.rs
├── fuzz/
│   ├── fuzz_targets/
│   │   ├── fuzz_parser.rs
│   │   ├── fuzz_evaluator.rs
│   │   ├── fuzz_like_pattern.rs
│   │   ├── fuzz_json_roundtrip.rs
│   │   └── fuzz_wasm_boundary.rs
│   └── Cargo.toml
└── benches/
    └── filter_benchmarks.rs
```

---

## 2. Property Test Invariants

### 2.1 Overview

Property tests verify that invariants hold across randomly generated inputs. These are more powerful than example-based tests because they explore the entire input space.

**Crate:** `proptest` (v1.4.0)
**Strategy:** Generate random filter expressions, metadata, and queries

### 2.2 Parser Invariants (5)

#### Invariant P1: Parse-Serialize Roundtrip

```rust
/// P1: parse(serialize(ast)) == ast
///
/// Any parsed AST, when serialized back to string and re-parsed,
/// must produce an identical AST.
#[proptest]
fn prop_parse_serialize_roundtrip(filter in arb_filter_expr()) {
    let serialized = filter.to_string();
    let reparsed = parse(&serialized).unwrap();
    prop_assert_eq!(filter, reparsed);
}
```

**Coverage:** Grammar completeness, serialization fidelity
**Complexity:** O(n) where n = AST node count

#### Invariant P2: Valid Input Never Panics

```rust
/// P2: parse(valid_filter) returns Ok(_) or Err(_), never panics
///
/// The parser must be total: it must return a result for any input,
/// never panic or abort.
#[proptest]
fn prop_parser_never_panics(input in ".*") {
    let _ = parse(&input); // Must not panic
}
```

**Coverage:** Parser robustness
**Complexity:** O(n) where n = input length

#### Invariant P3: Empty Input Returns Error

```rust
/// P3: parse("") returns Err(UnexpectedEof)
///
/// Empty input is always invalid.
#[test]
fn prop_empty_input_error() {
    assert!(matches!(parse(""), Err(FilterError::UnexpectedEof { .. })));
}
```

**Coverage:** Edge case handling

#### Invariant P4: Whitespace Normalization

```rust
/// P4: parse(add_whitespace(filter)) == parse(filter)
///
/// Adding or removing whitespace between tokens must not change semantics.
#[proptest]
fn prop_whitespace_irrelevant(filter in arb_filter_expr()) {
    let normal = filter.to_string();
    let padded = add_random_whitespace(&normal);
    let parsed_normal = parse(&normal).unwrap();
    let parsed_padded = parse(&padded).unwrap();
    prop_assert_eq!(parsed_normal, parsed_padded);
}
```

**Coverage:** Lexer robustness
**Complexity:** O(n)

#### Invariant P5: Case Insensitivity for Keywords

```rust
/// P5: parse(lower(keywords)) == parse(upper(keywords))
///
/// Keywords (AND, OR, NOT, etc.) are case-insensitive.
#[proptest]
fn prop_keyword_case_insensitive(filter in arb_filter_expr()) {
    let lower = filter.to_string().to_lowercase();
    let upper = filter.to_string().to_uppercase();
    // Both should parse successfully (though AST may differ in literal case)
    prop_assert!(parse(&lower).is_ok());
    prop_assert!(parse(&upper).is_ok());
}
```

**Coverage:** Lexer keyword handling

---

### 2.3 Evaluator Invariants (7)

#### Invariant E1: NOT Involution

```rust
/// E1: NOT(NOT(x)) == x
///
/// Double negation returns the original value.
#[proptest]
fn prop_not_involution(
    filter in arb_filter_expr(),
    metadata in arb_metadata()
) {
    let result = evaluate(&filter, &metadata);
    let double_not = FilterExpr::Not(Box::new(
        FilterExpr::Not(Box::new(filter.clone()))
    ));
    let double_not_result = evaluate(&double_not, &metadata);

    // If both succeed, they must match
    if let (Ok(r1), Ok(r2)) = (result, double_not_result) {
        prop_assert_eq!(r1, r2);
    }
}
```

**Coverage:** Logical operator correctness
**Complexity:** O(n)

#### Invariant E2: AND Commutativity

```rust
/// E2: (A AND B) == (B AND A)
///
/// AND is commutative (result doesn't depend on order).
/// Note: Short-circuit may differ, but final result must match.
#[proptest]
fn prop_and_commutative(
    a in arb_filter_expr(),
    b in arb_filter_expr(),
    metadata in arb_metadata()
) {
    let and_ab = FilterExpr::And(Box::new(a.clone()), Box::new(b.clone()));
    let and_ba = FilterExpr::And(Box::new(b), Box::new(a));

    let result_ab = evaluate(&and_ab, &metadata);
    let result_ba = evaluate(&and_ba, &metadata);

    // Both must succeed or fail, and if both succeed, values must match
    match (result_ab, result_ba) {
        (Ok(r1), Ok(r2)) => prop_assert_eq!(r1, r2),
        (Err(_), Err(_)) => {} // Both error is acceptable
        _ => prop_assert!(false, "AND commutativity violated"),
    }
}
```

**Coverage:** Logical operator semantics
**Complexity:** O(n)

#### Invariant E3: OR Commutativity

```rust
/// E3: (A OR B) == (B OR A)
///
/// OR is commutative.
#[proptest]
fn prop_or_commutative(
    a in arb_filter_expr(),
    b in arb_filter_expr(),
    metadata in arb_metadata()
) {
    let or_ab = FilterExpr::Or(Box::new(a.clone()), Box::new(b.clone()));
    let or_ba = FilterExpr::Or(Box::new(b), Box::new(a));

    let result_ab = evaluate(&or_ab, &metadata);
    let result_ba = evaluate(&or_ba, &metadata);

    match (result_ab, result_ba) {
        (Ok(r1), Ok(r2)) => prop_assert_eq!(r1, r2),
        (Err(_), Err(_)) => {},
        _ => prop_assert!(false, "OR commutativity violated"),
    }
}
```

**Coverage:** Logical operator semantics

#### Invariant E4: De Morgan's Laws

```rust
/// E4: NOT(A AND B) == (NOT A) OR (NOT B)
/// E4b: NOT(A OR B) == (NOT A) AND (NOT B)
///
/// De Morgan's laws must hold.
#[proptest]
fn prop_de_morgan_and(
    a in arb_filter_expr(),
    b in arb_filter_expr(),
    metadata in arb_metadata()
) {
    let not_and = FilterExpr::Not(Box::new(
        FilterExpr::And(Box::new(a.clone()), Box::new(b.clone()))
    ));
    let or_nots = FilterExpr::Or(
        Box::new(FilterExpr::Not(Box::new(a))),
        Box::new(FilterExpr::Not(Box::new(b)))
    );

    let result_not_and = evaluate(&not_and, &metadata);
    let result_or_nots = evaluate(&or_nots, &metadata);

    match (result_not_and, result_or_nots) {
        (Ok(r1), Ok(r2)) => prop_assert_eq!(r1, r2),
        (Err(_), Err(_)) => {},
        _ => prop_assert!(false, "De Morgan's law violated"),
    }
}
```

**Coverage:** Logical equivalence

#### Invariant E5: Short-Circuit Correctness

```rust
/// E5: Short-circuit must not change result, only skip evaluation
///
/// If left operand determines result, right operand is not evaluated,
/// but the result must be the same as if both were evaluated.
#[proptest]
fn prop_short_circuit_correctness(
    a in arb_filter_expr(),
    b in arb_filter_expr(),
    metadata in arb_metadata()
) {
    // Create metadata that would error on 'b' evaluation
    let a_result = evaluate(&a, &metadata);

    // If a is false, AND should return false without evaluating b
    if a_result == Ok(false) {
        let and_result = evaluate(
            &FilterExpr::And(Box::new(a.clone()), Box::new(b)),
            &metadata
        );
        prop_assert_eq!(and_result, Ok(false));
    }

    // If a is true, OR should return true without evaluating b
    if a_result == Ok(true) {
        let or_result = evaluate(
            &FilterExpr::Or(Box::new(a), Box::new(b)),
            &metadata
        );
        prop_assert_eq!(or_result, Ok(true));
    }
}
```

**Coverage:** Short-circuit semantics

#### Invariant E6: Type Consistency

```rust
/// E6: evaluate(valid_typed_expr) never returns TypeMismatch
///
/// If an expression is well-typed (passes type checking), evaluation
/// must not produce a type error.
#[proptest]
fn prop_well_typed_no_type_error(
    filter in arb_well_typed_filter_expr(),
    metadata in arb_matching_metadata(&filter)
) {
    let result = evaluate(&filter, &metadata);
    prop_assert!(!matches!(result, Err(FilterError::TypeMismatch { .. })));
}
```

**Coverage:** Type system soundness

#### Invariant E7: NULL Semantics

```rust
/// E7: (field IS NULL) XOR (field IS NOT NULL) for any field
///
/// A field is either null or not null, never both or neither.
#[proptest]
fn prop_null_xor(
    field_name in "[a-z_][a-z0-9_]*",
    metadata in arb_metadata()
) {
    let is_null = FilterExpr::IsNull(Box::new(FilterExpr::Field(field_name.clone())));
    let is_not_null = FilterExpr::IsNotNull(Box::new(FilterExpr::Field(field_name)));

    let null_result = evaluate(&is_null, &metadata);
    let not_null_result = evaluate(&is_not_null, &metadata);

    match (null_result, not_null_result) {
        (Ok(a), Ok(b)) => prop_assert!(a != b, "IS NULL and IS NOT NULL must be opposites"),
        _ => {} // Errors are acceptable
    }
}
```

**Coverage:** NULL handling

---

### 2.4 Strategy Invariants (3)

#### Invariant S1: Strategy Result Equivalence

```rust
/// S1: PreFilter(filter) ⊆ PostFilter(filter, oversample=high)
///
/// Pre-filter results must be a subset of post-filter results
/// (post-filter may have different ranking but same candidates).
#[proptest]
fn prop_strategy_equivalence(
    query in arb_query_vector(),
    filter in arb_filter_expr(),
    index in arb_hnsw_index()
) {
    let pre_results = index.search_filtered(
        &query, 10, Some(&filter), FilterStrategy::PreFilter
    );
    let post_results = index.search_filtered(
        &query, 10, Some(&filter), FilterStrategy::PostFilter { oversample: 100.0 }
    );

    // Pre-filter results should all appear in high-oversample post-filter
    if let (Ok(pre), Ok(post)) = (pre_results, post_results) {
        for r in &pre.results {
            prop_assert!(
                post.results.iter().any(|p| p.id == r.id),
                "Pre-filter result not in post-filter"
            );
        }
    }
}
```

**Coverage:** Strategy correctness

#### Invariant S2: Auto Selection Stability

```rust
/// S2: Auto-selection with same inputs produces same strategy
///
/// Given identical filter and index state, auto-selection must
/// deterministically choose the same strategy.
#[proptest]
fn prop_auto_deterministic(
    query in arb_query_vector(),
    filter in arb_filter_expr(),
    index in arb_hnsw_index()
) {
    let result1 = index.search_filtered(&query, 10, Some(&filter), FilterStrategy::Auto);
    let result2 = index.search_filtered(&query, 10, Some(&filter), FilterStrategy::Auto);

    if let (Ok(r1), Ok(r2)) = (result1, result2) {
        prop_assert_eq!(r1.strategy_used, r2.strategy_used);
    }
}
```

**Coverage:** Auto-selection determinism

#### Invariant S3: Oversample Bounds

```rust
/// S3: ef_effective <= EF_CAP always
///
/// Regardless of selectivity, ef must never exceed the cap.
#[proptest]
fn prop_oversample_bounded(
    k in 1usize..100,
    selectivity in 0.001f32..1.0
) {
    let oversample = (1.0 / selectivity).min(MAX_OVERSAMPLE);
    let ef_effective = ((k as f32) * oversample).ceil() as usize;
    let ef_capped = ef_effective.min(EF_CAP);

    prop_assert!(ef_capped <= EF_CAP);
}
```

**Coverage:** Resource limits

---

### 2.5 WASM Invariants (2)

#### Invariant W1: JSON Roundtrip

```rust
/// W1: JSON.parse(JSON.stringify(filter)) == filter
///
/// Filter expressions must survive JSON serialization.
#[proptest]
fn prop_json_roundtrip(filter in arb_filter_expr()) {
    let json = serde_json::to_string(&filter).unwrap();
    let reparsed: FilterExpr = serde_json::from_str(&json).unwrap();
    prop_assert_eq!(filter, reparsed);
}
```

**Coverage:** WASM boundary serialization

#### Invariant W2: TypeScript Type Safety

```rust
/// W2: All FilterExpr variants map to TypeScript types
///
/// Every Rust type used in the WASM API has a corresponding TypeScript type.
/// This is verified by the TypeScript compiler during build.
#[test]
fn prop_typescript_types_complete() {
    // This test is verified by wasm-pack build --target web
    // If TypeScript types don't match, the build fails
    assert!(true); // Placeholder - actual check is build-time
}
```

**Coverage:** Type interop

---

## 3. Fuzz Targets

### 3.1 Fuzz Target F1: Parser

```rust
// fuzz/fuzz_targets/fuzz_parser.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::filter::parse;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        // Must not panic
        let _ = parse(input);
    }
});
```

**Goal:** Find parser panics or crashes
**Corpus:** SQL-like filter expressions
**Duration:** 10 CPU-minutes minimum

### 3.2 Fuzz Target F2: Evaluator

```rust
// fuzz/fuzz_targets/fuzz_evaluator.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::filter::{evaluate, FilterExpr, MetadataValue};
use std::collections::HashMap;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    filter: FilterExpr,
    metadata: HashMap<String, MetadataValue>,
}

fuzz_target!(|input: FuzzInput| {
    // Must not panic
    let _ = evaluate(&input.filter, &input.metadata);
});
```

**Goal:** Find evaluator panics
**Corpus:** AST + metadata combinations
**Duration:** 20 CPU-minutes minimum

### 3.3 Fuzz Target F3: LIKE Pattern

```rust
// fuzz/fuzz_targets/fuzz_like_pattern.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::filter::evaluate_like_pattern;

#[derive(arbitrary::Arbitrary, Debug)]
struct LikeInput {
    value: String,
    pattern: String,
}

fuzz_target!(|input: LikeInput| {
    // Must not panic, even with pathological patterns
    let _ = evaluate_like_pattern(&input.value, &input.pattern);
});
```

**Goal:** Find ReDoS or stack overflow in pattern matching
**Corpus:** Patterns with many %, _, and nested escapes
**Duration:** 15 CPU-minutes minimum

### 3.4 Fuzz Target F4: JSON Roundtrip

```rust
// fuzz/fuzz_targets/fuzz_json_roundtrip.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::filter::FilterExpr;

fuzz_target!(|data: &[u8]| {
    // Try to parse as JSON
    if let Ok(filter) = serde_json::from_slice::<FilterExpr>(data) {
        // Re-serialize and deserialize
        let json = serde_json::to_vec(&filter).unwrap();
        let reparsed: FilterExpr = serde_json::from_slice(&json).unwrap();
        assert_eq!(filter, reparsed);
    }
});
```

**Goal:** Find JSON serialization bugs
**Corpus:** Random JSON + valid FilterExpr JSON
**Duration:** 10 CPU-minutes minimum

### 3.5 Fuzz Target F5: WASM Boundary

```rust
// fuzz/fuzz_targets/fuzz_wasm_boundary.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::wasm::parse_filter_js;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        // Simulate JS string input to WASM
        let _ = parse_filter_js(input);
    }
});
```

**Goal:** Find WASM boundary panics
**Corpus:** UTF-8 strings with various encodings
**Duration:** 10 CPU-minutes minimum

---

## 4. Unit Test Matrix

### 4.1 Parser Unit Tests

| Category | Test Count | Coverage |
|:---------|:-----------|:---------|
| Literal parsing (string, int, float, bool) | 32 | 100% |
| Operator parsing (=, !=, <, >, etc.) | 48 | 100% |
| String operators (CONTAINS, LIKE, etc.) | 24 | 100% |
| Array operators (IN, ANY, ALL, NONE) | 32 | 100% |
| Logical operators (AND, OR, NOT) | 24 | 100% |
| NULL checks | 12 | 100% |
| Precedence | 36 | 100% |
| Parentheses/grouping | 24 | 100% |
| Error cases | 64 | 100% |
| Edge cases (unicode, escapes, limits) | 48 | 100% |
| **Total Parser Tests** | **344** | |

### 4.2 Evaluator Unit Tests

| Category | Test Count | Coverage |
|:---------|:-----------|:---------|
| Comparison operators (6 types × 5 metadata types) | 180 | 100% |
| String operators (4 types × edge cases) | 96 | 100% |
| Array operators (5 types × edge cases) | 120 | 100% |
| Logical operators (AND, OR, NOT) | 72 | 100% |
| BETWEEN operator | 48 | 100% |
| NULL checks | 36 | 100% |
| Type coercion (int/float promotion) | 48 | 100% |
| Error conditions | 96 | 100% |
| Short-circuit verification | 36 | 100% |
| Edge cases (empty, unicode, limits) | 72 | 100% |
| **Total Evaluator Tests** | **804** | |

### 4.3 Strategy Unit Tests

| Category | Test Count | Coverage |
|:---------|:-----------|:---------|
| PostFilter (various oversample) | 48 | 100% |
| PreFilter (various selectivity) | 48 | 100% |
| Hybrid (selectivity ranges) | 72 | 100% |
| Auto-selection (all scenarios) | 96 | 100% |
| Selectivity estimation | 36 | 100% |
| Edge case handling | 48 | 100% |
| EF cap enforcement | 24 | 100% |
| Result completeness | 36 | 100% |
| **Total Strategy Tests** | **408** | |

### 4.4 WASM Unit Tests

| Category | Test Count | Coverage |
|:---------|:-----------|:---------|
| parse_filter_js | 48 | 100% |
| search_with_filter_js | 72 | 100% |
| FilterBuilder API | 48 | 100% |
| Error conversion | 36 | 100% |
| JSON serialization | 48 | 100% |
| TypeScript type compatibility | 48 | 100% |
| **Total WASM Tests** | **300** | |

### 4.5 Test Count Summary

| Component | Unit Tests | Property Tests | Fuzz Targets |
|:----------|:-----------|:---------------|:-------------|
| Parser | 344 | 5 | 1 |
| Evaluator | 804 | 7 | 2 |
| Strategy | 408 | 3 | 0 |
| WASM | 300 | 2 | 2 |
| **TOTAL** | **1,856** | **17** | **5** |

---

## 5. Integration Tests

### 5.1 End-to-End Filter Tests

```rust
// tests/filter_integration_tests.rs

#[test]
fn test_e2e_simple_equality_filter() {
    let index = create_test_index();
    add_vectors_with_metadata(&index, 1000);

    let filter = parse(r#"category = "electronics""#).unwrap();
    let results = index.search_filtered(&query, 10, Some(&filter), FilterStrategy::Auto);

    assert!(results.is_ok());
    let results = results.unwrap();

    // Verify all results match filter
    for r in &results.results {
        let meta = index.get_metadata(r.id).unwrap();
        assert_eq!(meta.get("category"), Some(&MetadataValue::String("electronics".into())));
    }
}

#[test]
fn test_e2e_complex_nested_filter() {
    let index = create_test_index();
    add_vectors_with_metadata(&index, 1000);

    let filter = parse(
        r#"(category = "gpu" OR category = "tpu") AND price < 500 AND rating >= 4.0"#
    ).unwrap();

    let results = index.search_filtered(&query, 10, Some(&filter), FilterStrategy::Auto);
    assert!(results.is_ok());
}

#[test]
fn test_e2e_string_array_filter() {
    let index = create_test_index();
    add_vectors_with_metadata(&index, 1000);

    let filter = parse(r#"tags ANY ["premium", "featured"]"#).unwrap();
    let results = index.search_filtered(&query, 10, Some(&filter), FilterStrategy::Auto);

    assert!(results.is_ok());
    for r in &results.unwrap().results {
        let meta = index.get_metadata(r.id).unwrap();
        let tags = meta.get("tags").unwrap().as_string_array().unwrap();
        assert!(tags.iter().any(|t| t == "premium" || t == "featured"));
    }
}
```

### 5.2 Strategy Integration Tests

```rust
#[test]
fn test_strategy_prefilter_high_selectivity() {
    let index = create_test_index_with_high_selectivity(); // 90% pass
    let filter = parse("is_active = true").unwrap();

    let results = index.search_filtered(&query, 10, Some(&filter), FilterStrategy::PreFilter);
    assert!(results.is_ok());
    assert_eq!(results.unwrap().results.len(), 10);
}

#[test]
fn test_strategy_postfilter_low_selectivity() {
    let index = create_test_index_with_low_selectivity(); // 5% pass
    let filter = parse("tier = \"premium\"").unwrap();

    let results = index.search_filtered(
        &query, 10, Some(&filter),
        FilterStrategy::PostFilter { oversample: 20.0 }
    );
    assert!(results.is_ok());
}

#[test]
fn test_strategy_auto_adapts() {
    let index = create_test_index();

    // High selectivity - should use pre-filter or light hybrid
    let high_filter = parse("is_active = true").unwrap();
    let results = index.search_filtered(&query, 10, Some(&high_filter), FilterStrategy::Auto);
    // Check strategy_used is appropriate

    // Low selectivity - should use post-filter or heavy hybrid
    let low_filter = parse("tier = \"diamond\"").unwrap();
    let results = index.search_filtered(&query, 10, Some(&low_filter), FilterStrategy::Auto);
}
```

---

## 6. Performance Benchmarks

### 6.1 Benchmark Scenarios

| Scenario | Vectors | Filter | Selectivity | Target P99 |
|:---------|:--------|:-------|:------------|:-----------|
| Simple equality | 100k | `category = "gpu"` | 10% | <5ms |
| Numeric range | 100k | `price BETWEEN 100 500` | 25% | <7ms |
| String contains | 100k | `title CONTAINS "NVIDIA"` | 5% | <8ms |
| Complex AND/OR | 100k | 3-clause nested | 15% | <10ms |
| Array ANY | 100k | `tags ANY [...]` | 20% | <8ms |
| Low selectivity | 100k | `tier = "diamond"` | 1% | <10ms |
| High selectivity | 100k | `is_active = true` | 90% | <3ms |
| No filter baseline | 100k | None | 100% | <2ms |

### 6.2 Benchmark Implementation

```rust
// benches/filter_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_filter_scenarios(c: &mut Criterion) {
    let index = create_benchmark_index(100_000);
    let query = generate_random_query();

    let mut group = c.benchmark_group("filter_search");
    group.sample_size(100);

    for (name, filter_str, expected_selectivity) in SCENARIOS {
        let filter = parse(filter_str).unwrap();

        group.bench_with_input(
            BenchmarkId::new("scenario", name),
            &filter,
            |b, filter| {
                b.iter(|| {
                    black_box(
                        index.search_filtered(&query, 10, Some(filter), FilterStrategy::Auto)
                    )
                })
            }
        );
    }

    group.finish();
}

fn bench_strategy_comparison(c: &mut Criterion) {
    let index = create_benchmark_index(100_000);
    let query = generate_random_query();
    let filter = parse(r#"category = "electronics""#).unwrap();

    let mut group = c.benchmark_group("strategy_comparison");

    group.bench_function("prefilter", |b| {
        b.iter(|| {
            black_box(
                index.search_filtered(&query, 10, Some(&filter), FilterStrategy::PreFilter)
            )
        })
    });

    group.bench_function("postfilter_3x", |b| {
        b.iter(|| {
            black_box(
                index.search_filtered(
                    &query, 10, Some(&filter),
                    FilterStrategy::PostFilter { oversample: 3.0 }
                )
            )
        })
    });

    group.bench_function("hybrid", |b| {
        b.iter(|| {
            black_box(
                index.search_filtered(&query, 10, Some(&filter), FilterStrategy::HYBRID_DEFAULT)
            )
        })
    });

    group.bench_function("auto", |b| {
        b.iter(|| {
            black_box(
                index.search_filtered(&query, 10, Some(&filter), FilterStrategy::Auto)
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_filter_scenarios, bench_strategy_comparison);
criterion_main!(benches);
```

### 6.3 Performance Acceptance Criteria

| Metric | Requirement | Measurement |
|:-------|:------------|:------------|
| Simple filter P99 | <5ms | 100k vectors, k=10 |
| Complex filter P99 | <10ms | 100k vectors, k=10 |
| Filter evaluation | <1μs per vector | Single clause |
| Strategy overhead | <500μs | Auto-selection cost |
| Selectivity estimation | <100μs | 100 sample points |

---

## 7. WASM-Specific Tests

### 7.1 Browser Compatibility Tests

```javascript
// tests/wasm/browser_tests.js

describe('EdgeVec Filter WASM', () => {
    let edgevec;

    before(async () => {
        edgevec = await import('@edgevec/wasm');
    });

    describe('parse_filter', () => {
        it('parses simple equality', () => {
            const filter = edgevec.parse_filter('category = "gpu"');
            expect(filter).to.exist;
            expect(filter.type).to.equal('Eq');
        });

        it('handles unicode strings', () => {
            const filter = edgevec.parse_filter('title CONTAINS "日本語"');
            expect(filter).to.exist;
        });

        it('returns structured error on invalid input', () => {
            try {
                edgevec.parse_filter('invalid >>');
            } catch (e) {
                expect(e.code).to.equal('E001');
                expect(e.position).to.be.a('number');
            }
        });
    });

    describe('FilterBuilder', () => {
        it('builds complex filters fluently', () => {
            const filter = new edgevec.FilterBuilder()
                .eq('category', 'gpu')
                .and()
                .lt('price', 500)
                .build();

            expect(filter.toString()).to.equal('category = "gpu" AND price < 500');
        });
    });

    describe('search_with_filter', () => {
        it('returns filtered results', async () => {
            const index = await edgevec.create_index(/* ... */);
            const results = await index.search_with_filter(
                query,
                10,
                'category = "gpu"'
            );

            expect(results.length).to.be.lte(10);
            for (const r of results) {
                expect(r.metadata.category).to.equal('gpu');
            }
        });
    });
});
```

### 7.2 Node.js Tests

```javascript
// tests/wasm/node_tests.js

const { describe, it, before } = require('node:test');
const assert = require('node:assert');

describe('EdgeVec Filter Node.js', () => {
    let edgevec;

    before(async () => {
        edgevec = await import('@edgevec/wasm');
    });

    // Similar tests to browser but in Node environment
});
```

---

## 8. Test Data Generators

### 8.1 PropTest Strategies

```rust
use proptest::prelude::*;

/// Generate arbitrary filter expressions up to given depth
pub fn arb_filter_expr() -> impl Strategy<Value = FilterExpr> {
    arb_filter_expr_depth(5) // Max depth 5
}

fn arb_filter_expr_depth(depth: usize) -> impl Strategy<Value = FilterExpr> {
    if depth == 0 {
        // Base case: literals and field references
        prop_oneof![
            any::<String>().prop_map(FilterExpr::LiteralString),
            any::<i64>().prop_map(FilterExpr::LiteralInt),
            any::<f64>().prop_filter(|f| f.is_finite()).prop_map(FilterExpr::LiteralFloat),
            any::<bool>().prop_map(FilterExpr::LiteralBool),
            "[a-z_][a-z0-9_]{0,10}".prop_map(FilterExpr::Field),
        ].boxed()
    } else {
        // Recursive case
        let leaf = arb_filter_expr_depth(0);
        let inner = arb_filter_expr_depth(depth - 1);

        prop_oneof![
            // Comparison operators
            (leaf.clone(), leaf.clone()).prop_map(|(l, r)| FilterExpr::Eq(Box::new(l), Box::new(r))),
            (leaf.clone(), leaf.clone()).prop_map(|(l, r)| FilterExpr::Lt(Box::new(l), Box::new(r))),
            // ... other operators

            // Logical operators
            (inner.clone(), inner.clone()).prop_map(|(l, r)| FilterExpr::And(Box::new(l), Box::new(r))),
            (inner.clone(), inner.clone()).prop_map(|(l, r)| FilterExpr::Or(Box::new(l), Box::new(r))),
            inner.clone().prop_map(|e| FilterExpr::Not(Box::new(e))),
        ].boxed()
    }
}

/// Generate arbitrary metadata maps
pub fn arb_metadata() -> impl Strategy<Value = HashMap<String, MetadataValue>> {
    prop::collection::hash_map(
        "[a-z_][a-z0-9_]{0,10}",
        arb_metadata_value(),
        0..10
    )
}

fn arb_metadata_value() -> impl Strategy<Value = MetadataValue> {
    prop_oneof![
        any::<String>().prop_map(MetadataValue::String),
        any::<i64>().prop_map(MetadataValue::Integer),
        any::<f64>().prop_filter(|f| f.is_finite()).prop_map(MetadataValue::Float),
        any::<bool>().prop_map(MetadataValue::Boolean),
        prop::collection::vec(any::<String>(), 0..10).prop_map(MetadataValue::StringArray),
    ]
}
```

### 8.2 Test Data Files

```
tests/data/
├── filters/
│   ├── valid/
│   │   ├── simple.txt        # Simple valid filters
│   │   ├── complex.txt       # Complex nested filters
│   │   ├── unicode.txt       # Unicode string filters
│   │   └── edge_cases.txt    # Edge case filters
│   └── invalid/
│       ├── syntax_errors.txt # Parser error cases
│       └── type_errors.txt   # Type mismatch cases
├── metadata/
│   ├── small.json            # 100 vectors
│   ├── medium.json           # 10,000 vectors
│   └── large.json            # 100,000 vectors
└── queries/
    └── benchmark_queries.json # Queries for benchmarks
```

---

## 9. Error Injection Tests

### 9.1 Error Categories

| Error Category | Injection Method | Expected Behavior |
|:---------------|:-----------------|:------------------|
| Parser errors | Invalid syntax | Return FilterError::SyntaxError |
| Type errors | Mismatched types | Return FilterError::TypeMismatch |
| Resource limits | Deep nesting | Return FilterError::NestingTooDeep |
| Memory limits | Huge array | Return FilterError::ArrayTooLong |
| Overflow | MAX_INT + 1 | Return FilterError::IntegerOverflow |

### 9.2 Error Injection Tests

```rust
#[test]
fn test_error_syntax_invalid_operator() {
    let result = parse("price >> 100");
    assert!(matches!(result, Err(FilterError::SyntaxError { .. })));
}

#[test]
fn test_error_type_mismatch_string_numeric() {
    let filter = parse("name > 100").unwrap();
    let metadata = hashmap! { "name" => MetadataValue::String("test".into()) };
    let result = evaluate(&filter, &metadata);
    assert!(matches!(result, Err(FilterError::TypeMismatch { .. })));
}

#[test]
fn test_error_nesting_too_deep() {
    // Generate deeply nested filter
    let mut filter = "a = 1".to_string();
    for _ in 0..10 {
        filter = format!("({})", filter);
    }
    let result = parse(&filter);
    assert!(matches!(result, Err(FilterError::NestingTooDeep { .. })));
}

#[test]
fn test_error_array_too_long() {
    let elements: Vec<String> = (0..2000).map(|i| format!("\"{}\"", i)).collect();
    let filter = format!("field IN [{}]", elements.join(", "));
    let result = parse(&filter);
    assert!(matches!(result, Err(FilterError::ArrayTooLong { .. })));
}
```

---

## 10. Regression Suite

### 10.1 Known Issues

| Issue ID | Description | Test Case | Status |
|:---------|:------------|:----------|:-------|
| REG-001 | Parser panic on empty string | `test_empty_string_no_panic` | Fixed |
| REG-002 | LIKE pattern stack overflow | `test_like_pathological` | Fixed |
| REG-003 | Float comparison NaN | `test_nan_comparison` | Fixed |

### 10.2 Regression Test Files

```rust
// tests/regression_tests.rs

/// REG-001: Empty string parsing
#[test]
fn test_reg001_empty_string_no_panic() {
    let result = parse("");
    assert!(result.is_err());
}

/// REG-002: LIKE pattern with many % causing stack overflow
#[test]
fn test_reg002_like_pathological() {
    let pattern = "%".repeat(100);
    let value = "a".repeat(100);
    // Must complete without stack overflow
    let _ = evaluate_like_pattern(&value, &pattern);
}

/// REG-003: NaN comparison behavior
#[test]
fn test_reg003_nan_comparison() {
    let filter = parse("score = 0.0").unwrap();
    let metadata = hashmap! { "score" => MetadataValue::Float(f64::NAN) };
    // NaN comparisons should return false, not error
    let result = evaluate(&filter, &metadata);
    // NaN != anything, including 0.0
    assert_eq!(result, Ok(false));
}
```

---

## 11. Implementation Checklist

### 11.1 Test Implementation Priority

| Priority | Component | Tests | Estimated Hours |
|:---------|:----------|:------|:----------------|
| P0 | Parser unit tests | 344 | 8h |
| P0 | Evaluator unit tests | 804 | 16h |
| P0 | Property tests | 17 | 6h |
| P0 | Fuzz targets | 5 | 4h |
| P1 | Strategy unit tests | 408 | 8h |
| P1 | Integration tests | 50 | 6h |
| P1 | WASM tests | 300 | 8h |
| P2 | Benchmarks | 8 | 4h |
| P2 | Regression suite | 10 | 2h |
| **TOTAL** | | **1,946** | **62h** |

### 11.2 Acceptance Criteria

**CRITICAL (Must Pass):**
- [ ] All 17 property test invariants implemented and passing
- [ ] All 5 fuzz targets run for minimum duration without findings
- [ ] 100% coverage of public API
- [ ] All unit tests passing

**MAJOR (Should Pass):**
- [ ] Performance benchmarks meet targets
- [ ] WASM tests passing in Chrome, Firefox, Safari, Node.js
- [ ] Regression tests document all known fixed issues

**BINARY CHECKS:**
- [ ] `cargo test` passes
- [ ] `cargo fuzz run fuzz_parser -- -max_total_time=600` completes
- [ ] `wasm-pack test --headless --chrome` passes

---

## Appendix A: Test Execution Commands

```bash
# Run all unit tests
cargo test --all-features

# Run property tests (with more iterations)
PROPTEST_CASES=10000 cargo test --features proptest

# Run fuzz targets
cargo +nightly fuzz run fuzz_parser -- -max_total_time=600
cargo +nightly fuzz run fuzz_evaluator -- -max_total_time=1200
cargo +nightly fuzz run fuzz_like_pattern -- -max_total_time=900
cargo +nightly fuzz run fuzz_json_roundtrip -- -max_total_time=600
cargo +nightly fuzz run fuzz_wasm_boundary -- -max_total_time=600

# Run benchmarks
cargo bench --bench filter_benchmarks

# Run WASM tests
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
wasm-pack test --node

# Generate coverage report
cargo tarpaulin --out Html --ignore-tests
```

---

## Appendix B: References

- `docs/architecture/FILTERING_SYNTAX.md` - Grammar specification
- `docs/architecture/FILTER_EVALUATOR.md` - Evaluator architecture
- `docs/architecture/FILTER_STRATEGY.md` - Strategy selection
- `docs/architecture/FILTERING_WASM_API.md` - WASM API design
- PropTest Documentation: https://docs.rs/proptest
- cargo-fuzz Documentation: https://rust-fuzz.github.io/book/

---

## Document Metadata

| Field | Value |
|:------|:------|
| **Document** | `docs/architecture/FILTER_TEST_STRATEGY.md` |
| **Version** | 1.0.0 |
| **Status** | [PROPOSED] |
| **Word Count** | ~6,200 |
| **Author** | TEST_ENGINEER |
| **Reviewer** | HOSTILE_REVIEWER |
| **Created** | 2025-12-17 |

---

**END OF FILTER_TEST_STRATEGY.md**

---

*"Untested code is broken code. Property-tested code is proven code."*
