# Contributing to EdgeVec

**Thank you for your interest in contributing to EdgeVec!**

EdgeVec follows a **"Nvidia Grade"** development protocol where correctness and verification are paramount. This document outlines the strict quality standards all contributions must meet.

---

## Table of Contents

1. [Core Principles](#core-principles)
2. [Forbidden Patterns](#forbidden-patterns)
3. [Required Practices](#required-practices)
4. [Testing Standards](#testing-standards)
5. [Code Review Process](#code-review-process)
6. [Development Workflow](#development-workflow)

---

## Core Principles

### 1. Test-First Development (TDD)

**Every feature begins with tests, not implementation.**

```rust
// ✅ CORRECT: Write the test first
#[test]
fn test_insert_returns_vector_id() {
    let mut index = VectorIndex::new(128);
    let vector = vec![0.0; 128];
    let id = index.insert(&vector).expect("insert failed");
    assert!(id.is_valid());
}

// Then implement the function to pass the test
impl VectorIndex {
    pub fn insert(&mut self, vector: &[f32]) -> Result<VectorId, Error> {
        // Implementation
    }
}
```

### 2. Correctness Over Convenience

**Code must be provably correct before it can be fast.**

- All public APIs must have comprehensive tests
- All algorithms must have property-based tests
- All unsafe code must have documented safety proofs
- All performance claims must have benchmark evidence

### 3. Hostile Default

**Assume your code will break in production. Prove it won't.**

- Design for failure modes
- Handle all error paths explicitly
- No silent failures
- No assumptions about input validity

---

## Forbidden Patterns

### ❌ FORBIDDEN: `unwrap()` in Library Code

**Never use `unwrap()`, `expect()`, or any panic-inducing operation in library code.**

```rust
// ❌ FORBIDDEN
pub fn search(&self, query: &[f32]) -> Vec<Result> {
    let k = query.len();
    self.index.get(k).unwrap()  // PANIC if k is out of bounds!
}

// ✅ CORRECT
pub fn search(&self, query: &[f32]) -> Result<Vec<SearchResult>, Error> {
    let k = query.len();
    self.index.get(k)
        .ok_or(Error::InvalidQueryDimension { 
            expected: self.dim, 
            actual: k 
        })
}
```

**Exceptions:** `unwrap()` is allowed ONLY in:
- Test code (`#[cfg(test)]`)
- Example code (`examples/`)
- Build scripts (`build.rs`)

### ❌ FORBIDDEN: Panic in Public APIs

```rust
// ❌ FORBIDDEN
pub fn insert(&mut self, vector: &[f32]) {
    assert_eq!(vector.len(), self.dim, "Dimension mismatch!");  // PANIC!
}

// ✅ CORRECT
pub fn insert(&mut self, vector: &[f32]) -> Result<VectorId, Error> {
    if vector.len() != self.dim {
        return Err(Error::DimensionMismatch {
            expected: self.dim,
            actual: vector.len(),
        });
    }
    // ...
}
```

### ❌ FORBIDDEN: `unsafe` Without Justification

**Every `unsafe` block must have:**
1. A comment explaining why it's needed
2. A proof of safety
3. A reference to the review that approved it

```rust
// ❌ FORBIDDEN
unsafe {
    *ptr = value;  // No explanation!
}

// ✅ CORRECT
// SAFETY: This unsafe block is required because:
// 1. We need to avoid bounds checks in the search hot path
// 2. The index `i` is guaranteed valid by invariant [I1] (len <= capacity)
// 3. This was reviewed in PR #42 by HOSTILE_REVIEWER
//
// Proof of safety:
// - Line 127 checks `i < self.len` before this block
// - `self.data` is always valid for `self.len` elements (invariant I2)
// - No aliasing: `self.data` is uniquely owned by this struct
unsafe {
    *self.data.get_unchecked_mut(i) = value;
}
```

### ❌ FORBIDDEN: Magic Numbers

```rust
// ❌ FORBIDDEN
if neighbors.len() > 32 {
    // Why 32? Is it configurable? Where did it come from?
}

// ✅ CORRECT
/// Maximum number of neighbors per HNSW node.
/// 
/// Source: Section 5.2 of the HNSW paper (Malkov & Yashunin, 2018)
/// recommends M=16 for most use cases. We use 2*M=32 as the capacity.
pub const MAX_NEIGHBORS: usize = 32;

if neighbors.len() > MAX_NEIGHBORS {
    // Clear intent, documented source
}
```

### ❌ FORBIDDEN: TODO Without Issue Link

```rust
// ❌ FORBIDDEN
// TODO: optimize this later

// ✅ CORRECT
// TODO(issue #123): Optimize with SIMD after performance profiling
```

### ❌ FORBIDDEN: Commented-Out Code

```rust
// ❌ FORBIDDEN
pub fn search(&self, query: &[f32]) -> Result<Vec<Result>, Error> {
    // let old_approach = self.linear_search(query);
    self.hnsw_search(query)
}

// ✅ CORRECT: Delete it. Git remembers.
pub fn search(&self, query: &[f32]) -> Result<Vec<SearchResult>, Error> {
    self.hnsw_search(query)
}
```

---

## Required Practices

### ✅ REQUIRED: Explicit Error Handling

**Use `Result<T, E>` for all fallible operations.**

```rust
// Define clear error types
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("index is empty")]
    EmptyIndex,
    
    #[error("k={k} exceeds vector count={count}")]
    KTooLarge { k: usize, count: usize },
}

// Use Result everywhere
pub fn insert(&mut self, vector: &[f32]) -> Result<VectorId, Error>;
pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>, Error>;
```

### ✅ REQUIRED: Documentation Comments

**Every public item must have a doc comment.**

```rust
/// A hierarchical navigable small world (HNSW) index for approximate nearest neighbor search.
///
/// # Examples
///
/// ```rust
/// use edgevec::HnswIndex;
///
/// let mut index = HnswIndex::new(128); // 128-dimensional vectors
/// index.insert(&vec![0.0; 128])?;
/// let results = index.search(&vec![1.0; 128], 10)?;
/// ```
///
/// # Performance
///
/// - Search: O(log n) average case
/// - Insert: O(log n) average case
/// - Memory: ~82 bytes per vector (including index overhead)
///
/// # Thread Safety
///
/// This type is `Send + Sync` and can be safely shared across threads.
pub struct HnswIndex {
    // ...
}
```

### ✅ REQUIRED: Struct Invariants

**Document invariants for all data structures.**

```rust
/// Vector storage with arena allocation.
///
/// # Invariants
///
/// - [I1] `len <= capacity` (always)
/// - [I2] `data[0..len]` is always valid
/// - [I3] `tombstones.len() == capacity` (bitmap size matches capacity)
/// - [I4] If `tombstones[i] == true`, then `data[i]` is logically deleted
///
/// # Safety
///
/// Violating any invariant is undefined behavior. All public methods maintain invariants.
pub struct VectorStorage {
    data: Vec<Vector>,
    len: usize,
    capacity: usize,
    tombstones: BitVec,
}
```

### ✅ REQUIRED: Test Coverage

**100% coverage for all public APIs.**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Unit test: Empty index returns error
    #[test]
    fn test_search_empty_index() {
        let index = HnswIndex::new(128);
        let query = vec![0.0; 128];
        let result = index.search(&query, 10);
        assert!(matches!(result, Err(Error::EmptyIndex)));
    }

    /// Boundary test: Single vector
    #[test]
    fn test_search_single_vector() {
        let mut index = HnswIndex::new(128);
        index.insert(&vec![1.0; 128]).unwrap();
        let results = index.search(&vec![1.0; 128], 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    /// Scale test: Large index
    #[test]
    fn test_search_100k_vectors() {
        let mut index = HnswIndex::new(128);
        for _ in 0..100_000 {
            index.insert(&random_vector(128)).unwrap();
        }
        let results = index.search(&random_vector(128), 10).unwrap();
        assert_eq!(results.len(), 10);
    }

    /// Property-based test: Idempotence
    #[test]
    fn test_search_is_deterministic() {
        proptest!(|(seed: u64)| {
            let mut index = HnswIndex::with_seed(128, seed);
            for _ in 0..1000 {
                index.insert(&random_vector(128)).unwrap();
            }
            let query = random_vector(128);
            let results1 = index.search(&query, 10).unwrap();
            let results2 = index.search(&query, 10).unwrap();
            prop_assert_eq!(results1, results2);
        });
    }
}
```

---

## Testing Standards

### "Nvidia Grade" Verification

EdgeVec uses a multi-layer test pyramid:

```
         /\
        /E2E\         ← 5 End-to-End tests (browser automation)
       /------\
      /Integr.\      ← 6 Integration tests (persistence + search)
     /----------\
    /  Recall   \    ← 1 Recall test (95% threshold on SIFT dataset)
   /--------------\
  / Property-Based\  ← 8 Property tests (invariants, serialization)
 /------------------\
/   Fuzz + Miri     \ ← 4 Fuzz targets + Miri checks (memory safety)
/--------------------\
|   Unit Tests (200+)|  ← Comprehensive unit coverage
----------------------
```

### Required Test Types

1. **Unit Tests:** Every function has at least one test
2. **Integration Tests:** Cross-component workflows
3. **Property-Based Tests:** Invariants hold for all inputs
4. **Fuzz Tests:** No crashes on adversarial inputs
5. **Recall Tests:** Search returns correct results (>95% recall)
6. **Benchmark Tests:** Performance budgets are met

### Test Naming Convention

```rust
#[test]
fn test_<function>_<scenario>_<expected_behavior>() {
    // test_insert_duplicate_vector_returns_error
    // test_search_empty_index_returns_error
    // test_serialize_roundtrip_preserves_data
}
```

---

## Code Review Process

### Self-Review Checklist

Before submitting a PR, verify:

- [ ] `cargo fmt` passes (formatting)
- [ ] `cargo clippy -- -D warnings` passes (no warnings)
- [ ] `cargo test` passes (all tests)
- [ ] `cargo doc --no-deps` passes (documentation)
- [ ] No `unwrap()` in library code
- [ ] No `TODO` without issue reference
- [ ] All public APIs documented
- [ ] Test coverage for new code
- [ ] Acceptance criteria met (from task plan)

### Hostile Review

All PRs undergo **HOSTILE_REVIEWER** scrutiny:

1. **Correctness:** Does the code do what it claims?
2. **Safety:** Are all invariants maintained?
3. **Performance:** Are performance budgets met?
4. **Testing:** Is verification comprehensive?
5. **Documentation:** Is intent clear?

**Standard:** Maximum scrutiny. Approval requires zero critical issues.

---

## Development Workflow

### 1. Check the Plan

**No code without an approved task.**

```bash
# Verify your task exists in the weekly plan
cat docs/planning/weeks/week_XX/OVERVIEW.md

# Find your task ID (e.g., W3.2)
# Read acceptance criteria
```

### 2. Write Tests First

```bash
# Create test file first
touch tests/hnsw_test.rs

# Write failing tests that define the interface
cargo test  # Should fail
```

### 3. Implement

```bash
# Now write the code to pass the tests
vim src/hnsw.rs
cargo test  # Should pass
```

### 4. Verify Quality

```bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Test
cargo test

# Document
cargo doc --no-deps --open

# Benchmark (if applicable)
cargo bench
```

### 5. Submit for Review

```bash
# Create PR with:
# - Reference to task ID (e.g., "Implements W3.2")
# - Test results
# - Benchmark results (if applicable)
# - Self-review checklist completed
```

---

## Examples

### Good Contribution Example

```rust
/// Inserts a vector into the index.
///
/// # Arguments
///
/// * `vector` - A slice of f32 values representing the vector.
///
/// # Returns
///
/// Returns `Ok(VectorId)` if successful, or `Err(Error)` if:
/// - The vector dimension doesn't match the index dimension
/// - The index is at capacity
///
/// # Examples
///
/// ```rust
/// let mut index = HnswIndex::new(128);
/// let vector = vec![0.0; 128];
/// let id = index.insert(&vector)?;
/// ```
pub fn insert(&mut self, vector: &[f32]) -> Result<VectorId, Error> {
    // Validate dimension
    if vector.len() != self.dim {
        return Err(Error::DimensionMismatch {
            expected: self.dim,
            actual: vector.len(),
        });
    }

    // Check capacity
    if self.len >= self.capacity {
        return Err(Error::AtCapacity {
            capacity: self.capacity,
        });
    }

    // Insert logic...
    let id = VectorId::new(self.len);
    self.storage.push(vector);
    self.len += 1;

    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_valid_vector_succeeds() {
        let mut index = HnswIndex::new(128);
        let vector = vec![0.0; 128];
        let result = index.insert(&vector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_wrong_dimension_fails() {
        let mut index = HnswIndex::new(128);
        let vector = vec![0.0; 64]; // Wrong dimension
        let result = index.insert(&vector);
        assert!(matches!(result, Err(Error::DimensionMismatch { .. })));
    }
}
```

---

## Questions?

If you have questions about these standards:

1. Read the architecture docs: `docs/architecture/`
2. Check existing code for patterns
3. Ask in issues or discussions

**Remember:** Strict standards produce reliable software. Thank you for helping build EdgeVec to "Nvidia Grade" quality! 🚀

---

**Document Version:** 1.0  
**Last Updated:** 2025-12-05  
**Status:** ACTIVE

