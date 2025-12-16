# Week 22, Day 2: Filter Evaluator Architecture

**Date:** 2025-12-18
**Sprint:** Week 22 (v0.5.0 Phase)
**Day Theme:** AST Design & Evaluation Algorithm
**Status:** PLANNED

---

## Task W22.2: Filter Evaluator Architecture

**Priority:** CRITICAL (P0)
**Estimated Effort:** 8 hours (3x rule: 2.5h optimistic × 3 = 7.5h + 0.5h buffer)
**Status:** PLANNED
**Depends On:** W22.1 (Query Syntax Design)
**Blocks:** W22.3, W22.4, W22.5

---

### Context

Day 2 designs the filter evaluation engine that will execute parsed queries against metadata. This is the core algorithmic challenge of the filtering system.

**Strategic Importance:**
- AST design determines implementation complexity in Week 23
- Evaluation algorithm affects query performance (<10ms target)
- Memory allocation strategy impacts WASM bundle size

**Reference Documents:**
- `docs/architecture/FILTERING_SYNTAX.md` (from Day 1)
- `docs/schemas/METADATA_SCHEMA_V1.md` (FROZEN)

---

### Objective

Create `docs/architecture/FILTER_EVALUATOR.md` with:
1. Complete AST enum design (15+ variant types)
2. Evaluation algorithm with complexity analysis
3. Short-circuit evaluation strategy
4. Memory allocation strategy
5. Error handling during evaluation

---

### Technical Approach

#### 1. AST Node Types Design

**Target:** 15+ variant types covering all operators

```rust
/// Filter expression Abstract Syntax Tree.
///
/// Each node represents an operation in the filter query.
/// The tree is evaluated recursively against vector metadata.
#[derive(Clone, Debug, PartialEq)]
pub enum FilterExpr {
    // ═══════════════════════════════════════════════════════
    // LITERALS (3 variants)
    // ═══════════════════════════════════════════════════════

    /// String literal: "hello"
    LiteralString(String),

    /// Numeric literal: 42 or 3.14
    LiteralNumber(f64),  // Use f64 for both int/float

    /// Boolean literal: true or false
    LiteralBool(bool),

    /// Array literal: ["a", "b", "c"]
    LiteralArray(Vec<FilterExpr>),

    // ═══════════════════════════════════════════════════════
    // FIELD ACCESS (1 variant)
    // ═══════════════════════════════════════════════════════

    /// Field reference: category, price, tags
    Field(String),

    // ═══════════════════════════════════════════════════════
    // COMPARISON OPERATORS (6 variants)
    // ═══════════════════════════════════════════════════════

    /// Equal: field = value
    Eq(Box<FilterExpr>, Box<FilterExpr>),

    /// Not equal: field != value
    Ne(Box<FilterExpr>, Box<FilterExpr>),

    /// Less than: field < value
    Lt(Box<FilterExpr>, Box<FilterExpr>),

    /// Less than or equal: field <= value
    Le(Box<FilterExpr>, Box<FilterExpr>),

    /// Greater than: field > value
    Gt(Box<FilterExpr>, Box<FilterExpr>),

    /// Greater than or equal: field >= value
    Ge(Box<FilterExpr>, Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════
    // STRING OPERATORS (3 variants)
    // ═══════════════════════════════════════════════════════

    /// Contains substring: field CONTAINS "text"
    Contains(Box<FilterExpr>, Box<FilterExpr>),

    /// Starts with prefix: field STARTS_WITH "Dr."
    StartsWith(Box<FilterExpr>, Box<FilterExpr>),

    /// Ends with suffix: field ENDS_WITH ".com"
    EndsWith(Box<FilterExpr>, Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════
    // ARRAY/SET OPERATORS (2 variants)
    // ═══════════════════════════════════════════════════════

    /// In set: field IN ["a", "b"]
    In(Box<FilterExpr>, Box<FilterExpr>),

    /// Not in set: field NOT IN ["x", "y"]
    NotIn(Box<FilterExpr>, Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════
    // LOGICAL OPERATORS (3 variants)
    // ═══════════════════════════════════════════════════════

    /// Logical AND: expr AND expr
    And(Box<FilterExpr>, Box<FilterExpr>),

    /// Logical OR: expr OR expr
    Or(Box<FilterExpr>, Box<FilterExpr>),

    /// Logical NOT: NOT expr
    Not(Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════
    // NULL CHECKS (2 variants)
    // ═══════════════════════════════════════════════════════

    /// Is null: field IS NULL
    IsNull(Box<FilterExpr>),

    /// Is not null: field IS NOT NULL
    IsNotNull(Box<FilterExpr>),
}
```

**Total Variants:** 20 (exceeds 15+ requirement)

#### 2. Evaluation Algorithm

**Strategy:** Recursive tree walk with short-circuit evaluation

```rust
/// Evaluates a filter expression against vector metadata.
///
/// Returns `true` if the vector passes the filter, `false` otherwise.
/// Returns `Err` if evaluation fails (type mismatch, missing field, etc.)
pub fn evaluate(
    expr: &FilterExpr,
    metadata: &HashMap<String, MetadataValue>,
) -> Result<bool, FilterError> {
    match expr {
        // Literals evaluate to themselves (used in comparisons)
        FilterExpr::LiteralString(_) => Err(FilterError::InvalidExpression),
        FilterExpr::LiteralNumber(_) => Err(FilterError::InvalidExpression),
        FilterExpr::LiteralBool(b) => Ok(*b),
        FilterExpr::LiteralArray(_) => Err(FilterError::InvalidExpression),

        // Field access looks up metadata
        FilterExpr::Field(name) => {
            metadata.get(name)
                .map(|v| v.as_boolean().unwrap_or(false))
                .ok_or_else(|| FilterError::UnknownField(name.clone()))
        }

        // Comparisons
        FilterExpr::Eq(left, right) => evaluate_comparison(left, right, metadata, |a, b| a == b),
        FilterExpr::Ne(left, right) => evaluate_comparison(left, right, metadata, |a, b| a != b),
        FilterExpr::Lt(left, right) => evaluate_comparison(left, right, metadata, |a, b| a < b),
        FilterExpr::Le(left, right) => evaluate_comparison(left, right, metadata, |a, b| a <= b),
        FilterExpr::Gt(left, right) => evaluate_comparison(left, right, metadata, |a, b| a > b),
        FilterExpr::Ge(left, right) => evaluate_comparison(left, right, metadata, |a, b| a >= b),

        // String operations
        FilterExpr::Contains(left, right) => evaluate_string_op(left, right, metadata, |s, p| s.contains(p)),
        FilterExpr::StartsWith(left, right) => evaluate_string_op(left, right, metadata, |s, p| s.starts_with(p)),
        FilterExpr::EndsWith(left, right) => evaluate_string_op(left, right, metadata, |s, p| s.ends_with(p)),

        // Set operations
        FilterExpr::In(field, array) => evaluate_in_op(field, array, metadata, false),
        FilterExpr::NotIn(field, array) => evaluate_in_op(field, array, metadata, true),

        // Logical operations with SHORT-CIRCUIT
        FilterExpr::And(left, right) => {
            // Short-circuit: if left is false, don't evaluate right
            if !evaluate(left, metadata)? {
                return Ok(false);
            }
            evaluate(right, metadata)
        }
        FilterExpr::Or(left, right) => {
            // Short-circuit: if left is true, don't evaluate right
            if evaluate(left, metadata)? {
                return Ok(true);
            }
            evaluate(right, metadata)
        }
        FilterExpr::Not(inner) => Ok(!evaluate(inner, metadata)?),

        // Null checks
        FilterExpr::IsNull(field) => {
            if let FilterExpr::Field(name) = field.as_ref() {
                Ok(!metadata.contains_key(name))
            } else {
                Err(FilterError::InvalidExpression)
            }
        }
        FilterExpr::IsNotNull(field) => {
            if let FilterExpr::Field(name) = field.as_ref() {
                Ok(metadata.contains_key(name))
            } else {
                Err(FilterError::InvalidExpression)
            }
        }
    }
}
```

#### 3. Short-Circuit Evaluation

**AND Short-Circuit:**
```
A AND B AND C
  │
  ├─ If A = false → Return false (don't evaluate B or C)
  ├─ If A = true, B = false → Return false (don't evaluate C)
  └─ If A = true, B = true → Evaluate C
```

**OR Short-Circuit:**
```
A OR B OR C
  │
  ├─ If A = true → Return true (don't evaluate B or C)
  ├─ If A = false, B = true → Return true (don't evaluate C)
  └─ If A = false, B = false → Evaluate C
```

#### 4. Memory Overhead Analysis

| AST Node | Size (bytes) | Notes |
|:---------|:-------------|:------|
| `LiteralString` | 24 + string len | String on heap |
| `LiteralNumber` | 8 | f64 inline |
| `LiteralBool` | 1 | bool inline |
| `LiteralArray` | 24 + elements | Vec on heap |
| `Field` | 24 + name len | String on heap |
| Binary ops (Eq, And, etc.) | 16 | Two Box pointers |
| Unary ops (Not, IsNull) | 8 | One Box pointer |

**Typical Query Size:**
- Simple: `category = "gpu"` → ~80 bytes
- Medium: `price > 100 AND category = "gpu"` → ~200 bytes
- Complex: 3+ clauses → ~500 bytes

**Memory Ceiling:** 10KB per query max (triggers optimization if exceeded)

#### 5. Complexity Analysis

| Operator | Time Complexity | Space Complexity |
|:---------|:----------------|:-----------------|
| `Eq`, `Ne`, `Lt`, etc. | O(1) | O(1) |
| `Contains` | O(n×m) | O(1) |
| `StartsWith`, `EndsWith` | O(m) | O(1) |
| `In`, `NotIn` | O(k) | O(1) |
| `And`, `Or` | O(1) + children | O(1) |
| `Not` | O(1) + child | O(1) |
| `IsNull`, `IsNotNull` | O(1) | O(1) |

Where: n = string length, m = pattern length, k = array size

**Total Query Evaluation:** O(d × c) where d = tree depth, c = max child complexity

---

### Deliverables

1. **`docs/architecture/FILTER_EVALUATOR.md`** containing:
   - Complete AST enum design (20 variants)
   - Evaluation algorithm pseudo-code
   - Short-circuit behavior specification
   - Memory overhead calculations
   - Complexity analysis tables
   - Error types catalog (10+ types)

---

### Acceptance Criteria

**CRITICAL (Must Pass):**
- [ ] Complete AST enum design with 15+ variant types documented
- [ ] Evaluation algorithm pseudo-code with O(n) complexity annotations
- [ ] Short-circuit behavior formally specified (AND/OR early exit)
- [ ] Memory overhead calculated per node type (bytes per variant)

**MAJOR (Should Pass):**
- [ ] Error types cataloged with 10+ distinct error messages
- [ ] Complexity analysis for all operators
- [ ] Unicode string comparison strategy defined
- [ ] 5+ short-circuit test case examples

**BINARY CHECKS:**
- [ ] Total AST overhead for typical query <1KB
- [ ] Memory ceiling defined (10KB per query max)
- [ ] All Rust code examples pass `cargo fmt --check`

**Preliminary Memory Budget:**
- [ ] Estimate AST node size for typical query: <200 bytes
- [ ] Define memory ceiling that triggers design changes: 10KB

---

### Error Types Catalog

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FilterError {
    // Parse errors (from Day 1)
    SyntaxError { position: usize, message: String },

    // Evaluation errors
    UnknownField(String),
    TypeMismatch { expected: &'static str, got: &'static str },
    InvalidComparison { left_type: &'static str, right_type: &'static str },
    InvalidStringOp { field_type: &'static str },
    InvalidArrayOp { value_type: &'static str },

    // Value errors
    IntegerOverflow,
    FloatNotFinite,
    StringTooLong { length: usize, max: usize },
    ArrayTooLong { length: usize, max: usize },

    // Expression errors
    InvalidExpression,
    NestingTooDeep { depth: usize, max: usize },
}
```

---

### Implementation Checklist

- [ ] Create `docs/architecture/FILTER_EVALUATOR.md`
- [ ] Write AST enum design with all 20 variants
- [ ] Document each variant's purpose and usage
- [ ] Write evaluation algorithm pseudo-code
- [ ] Create short-circuit evaluation diagrams
- [ ] Calculate memory overhead per node type
- [ ] Create complexity analysis table
- [ ] Define error types catalog
- [ ] Write 5+ short-circuit test cases
- [ ] Document Unicode handling strategy

---

### Dependencies

**Blocks:**
- W22.3 (Strategy needs evaluator design)
- W22.4 (WASM API needs AST structure)
- W22.5 (Test strategy needs error types)

**Blocked By:**
- W22.1 (Query Syntax Design) - MUST be complete

---

### Verification Method

**Day 2 is COMPLETE when:**

1. `docs/architecture/FILTER_EVALUATOR.md` exists
2. AST enum has 15+ variants documented
3. Evaluation algorithm has complexity annotations
4. Memory overhead table is complete
5. Error catalog has 10+ types

---

### Estimated Timeline

| Phase | Time | Cumulative |
|:------|:-----|:-----------|
| Review W22.1 grammar | 0.5h | 0.5h |
| AST enum design | 2h | 2.5h |
| Evaluation algorithm | 2h | 4.5h |
| Short-circuit specification | 1h | 5.5h |
| Memory analysis | 1h | 6.5h |
| Error types catalog | 0.5h | 7h |
| Documentation polish | 0.5h | 7.5h |
| Buffer | 0.5h | 8h |

---

### Hostile Review Checkpoint

**End of Day 2:** Submit for `/review` with:
- `docs/architecture/FILTER_EVALUATOR.md`

**Expected Review Focus:**
- AST completeness (covers all grammar rules)
- Evaluation correctness (all operators handled)
- Short-circuit behavior (correctly specified)
- Memory budget (within limits)
- Error handling (comprehensive)

---

**Task Owner:** META_ARCHITECT
**Review Required:** HOSTILE_REVIEWER
**Next Task:** W22.3 (Pre-Filter vs Post-Filter Strategy)

---

*"Design the tree well, and the forest will follow."*
