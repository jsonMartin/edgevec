# Week 35 Day 6: v0.8.0 Release Preparation

**Date:** 2026-02-01
**Focus:** Prepare v0.8.0 release
**Hours:** 2h
**Status:** [ ] PENDING

---

## Context

v0.8.0 is the "Consolidation + Developer Experience" release. Day 6 prepares all release artifacts before final hostile review on Day 7.

**Priority:** P0 - Release critical
**Release Name:** v0.8.0: Consolidation + Developer Experience

---

## Tasks

### W35.6: Release Preparation (2h)

**Goal:** All release artifacts ready for hostile review.

**Subtasks:**

- [ ] **6.1** Version bumps (15min)
  - Update `Cargo.toml` version to "0.8.0"
  - Update `pkg/package.json` version to "0.8.0"
  - Verify all version strings consistent

- [ ] **6.2** CHANGELOG update (30min)
  - Create comprehensive v0.8.0 entry
  - List all features from Weeks 32-35
  - Document all fixes
  - Credit contributors

- [ ] **6.3** README updates (30min)
  - Update version badges
  - Verify all examples work
  - Check all links valid
  - Update feature list if needed

- [ ] **6.4** Quality gate verification (30min)
  - Run `cargo test --all-features`
  - Run `cargo clippy -- -D warnings`
  - Run `npm run build` (WASM)
  - Run `npx tsc --noEmit` (TypeScript)

- [ ] **6.5** Pre-release checklist (15min)
  - [ ] All tests pass
  - [ ] Clippy clean
  - [ ] WASM builds
  - [ ] TypeScript compiles
  - [ ] CHANGELOG complete
  - [ ] README updated
  - [ ] No uncommitted changes

---

## CHANGELOG v0.8.0 Template

```markdown
## [0.8.0] - 2026-02-02

### Added
- **Vue 3 Composables** (Week 34)
  - `useEdgeVec` - Database initialization with reactive state
  - `useSearch` - Reactive search with debouncing
  - Full TypeScript support with MaybeRef/MaybeRefOrGetter

- **Standalone Filter Functions** (Week 34)
  - Export `eq`, `ne`, `gt`, `gte`, `lt`, `lte` from main package
  - Export `contains`, `startsWith`, `endsWith` for strings
  - Export `all`, `any`, `and`, `or`, `not` for composition

- **Documentation** (Weeks 33-35)
  - 25 filter examples with real-world use cases
  - Embedding integration guide (Ollama, Transformers.js, OpenAI, Cohere, HuggingFace)
  - EdgeVec vs pgvector comparison guide

- **SIMD Optimizations** (Week 32)
  - Euclidean distance SIMD acceleration
  - Consolidated SIMD dispatch system

### Fixed
- WAL chunk_size edge case handling (Week 35)
- Safety documentation placement for clippy compliance (Week 35)
- 50+ cast_possible_truncation warnings resolved (Week 35)
- Test and bench clippy warnings cleaned (Week 35)

### Changed
- Consistent high-level API across all documentation
- Improved TypeScript type exports

### Documentation
- Filter examples guide (25 examples)
- Embedding integration guide (5 providers)
- Vue composables documentation
- EdgeVec vs pgvector comparison
```

---

## Version Files to Update

### Cargo.toml
```toml
[package]
name = "edgevec"
version = "0.8.0"  # Update this
```

### pkg/package.json
```json
{
  "name": "edgevec",
  "version": "0.8.0"  // Update this
}
```

---

## Quality Gate Commands

```bash
# Rust tests
cargo test --all-features

# Clippy (strict)
cargo clippy -- -D warnings

# WASM build
npm run build

# TypeScript check
npx tsc --noEmit

# Optional: Run benchmarks
cargo bench --no-run
```

---

## Acceptance Criteria

- [ ] Version bumped in all locations
- [ ] CHANGELOG comprehensive and accurate
- [ ] README current and all links work
- [ ] All quality gates pass
- [ ] Ready for hostile review

---

## Exit Criteria

Day 6 is complete when:
- [ ] All version strings updated
- [ ] CHANGELOG written
- [ ] README verified
- [ ] Quality gates pass
- [ ] Pre-release checklist complete
- [ ] Ready for Day 7 hostile review

---

## Commit Message Template

```
chore(release): prepare v0.8.0 release

- Bump version to 0.8.0 in Cargo.toml and package.json
- Update CHANGELOG with Week 32-35 changes
- Verify README accuracy
- All quality gates pass

Ready for HOSTILE_REVIEWER approval

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```

---

**Day 6 Total:** 2 hours
**Agent:** DOCWRITER + RUST_ENGINEER
