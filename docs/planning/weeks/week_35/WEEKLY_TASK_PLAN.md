# Week 35: Technical Debt + v0.8.0 Release

**Date Range:** 2026-01-27 to 2026-02-02
**Focus:** Milestone 8.4 (Technical Debt) + v0.8.0 Release
**Total Hours:** ~14h
**Status:** [ ] PENDING

---

## Strategic Context

Week 35 is the **final week of v0.8.0** cycle. Focus on:
1. Eliminating technical debt identified in Reddit feedback (chillfish8)
2. Fixing clippy warnings accumulated during rapid development
3. Preparing and shipping v0.8.0 release

**Source:** ROADMAP.md v6.0, Milestone 8.4

---

## Week Summary

| Day | Focus | Hours | Priority |
|:----|:------|:------|:---------|
| 1 | WAL chunk_size edge case fix | 2h | P1 |
| 2 | Safety doc placement cleanup | 2h | P2 |
| 3 | cast_possible_truncation fixes (Part 1) | 2h | P2 |
| 4 | cast_possible_truncation fixes (Part 2) | 2h | P2 |
| 5 | Test clippy warnings + Comparison doc | 2h | P2/P3 |
| 6 | v0.8.0 Release preparation | 2h | P0 |
| 7 | Hostile review + v0.8.0 Release | 2h | P0 |

**Total:** ~14 hours

---

## Milestone 8.4: Technical Debt Reduction

### Task Breakdown

| ID | Task | Hours | Priority | Source |
|:---|:-----|:------|:---------|:-------|
| W35.1 | WAL chunk_size edge case fix | 2h | P1 | chillfish8 feedback |
| W35.2 | Safety doc placement cleanup | 2h | P2 | clippy audit |
| W35.3 | cast_possible_truncation fixes | 4h | P2 | clippy (50+ warnings) |
| W35.4 | Test code clippy warnings | 2h | P3 | cleanup |
| W35.5 | "EdgeVec vs pgvector" doc | 2h | P2 | Milestone 8.3 |
| W35.6 | v0.8.0 release | 2h | P0 | release |

---

## Technical Debt Details

### W35.1: WAL chunk_size Edge Case (P1)

**Problem:** WAL persistence has edge case with chunk_size parameter.
**Source:** chillfish8 Reddit feedback (December 2025)
**Impact:** Potential data corruption on edge cases

**Investigation:**
- Locate WAL chunk handling code
- Identify edge case scenario
- Write failing test
- Implement fix
- Verify with property tests

---

### W35.2: Safety Doc Placement (P2)

**Problem:** `#[doc]` safety comments placed incorrectly.
**Source:** Clippy lint `clippy::undocumented_unsafe_blocks`

**Pattern to fix:**
```rust
// BEFORE (wrong)
/// # Safety
/// - Must be valid pointer
unsafe fn foo() { ... }

// AFTER (correct)
/// Does something unsafe.
///
/// # Safety
///
/// - Must be valid pointer
unsafe fn foo() { ... }
```

**Scope:** Audit all `unsafe` blocks in `src/`

---

### W35.3: cast_possible_truncation (P2)

**Problem:** ~50 warnings for potential truncation in integer casts.
**Source:** `cargo clippy -- -W clippy::cast_possible_truncation`

**Common patterns:**
```rust
// WARNING: usize -> u32 can truncate on 64-bit
let index = some_usize as u32;

// FIX: Use TryFrom with explicit handling
let index = u32::try_from(some_usize)
    .expect("index overflow: max 4B entries");

// OR: Add explicit bounds check
assert!(some_usize <= u32::MAX as usize);
let index = some_usize as u32;
```

**Strategy:**
1. Run `cargo clippy` to get full list
2. Categorize by severity
3. Fix highest-risk casts first
4. Add `#[allow]` with justification for intentional casts

---

### W35.4: Test Code Clippy Warnings (P3)

**Scope:** `tests/` and `benches/` directories
**Goal:** Clean test output, no warnings during CI

---

### W35.5: "EdgeVec vs pgvector" Comparison Doc (P2)

**Source:** Milestone 8.3 documentation
**Location:** `docs/guides/COMPARISON_PGVECTOR.md`

**Content:**
- Feature comparison table
- Performance comparison (where applicable)
- Use case guidance
- Migration considerations

---

## v0.8.0 Release Checklist

### Version Updates
- [ ] `Cargo.toml` version = "0.8.0"
- [ ] `pkg/package.json` version = "0.8.0"
- [ ] README badges updated

### CHANGELOG Entry
```markdown
## [0.8.0] - 2026-02-02

### Added
- Vue 3 composables (`useEdgeVec`, `useSearch`)
- Standalone filter function exports (`eq`, `ne`, `gt`, etc.)
- 25 filter examples documentation
- Embedding integration guide (5 providers)
- SIMD Euclidean distance (Week 32)

### Fixed
- WAL chunk_size edge case
- Safety doc placement (clippy compliance)
- 50+ cast_possible_truncation warnings

### Documentation
- Filter examples guide (25 examples)
- Embedding integration guide (Ollama, Transformers.js, OpenAI, Cohere, HuggingFace)
- Vue composables documentation
```

### Quality Gates
- [ ] All tests pass (`cargo test --all-features`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] WASM builds (`npm run build`)
- [ ] TypeScript compiles (`npx tsc --noEmit`)
- [ ] HOSTILE_REVIEWER approval

### Release Commands
```bash
# Verify everything
cargo test --all-features
cargo clippy -- -D warnings
npm run build
npx tsc --noEmit

# Create tag
git tag -a v0.8.0 -m "v0.8.0: Consolidation + Developer Experience"
git push origin v0.8.0

# Publish
cargo publish
cd pkg && npm publish && cd ..

# GitHub release
gh release create v0.8.0 --title "v0.8.0: Consolidation + Developer Experience" --notes-file CHANGELOG_v0.8.0.md
```

---

## Exit Criteria

Week 35 is complete when:

- [ ] WAL edge case fixed with test
- [ ] Safety docs properly placed
- [ ] cast_possible_truncation warnings addressed
- [ ] Test clippy clean
- [ ] "EdgeVec vs pgvector" doc complete
- [ ] v0.8.0 released to crates.io + npm
- [ ] GitHub release published
- [ ] HOSTILE_REVIEWER approval

---

## Success Metrics

| Metric | Before | Target |
|:-------|:-------|:-------|
| Clippy warnings | ~50 | <10 |
| Test warnings | ~20 | 0 |
| Technical debt issues | 4 | 0 |
| v0.8.0 released | No | Yes |

---

## Dependencies

- Week 34 COMPLETE (Vue composables, Filter examples, Embedding guide)
- SIMD consolidation (Week 32) COMPLETE

---

## Risk Mitigation

| Risk | Mitigation |
|:-----|:-----------|
| WAL fix complex | Time-box to 2h, document if incomplete |
| Too many clippy fixes | Prioritize P1/P2, defer P3 to v0.8.1 |
| Release blockers | Day 7 buffer for unexpected issues |

---

## Notes

- v0.8.0 is the "Consolidation + Developer Experience" release
- Focus on quality over new features
- This completes Phase 8 of the roadmap
- v0.9.0 (Community Features) starts Week 36

---

**Agent:** PLANNER
**Version:** 1.0.0
**Created:** 2026-01-26
