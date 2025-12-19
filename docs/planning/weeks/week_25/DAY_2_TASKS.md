# Week 25 Day 2: Bug Fixes & Polish

**Date:** 2025-12-21
**Focus:** Address any reported issues, minor improvements
**Estimated Duration:** 4-6 hours (if bugs found) or 2 hours (if no bugs)

---

## Tasks

### W25.2.1: P0/P1 Bug Fixes

**Objective:** Fix any critical or high-priority bugs from Day 1 triage.

**Acceptance Criteria:**
- [ ] All P0 bugs fixed and tested
- [ ] All P1 bugs fixed or documented with workaround
- [ ] Regression tests added for each fix
- [ ] `cargo test` passes

**Deliverables:**
- Bug fix commits
- Regression tests

**Dependencies:** W25.1.4 (issue triage)

**Estimated Duration:** 2-4 hours (variable)

**Agent:** RUST_ENGINEER

**Note:** If no P0/P1 bugs found, mark as SKIPPED and proceed to W25.2.2.

---

### W25.2.2: Error Message Improvements

**Objective:** Review and improve error messages in Filter API.

**Acceptance Criteria:**
- [ ] Audit all error messages in `src/filter/parser.rs`
- [ ] Ensure errors include position information
- [ ] Ensure suggestions are helpful
- [ ] Add context to generic errors

**Deliverables:**
- Improved error messages
- Updated tests

**Dependencies:** None

**Estimated Duration:** 1.5 hours

**Agent:** RUST_ENGINEER

**Focus Areas:**
```rust
// Before: "Expected operator"
// After: "Expected comparison operator (=, !=, >, <, >=, <=) at position 15, found 'xyz'"
```

---

### W25.2.3: Documentation Quick Fixes

**Objective:** Fix any documentation issues discovered during smoke test or community feedback.

**Acceptance Criteria:**
- [ ] Fix any typos or broken links in README
- [ ] Update any outdated examples
- [ ] Ensure Filter API examples are copy-paste ready
- [ ] Verify all docs/api/*.md files are accurate

**Deliverables:**
- Documentation updates

**Dependencies:** W25.1.5 (smoke test)

**Estimated Duration:** 1 hour

**Agent:** DOCWRITER

---

### W25.2.4: CHANGELOG Cleanup

**Objective:** Ensure CHANGELOG is complete and accurate.

**Acceptance Criteria:**
- [ ] All v0.5.0 features documented
- [ ] All v0.5.1 changes documented
- [ ] Links at bottom are correct
- [ ] Version comparison table updated

**Deliverables:**
- Updated CHANGELOG.md (if needed)

**Dependencies:** None

**Estimated Duration:** 30 minutes

**Agent:** DOCWRITER

---

### W25.2.5: Clippy & Formatting Audit

**Objective:** Ensure codebase is clean and warning-free.

**Acceptance Criteria:**
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] No TODO/FIXME comments in critical paths
- [ ] No dead code warnings

**Deliverables:**
- Clean codebase

**Dependencies:** None

**Estimated Duration:** 30 minutes

**Agent:** RUST_ENGINEER

**Commands:**
```bash
cargo clippy -- -D warnings
cargo fmt --check
grep -r "TODO\|FIXME" src/ --include="*.rs" | head -20
```

---

## Day 2 Checklist

- [ ] W25.2.1: P0/P1 bugs fixed (or SKIPPED if none)
- [ ] W25.2.2: Error messages improved
- [ ] W25.2.3: Documentation quick fixes
- [ ] W25.2.4: CHANGELOG cleanup
- [ ] W25.2.5: Clippy clean

## Day 2 Exit Criteria

- Zero P0/P1 bugs outstanding
- Codebase is clippy-clean
- Documentation is accurate

---

*Agent: RUST_ENGINEER / DOCWRITER*
*Status: [PROPOSED]*
