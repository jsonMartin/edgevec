# EdgeVec Release Checklist

**Version:** 1.0.0
**Created:** 2025-12-15 (W18.1)
**Purpose:** Prevent post-release hotfixes by validating releases locally before publication

---

## Overview

This checklist ensures releases are validated before publication, preventing post-release hotfixes like those required for v0.3.0 (4 emergency hotfixes due to missing pre-release validation).

**Key Lessons from v0.3.0:**

| Issue | Root Cause | Prevention |
|:------|:-----------|:-----------|
| Clippy errors in CI | `--all-targets` not run locally | Phase 1 validation |
| SIGILL crash | `target-cpu=native` in CI | Phase 2 CI simulation |
| 40+ min runtime | 36,600 proptest cases | Environment variables |
| 60s+ test hang | 10,000 vectors in tests | `NUM_VECTORS` control |

---

## Quick Start

For experienced maintainers, run the automated validation:

```bash
./scripts/pre-release-check.sh
```

If ALL checks pass, proceed to Phase 5 (Branch-Based Release).

### Platform Requirements

| Platform | Requirement |
|:---------|:------------|
| Linux | Native execution |
| macOS | Native execution |
| Windows | **WSL required** (Windows Subsystem for Linux) |

**Windows Users:** The pre-release script requires bash. Install WSL:
```powershell
wsl --install
```
Then run the script from within WSL:
```bash
wsl
cd /mnt/c/path/to/edgevec
./scripts/pre-release-check.sh
```

---

## Pre-Release Validation

### Phase 1: Local Code Quality

Run these checks in your development environment:

```bash
# 1. Formatting check
cargo fmt -- --check

# 2. Clippy with ALL targets (catches test/bench issues)
cargo clippy --all-targets -- -D clippy::correctness -W clippy::suspicious -W clippy::style

# 3. Documentation (no warnings)
cargo doc --no-deps 2>&1 | grep -c "warning" && echo "WARN: Doc warnings found" || echo "OK: Docs clean"
```

**Checklist:**
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy --all-targets` passes with no errors
- [ ] `cargo doc --no-deps` produces no warnings

### Phase 2: CI Simulation (CRITICAL)

**This phase prevents SIGILL crashes and timeout issues.**

Set environment variables to match CI configuration:

```bash
# CI-equivalent environment variables
export RUSTFLAGS="-C target-cpu=x86-64-v2"
export PROPTEST_CASES=32
export NUM_VECTORS=1000

# Run test suite
time cargo test --all
```

**Verify:**
- [ ] All tests pass
- [ ] Test suite completes in < 15 minutes
- [ ] No SIGILL or illegal instruction errors
- [ ] No timeout or hanging tests

**Why these values?**

| Variable | CI Value | Local Default | Purpose |
|:---------|:---------|:--------------|:--------|
| `RUSTFLAGS` | `-C target-cpu=x86-64-v2` | native | Prevents SIGILL on CI runners |
| `PROPTEST_CASES` | 32 | 256 | Reduces proptest runtime |
| `NUM_VECTORS` | 1000 | 10000 | Reduces integration test vectors |

### Phase 3: WASM Validation

```bash
# Check WASM target compilation
cargo check --target wasm32-unknown-unknown

# Build WASM package
wasm-pack build --release

# Verify bundle size
ls -la pkg/edgevec_bg.wasm
# Should be < 500KB
```

**Checklist:**
- [ ] `cargo check --target wasm32-unknown-unknown` passes
- [ ] `wasm-pack build --release` succeeds
- [ ] Bundle size < 500KB

### Phase 4: Dry Run (C1/C2 Fix)

**These dry runs catch packaging issues before publication.**

```bash
# Cargo publish dry run
cargo publish --dry-run

# NPM pack dry run
cd pkg
npm pack --dry-run
cd ..
```

**Checklist:**
- [ ] `cargo publish --dry-run` succeeds
- [ ] `npm pack --dry-run` succeeds
- [ ] No missing files in package

---

## Phase 5: Branch-Based Release

**Never release directly from main. Always use a release branch.**

### 5.1 Create Release Branch

```bash
# Create release branch
git checkout -b release/vX.Y.Z

# Verify you're on the release branch
git branch --show-current
# Should output: release/vX.Y.Z
```

### 5.2 Update Version Numbers

```bash
# Edit Cargo.toml version
# Edit pkg/package.json version
# Edit CHANGELOG.md (move Unreleased to vX.Y.Z)

# Commit version bump
git add -A
git commit -m "chore: bump version to vX.Y.Z"
```

### 5.3 Push and Wait for CI

```bash
# Push release branch
git push -u origin release/vX.Y.Z

# Wait for ALL CI checks to pass
# DO NOT PROCEED until CI is green
```

**Checklist:**
- [ ] CI is green on release branch
- [ ] All workflow jobs completed successfully:
  - `build` job (all platforms)
  - `test` job (unit, integration, proptest)
  - `clippy` job (lint checks)
  - `wasm` job (WASM build + browser tests)
- [ ] No warnings in CI logs
- [ ] Test duration < 15 minutes (watch for timeout issues)

### 5.4 Merge to Main

```bash
# Merge release branch to main
git checkout main
git pull origin main
git merge release/vX.Y.Z

# Push merged main
git push origin main
```

### 5.5 Tag Release

```bash
# Create annotated tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# Push tag (triggers release workflow)
git push origin vX.Y.Z
```

---

## Phase 6: Publication

### 6.1 Publish to crates.io

```bash
# Final verification
cargo publish --dry-run

# Publish (requires authentication)
cargo publish
```

### 6.2 Publish to npm

```bash
cd pkg

# Final verification
npm pack --dry-run

# Publish (requires OTP)
npm publish

cd ..
```

### 6.3 Create GitHub Release

1. Go to GitHub Releases
2. Click "Draft a new release"
3. Select tag: `vX.Y.Z`
4. Title: `EdgeVec vX.Y.Z`
5. Body: Copy from CHANGELOG.md
6. Attach any relevant assets
7. Click "Publish release"

---

## Phase 7: Post-Release Verification

**Verify the release is accessible and functional.**

### 7.1 Verify crates.io

```bash
# Check crates.io page
curl -s https://crates.io/api/v1/crates/edgevec | jq '.crate.max_version'
# Should return "X.Y.Z"

# Test installation
cargo new /tmp/test-edgevec && cd /tmp/test-edgevec
echo 'edgevec = "X.Y.Z"' >> Cargo.toml
cargo build
```

### 7.2 Verify npm

```bash
# Check npm page
npm view edgevec version
# Should return "X.Y.Z"

# Test installation
npm init -y
npm install edgevec@X.Y.Z
```

### 7.3 Verify CI Status

- [ ] CI remains green on main
- [ ] No new issues reported
- [ ] Documentation site updated (if applicable)

---

## Emergency Rollback

If critical issues are discovered post-release, see:
- **[ROLLBACK_PROCEDURES.md](./ROLLBACK_PROCEDURES.md)** for detailed rollback instructions

**Quick Reference:**

| Scenario | Action | Command |
|:---------|:-------|:--------|
| Bad crates.io release | Yank version | `cargo yank --version X.Y.Z` |
| Bad npm release | Deprecate | `npm deprecate edgevec@X.Y.Z "reason"` |
| CI broken after merge | Revert commit | `git revert <sha>` |

---

## Automated Script

For convenience, use the pre-release check script:

```bash
./scripts/pre-release-check.sh
```

This runs Phases 1-4 automatically and reports any failures.

---

## Checklist Summary

### Before Release Branch

- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy --all-targets` passes
- [ ] `cargo doc --no-deps` has no warnings
- [ ] CI simulation passes (with env vars)
- [ ] WASM build succeeds
- [ ] Bundle size < 500KB
- [ ] `cargo publish --dry-run` succeeds
- [ ] `npm pack --dry-run` succeeds

### During Release

- [ ] Release branch created
- [ ] Version numbers updated
- [ ] CI green on release branch
- [ ] Merged to main
- [ ] Tag pushed

### After Release

- [ ] crates.io accessible
- [ ] npm installable
- [ ] GitHub release created
- [ ] CI green on main

---

## Revision History

| Version | Date | Change |
|:--------|:-----|:-------|
| 1.0.0 | 2025-12-15 | Initial release (W18.1) |
| 1.1.0 | 2025-12-15 | Added Windows/WSL docs, clarified CI checks (hostile review fixes) |

---

**Maintainer:** EdgeVec Team
**Last Updated:** 2025-12-15
