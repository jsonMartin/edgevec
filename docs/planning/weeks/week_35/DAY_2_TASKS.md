# Week 35 Day 2: Safety Doc Placement Cleanup

**Date:** 2026-01-28
**Focus:** Fix clippy `undocumented_unsafe_blocks` warnings
**Hours:** 2h
**Status:** [ ] PENDING

---

## Context

Clippy lint `clippy::undocumented_unsafe_blocks` requires safety documentation to be placed correctly relative to `unsafe` blocks.

**Priority:** P2 - Code quality and maintainability
**Source:** Clippy audit

---

## Tasks

### W35.2: Safety Doc Placement (2h)

**Goal:** All `unsafe` blocks properly documented per clippy standards.

**Subtasks:**

- [ ] **2.1** Audit unsafe blocks (30min)
  - Run `cargo clippy -- -W clippy::undocumented_unsafe_blocks`
  - List all violations
  - Categorize by file

- [ ] **2.2** Fix documentation placement (60min)
  - Update each `unsafe` block with correct format
  - Add `// SAFETY:` comments where missing
  - Ensure comments explain why operation is safe

- [ ] **2.3** Verify fixes (30min)
  - Re-run clippy with same lint enabled
  - Ensure zero warnings
  - Run full test suite

---

## Pattern Reference

### Correct Documentation Style

```rust
// WRONG: Doc comment above function
/// # Safety
/// - Must be valid pointer
unsafe fn foo() { ... }

// CORRECT: SAFETY comment directly above unsafe block
unsafe fn foo() {
    // SAFETY: Pointer validity is guaranteed by constructor invariant
    unsafe { ... }
}

// CORRECT: For unsafe functions
/// Does something with raw pointers.
///
/// # Safety
///
/// Caller must ensure:
/// - `ptr` is valid and properly aligned
/// - `ptr` points to initialized memory
unsafe fn bar(ptr: *const u8) { ... }
```

### Common Patterns in EdgeVec

```rust
// SIMD operations
// SAFETY: CPU feature check performed at runtime via is_x86_feature_detected!
unsafe { _mm256_loadu_ps(ptr) }

// Pointer arithmetic
// SAFETY: Index is bounds-checked above, slice is contiguous
unsafe { slice.get_unchecked(index) }

// Memory transmutation
// SAFETY: f32 and [u8; 4] have identical size and alignment
unsafe { std::mem::transmute::<f32, [u8; 4]>(value) }
```

---

## Files to Audit

Expected locations:
- `src/simd/*.rs` - SIMD operations
- `src/storage/*.rs` - Raw pointer operations
- `src/hnsw/*.rs` - Graph operations
- `src/quantization/*.rs` - Memory layout operations

---

## Acceptance Criteria

- [ ] `cargo clippy -- -W clippy::undocumented_unsafe_blocks` produces zero warnings
- [ ] All `unsafe` blocks have `// SAFETY:` comments
- [ ] Comments explain WHY the operation is safe, not WHAT it does
- [ ] All tests pass

---

## Deliverables

- Updated Rust files with correct safety documentation
- Zero clippy warnings for this lint

---

## Exit Criteria

Day 2 is complete when:
- [ ] All unsafe blocks documented
- [ ] Clippy clean for this lint
- [ ] Tests pass

---

## Commit Message Template

```
fix(safety): correct unsafe block documentation placement

- Add SAFETY comments to all unsafe blocks
- Fix doc placement per clippy::undocumented_unsafe_blocks
- Document safety invariants for SIMD operations

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```

---

**Day 2 Total:** 2 hours
**Agent:** RUST_ENGINEER
