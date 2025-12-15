# Day 1: Release Process Documentation (W18.1)

**Date:** Week 18, Day 1
**Task ID:** W18.1
**Agent:** DOCWRITER
**Status:** [REVISED]
**Revision:** v1.2 — Addresses C1, C2 + adds rollback procedures (9→10 Process Fix)

---

## Pre-Task Validation Checklist [v1.2]

**Before starting W18.1, verify:**

- [ ] v0.3.0 release is stable (no critical bugs reported)
- [ ] CI is green on main branch
- [ ] Week 17 hostile review conditions acknowledged
- [ ] No blocking issues from Week 17

---

## Buffer Allocation

| Component | Base | Buffer | Total |
|:----------|:----:|:------:|:-----:|
| Release Checklist | 2h | 0.5h | 2.5h |
| Pre-Release Script | 2h | 0.5h | 2.5h |
| **Rollback Procedures [v1.2]** | 1h | 0h | 1h |
| **Total** | **5h** | **1h** | **6h**

---

## Objective

Create comprehensive release protocol documentation that prevents the 4-hotfix scenario from Week 17. This addresses hostile review findings C1, C2, and m2.

---

## Context

### Week 17 Post-Mortem

The v0.3.0 release required 4 emergency hotfixes:

| Commit | Issue | Root Cause |
|:-------|:------|:-----------|
| `800c3e9` | Clippy errors | `--all-targets` not run |
| `c951d6b` | SIGILL crash | `target-cpu=native` in CI |
| `8cdb357` | 40+ min runtime | 36,600 proptest cases |
| `7a03793` | 60s+ test hang | 10,000 vectors in integration test |

**All of these could have been prevented with a proper release checklist.**

---

## Deliverables

### 1. docs/RELEASE_CHECKLIST.md

Complete release protocol including:
- Pre-release local validation steps
- CI simulation commands
- Branch-based release workflow
- Post-release verification
- Rollback procedure

### 2. scripts/pre-release-check.sh

Automated script that runs all pre-release validation:

```bash
#!/bin/bash
# EdgeVec Pre-Release Validation Script
# Run this before any release to catch CI issues locally
#
# Addresses hostile review findings:
# - C1: cargo publish --dry-run
# - C2: npm publish --dry-run

set -e

echo "=== EdgeVec Pre-Release Check ==="
echo "Simulating CI environment..."

# Set CI-equivalent environment
export RUSTFLAGS="-C target-cpu=x86-64-v2"
export PROPTEST_CASES=32
export NUM_VECTORS=1000

echo ""
echo "1. Checking formatting..."
cargo fmt -- --check

echo ""
echo "2. Running Clippy (all targets)..."
cargo clippy --all-targets -- -D clippy::correctness -W clippy::suspicious -W clippy::style

echo ""
echo "3. Running test suite..."
time cargo test --all

echo ""
echo "4. Building WASM..."
wasm-pack build --release

echo ""
echo "5. Checking WASM target compilation..."
cargo check --target wasm32-unknown-unknown

echo ""
echo "6. [C1 FIX] Cargo publish dry-run..."
cargo publish --dry-run
if [ $? -ne 0 ]; then
    echo "FAIL: cargo publish --dry-run failed"
    exit 1
fi

echo ""
echo "7. [C2 FIX] NPM publish dry-run..."
cd pkg
npm pack --dry-run
if [ $? -ne 0 ]; then
    echo "FAIL: npm pack --dry-run failed"
    exit 1
fi
cd ..

echo ""
echo "8. Generating documentation..."
cargo doc --no-deps 2>&1 | grep -c "warning" && echo "WARNING: Doc warnings found" || echo "Docs clean"

echo ""
echo "=== All checks passed! ==="
echo "Safe to proceed with release."
echo ""
echo "Next steps:"
echo "  1. Create release branch: git checkout -b release/vX.Y.Z"
echo "  2. Push and wait for CI: git push -u origin release/vX.Y.Z"
echo "  3. After CI green, merge to main"
echo "  4. Tag: git tag vX.Y.Z && git push origin vX.Y.Z"
echo "  5. Publish: cargo publish && cd pkg && npm publish"
```

### 3. Rollback Procedures [v1.2 NEW]

**New deliverable for 10/10 Process Fix score.**

Create `docs/ROLLBACK_PROCEDURES.md`:

```markdown
# EdgeVec Rollback Procedures

## Quick Reference

| Scenario | Action | Command |
|:---------|:-------|:--------|
| Bad crates.io release | Yank version | `cargo yank --version X.Y.Z` |
| Bad npm release | Deprecate | `npm deprecate edgevec@X.Y.Z "reason"` |
| CI broken after merge | Revert commit | `git revert <sha>` |
| Critical security bug | Yank + patch | Yank → Fix → Re-release |

## Incident Response Checklist

### Phase 1: Assessment (< 5 minutes)

- [ ] Identify severity: CRITICAL / HIGH / MEDIUM / LOW
- [ ] Identify affected versions
- [ ] Check if issue is in crates.io AND/OR npm

### Phase 2: Containment (< 15 minutes)

For CRITICAL issues:
```bash
# Yank from crates.io
cargo yank --version X.Y.Z

# Deprecate on npm
npm deprecate edgevec@X.Y.Z "Critical issue discovered. Use X.Y.W instead."
```

### Phase 3: Communication (< 30 minutes)

- [ ] Update GitHub release notes with warning
- [ ] Post issue on GitHub with [INCIDENT] tag
- [ ] Notify users via changelog

### Phase 4: Resolution

1. Create hotfix branch: `git checkout -b hotfix/vX.Y.W`
2. Fix issue with tests
3. Run full pre-release check: `./scripts/pre-release-check.sh`
4. Wait for CI green
5. Release hotfix version
6. Unyank/undeprecate if safe to do so

## Version Yanking Policy

| Scenario | Yank? | Notes |
|:---------|:-----:|:------|
| Security vulnerability | YES | Always yank, even if patch exists |
| Build failure | YES | Prevents installation failures |
| Data corruption bug | YES | Protect user data |
| Performance regression | NO | Users can choose to downgrade |
| Missing feature | NO | Not a safety issue |
| Documentation error | NO | Not a runtime issue |
```

### 4. CONTRIBUTING.md Update

Add section linking to release checklist and rollback procedures.

---

## Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.1.1 | `docs/RELEASE_CHECKLIST.md` created | File exists |
| AC18.1.2 | CI validation commands documented | Commands listed |
| AC18.1.3 | Branch-based release workflow documented | Process diagram |
| AC18.1.4 | Pre-release CI simulation script | `scripts/pre-release-check.sh` |
| AC18.1.5 | Environment variables documented | CI env vars listed |
| AC18.1.6 | Post-release verification steps | Checklist items |

---

## Implementation Plan

### Step 1: Create RELEASE_CHECKLIST.md

```markdown
# EdgeVec Release Checklist

## Overview
This checklist ensures releases are validated before publication,
preventing post-release hotfixes like those required for v0.3.0.

## Pre-Release Validation

### Phase 1: Local Code Quality
- [ ] `cargo fmt -- --check`
- [ ] `cargo clippy --all-targets -- -D clippy::correctness`
- [ ] `cargo doc --no-deps` (no warnings)

### Phase 2: CI Simulation (CRITICAL)
Run with CI-equivalent settings:
```bash
export RUSTFLAGS="-C target-cpu=x86-64-v2"
export PROPTEST_CASES=32
export NUM_VECTORS=1000
cargo test --all
```

Verify:
- [ ] All tests pass
- [ ] Test suite completes in < 15 minutes
- [ ] No SIGILL or illegal instruction errors

### Phase 3: WASM Validation
- [ ] `cargo check --target wasm32-unknown-unknown`
- [ ] `wasm-pack build --release`
- [ ] Bundle size < 500KB

### Phase 4: Dry Run
- [ ] `cargo publish --dry-run`
- [ ] `npm pack --dry-run`

### Phase 5: Branch-Based Release
1. Create branch: `git checkout -b release/vX.Y.Z`
2. Push: `git push -u origin release/vX.Y.Z`
3. Wait for CI green on ALL jobs
4. Merge to main: `git checkout main && git merge release/vX.Y.Z`
5. Tag: `git tag vX.Y.Z`
6. Push tag: `git push origin vX.Y.Z`

### Phase 6: Publication
- [ ] `cargo publish`
- [ ] `npm publish` (with OTP)
- [ ] Create GitHub release with changelog

### Phase 7: Post-Release Verification
- [ ] CI green on main
- [ ] crates.io page accessible
- [ ] npm package installable
- [ ] Example code works

## Emergency Rollback

If issues discovered post-release:
1. `cargo yank --version X.Y.Z` (crates.io)
2. `npm deprecate edgevec@X.Y.Z "Critical issue"` (npm)
3. Create hotfix branch
4. Follow standard release process
```

### Step 2: Create Pre-Release Script

Create `scripts/pre-release-check.sh` with executable permissions.

### Step 3: Update CONTRIBUTING.md

Add link to release checklist.

---

## Files to Create/Modify

| File | Action | Description |
|:-----|:-------|:------------|
| `docs/RELEASE_CHECKLIST.md` | CREATE | Full release protocol |
| `scripts/pre-release-check.sh` | CREATE | Automated validation |
| `CONTRIBUTING.md` | MODIFY | Link to checklist |

---

## Verification Commands

```bash
# Verify checklist exists
test -f docs/RELEASE_CHECKLIST.md && echo "PASS: Checklist exists"

# Verify script exists and is executable
test -x scripts/pre-release-check.sh && echo "PASS: Script executable"

# Verify CONTRIBUTING.md updated
grep -q "RELEASE_CHECKLIST" CONTRIBUTING.md && echo "PASS: CONTRIBUTING updated"
```

---

## Handoff

**On Completion:**
- Mark W18.1 as COMPLETE
- Submit for hostile review: `/review docs/RELEASE_CHECKLIST.md`
- Proceed to W18.2
