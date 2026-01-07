# Week 35 Day 3: cast_possible_truncation Fixes (Part 1)

**Date:** 2026-01-29
**Focus:** Fix high-priority cast_possible_truncation warnings
**Hours:** 2h
**Status:** [ ] PENDING

---

## Context

Clippy lint `clippy::cast_possible_truncation` warns when casting between integer types may lose data (e.g., `usize as u32` on 64-bit systems).

**Priority:** P2 - Potential correctness issues
**Scope:** ~25 warnings (first half)

---

## Tasks

### W35.3a: cast_possible_truncation Part 1 (2h)

**Goal:** Fix first half of cast_possible_truncation warnings.

**Subtasks:**

- [ ] **3.1** Generate full warning list (20min)
  - Run `cargo clippy -- -W clippy::cast_possible_truncation`
  - Export to file for tracking
  - Count total warnings
  - Prioritize by file/severity

- [ ] **3.2** Categorize warnings (20min)
  - **Critical:** Index/size calculations that could overflow
  - **Medium:** Performance counters, metrics
  - **Low:** Debug/logging values
  - Identify intentional truncations

- [ ] **3.3** Fix critical warnings (60min)
  - Focus on `src/` core library code
  - Use `TryFrom` with proper error handling
  - Add explicit bounds checks where appropriate
  - Add `#[allow]` with justification for intentional casts

- [ ] **3.4** Verify fixes (20min)
  - Re-run clippy
  - Ensure fixed warnings stay fixed
  - Run test suite

---

## Fix Patterns

### Pattern 1: TryFrom with expect

```rust
// BEFORE: Can truncate on 64-bit systems
let index = some_usize as u32;

// AFTER: Explicit error on overflow
let index = u32::try_from(some_usize)
    .expect("index overflow: max 4B entries supported");
```

### Pattern 2: Explicit bounds check

```rust
// BEFORE
let len = slice.len() as u32;

// AFTER
assert!(slice.len() <= u32::MAX as usize, "slice too large");
let len = slice.len() as u32;
```

### Pattern 3: Saturating cast (for metrics)

```rust
// For non-critical values where saturation is acceptable
let count = counter.min(u32::MAX as usize) as u32;
```

### Pattern 4: Justified allow

```rust
// When truncation is intentional and safe
#[allow(clippy::cast_possible_truncation)]
// Truncation intentional: only lower 32 bits needed for hash bucket
let bucket = hash as u32;
```

---

## Priority Files

Expected high-impact files:
1. `src/hnsw/mod.rs` - Graph indices
2. `src/storage/mod.rs` - Vector storage
3. `src/quantization/mod.rs` - Quantization indices
4. `src/index.rs` - Main index operations

---

## Acceptance Criteria

- [ ] ~25 cast_possible_truncation warnings addressed
- [ ] All critical/medium priority casts fixed properly
- [ ] Low priority casts either fixed or `#[allow]` with justification
- [ ] No new test failures

---

## Tracking

Create a checklist as you work:

```markdown
## Cast Fixes Progress

### src/hnsw/mod.rs
- [ ] Line XXX: [description]
- [ ] Line YYY: [description]

### src/storage/mod.rs
- [ ] Line XXX: [description]

[Continue for each file...]
```

---

## Exit Criteria

Day 3 is complete when:
- [ ] ~25 warnings addressed (first half)
- [ ] All fixes use appropriate pattern
- [ ] Tests pass
- [ ] Progress documented for Day 4

---

## Commit Message Template

```
fix(types): address cast_possible_truncation warnings (Part 1)

- Fix N critical index/size casts with TryFrom
- Add bounds checks for M storage operations
- Add #[allow] with justification for K intentional casts

Progress: ~25/50 warnings addressed

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```

---

**Day 3 Total:** 2 hours
**Agent:** RUST_ENGINEER
