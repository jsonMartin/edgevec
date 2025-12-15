# EdgeVec Rollback Procedures

**Version:** 1.0.0
**Created:** 2025-12-15 (W18.1)
**Purpose:** Incident response procedures for failed or problematic releases

---

## Quick Reference

| Scenario | Action | Command |
|:---------|:-------|:--------|
| Bad crates.io release | Yank version | `cargo yank --version X.Y.Z` |
| Bad npm release | Deprecate | `npm deprecate edgevec@X.Y.Z "reason"` |
| CI broken after merge | Revert commit | `git revert <sha>` |
| Critical security bug | Yank + patch | Yank -> Fix -> Re-release |

---

## Incident Response Checklist

### Phase 1: Assessment (< 5 minutes)

When an issue is discovered post-release:

- [ ] **Identify severity**
  - CRITICAL: Security vulnerability, data loss, crash on all platforms
  - HIGH: Build failure, crash on specific platforms, major functionality broken
  - MEDIUM: Performance regression, minor functionality broken
  - LOW: Documentation error, cosmetic issue

- [ ] **Identify affected versions**
  - Which version(s) are affected?
  - Was this introduced in the latest release or is it a regression?

- [ ] **Check distribution channels**
  - [ ] crates.io: `cargo search edgevec`
  - [ ] npm: `npm view edgevec versions`
  - [ ] GitHub releases: Check release page

---

### Phase 2: Containment (< 15 minutes)

**For CRITICAL or HIGH severity issues, immediately contain the release.**

#### Yank from crates.io

Yanking prevents new installations but allows existing Cargo.lock files to continue working.

```bash
# Yank the problematic version
cargo yank --version X.Y.Z

# Verify it's yanked
cargo search edgevec
# Should show previous version as latest
```

**When to yank:**
- Security vulnerability
- Build failure (users can't compile)
- Data corruption bug
- Crash on startup

**When NOT to yank:**
- Performance regression (users can choose to downgrade)
- Documentation error
- Missing feature

#### Deprecate on npm

NPM doesn't support yanking, but we can deprecate.

```bash
# Deprecate the problematic version
npm deprecate edgevec@X.Y.Z "Critical issue discovered. Use X.Y.W instead."

# Verify deprecation
npm view edgevec
# Should show deprecation warning
```

#### Revert Git Commits

If the issue is in the main branch:

```bash
# Identify the problematic commit
git log --oneline -10

# Revert the commit
git revert <commit-sha>

# Push the revert
git push origin main
```

---

### Phase 3: Communication (< 30 minutes)

**Notify users about the issue and resolution.**

#### Update GitHub Release Notes

1. Go to GitHub Releases
2. Edit the problematic release
3. Add warning banner at top:

```markdown
> **WARNING:** This release has been yanked due to [brief issue description].
> Please use version X.Y.W instead.
```

#### Create GitHub Issue

Create an issue with the `[INCIDENT]` tag:

```markdown
## [INCIDENT] vX.Y.Z Release Issue

### Summary
Brief description of the issue.

### Severity
CRITICAL / HIGH / MEDIUM / LOW

### Affected Versions
- vX.Y.Z

### Impact
What users might experience.

### Resolution
- [ ] Version yanked from crates.io
- [ ] Version deprecated on npm
- [ ] Hotfix released as vX.Y.W
- [ ] Communication sent

### Timeline
- YYYY-MM-DD HH:MM: Issue discovered
- YYYY-MM-DD HH:MM: Version yanked
- YYYY-MM-DD HH:MM: Hotfix released
```

#### Update CHANGELOG

Add incident note to CHANGELOG.md:

```markdown
## [X.Y.Z] - YANKED

**This version has been yanked due to [issue]. Use vX.Y.W instead.**
```

---

### Phase 4: Resolution

#### Create Hotfix Branch

```bash
# From main (with revert already applied)
git checkout -b hotfix/vX.Y.W

# Or from previous stable tag
git checkout -b hotfix/vX.Y.W vX.Y.Z-1
```

#### Fix the Issue

1. Implement fix with tests
2. Add regression test for the specific issue
3. Update CHANGELOG with fix description

#### Validate Hotfix

Run FULL pre-release validation:

```bash
./scripts/pre-release-check.sh
```

**Extra validation for security issues:**
- Run `cargo audit`
- Review all unsafe blocks
- Run fuzzing for affected code path

#### Release Hotfix

Follow standard release process (see RELEASE_CHECKLIST.md):

1. Bump version to X.Y.W
2. Push release branch
3. Wait for CI green
4. Merge to main
5. Tag and push
6. Publish to crates.io and npm

#### Unyank/Undeprecate (if applicable)

If the original version was only yanked as a precaution and the fix is confirmed:

```bash
# Unyank on crates.io (if appropriate)
cargo yank --undo --version X.Y.Z

# Note: npm deprecation cannot be undone, only updated
npm deprecate edgevec@X.Y.Z ""
```

**Only unyank if:**
- The issue was a false alarm
- The version is safe to use
- There's a good reason for users to use this specific version

---

## Partial Rollback Scenarios

Sometimes a release succeeds on one platform but fails on another. Handle these cases carefully:

### Scenario: crates.io Published, npm Failed

```bash
# Option 1: Yank crates.io and retry both
cargo yank --version X.Y.Z
# Fix npm issue, then re-release as X.Y.Z+1

# Option 2: Continue with npm only (if Rust crate is valid)
cd pkg
npm publish
# Document that npm release was delayed
```

**Decision guide:**
- If the Rust crate works correctly → Option 2 (publish npm separately)
- If the Rust crate has issues → Option 1 (yank and retry)

### Scenario: npm Published, crates.io Failed

```bash
# Option 1: Deprecate npm and retry both
npm deprecate edgevec@X.Y.Z "crates.io release failed, use X.Y.Z+1"
# Fix crates.io issue, re-release both as X.Y.Z+1

# Option 2: Continue with crates.io only (if npm package is valid)
cargo publish
# Document that crates.io release was delayed
```

**Decision guide:**
- If the npm package works correctly → Option 2 (publish crates separately)
- If the npm package has issues → Option 1 (deprecate and retry)

### Best Practice: Atomic Releases

To avoid partial rollback scenarios:
1. Always run `./scripts/pre-release-check.sh` first
2. Publish crates.io BEFORE npm (Rust is the source of truth)
3. If crates.io fails, stop immediately
4. Only publish to npm after crates.io succeeds

---

## Version Yanking Policy

| Scenario | Yank? | Notes |
|:---------|:-----:|:------|
| Security vulnerability | **YES** | Always yank, even if patch exists |
| Build failure | **YES** | Prevents installation failures |
| Data corruption bug | **YES** | Protect user data |
| Crash on startup | **YES** | Unusable version |
| Crash under specific conditions | MAYBE | Consider if workaround exists |
| Performance regression | **NO** | Users can choose to downgrade |
| Missing feature | **NO** | Not a safety issue |
| Documentation error | **NO** | Not a runtime issue |
| Wrong version number | MAYBE | Only if causes dependency issues |

---

## Incident Severity Definitions

### CRITICAL

- Security vulnerability (RCE, data exposure, privilege escalation)
- Data loss or corruption
- Crash on all platforms/configurations
- Breaks all users

**Response time:** Immediate (< 1 hour)
**Actions:** Yank immediately, communicate within 30 minutes

### HIGH

- Build failure (users can't compile)
- Crash on major platform (e.g., all Windows users)
- Major functionality completely broken
- Significant security issue (DoS, limited exposure)

**Response time:** < 4 hours
**Actions:** Yank, hotfix within 24 hours

### MEDIUM

- Performance regression > 2x
- Minor functionality broken
- Crash under specific (rare) conditions
- Documentation significantly wrong

**Response time:** < 24 hours
**Actions:** May not need to yank, hotfix within 1 week

### LOW

- Cosmetic issues
- Minor documentation errors
- Performance regression < 2x
- Non-critical warnings

**Response time:** Next regular release
**Actions:** No immediate action needed, fix in next version

---

## Rollback Scenarios

### Scenario A: SIGILL Crash in CI (v0.3.0 example)

**Symptoms:** CI fails with SIGILL (illegal instruction)

**Root cause:** Compiled with `-C target-cpu=native` but CI runners have older CPUs

**Resolution:**
1. Don't yank (users with compatible CPUs are fine)
2. Hotfix with `-C target-cpu=x86-64-v2`
3. Add CI simulation to release checklist

### Scenario B: Proptest Timeout

**Symptoms:** CI times out after 40+ minutes

**Root cause:** Too many proptest cases (36,600 instead of 32)

**Resolution:**
1. Don't yank (not a safety issue)
2. Hotfix with correct `PROPTEST_CASES`
3. Add environment variable checks to release checklist

### Scenario C: Security Vulnerability

**Symptoms:** CVE reported or vulnerability discovered

**Resolution:**
1. **IMMEDIATELY** yank affected versions
2. Create private hotfix branch
3. Fix vulnerability
4. Release hotfix BEFORE public disclosure (if possible)
5. Coordinate disclosure with security researchers

### Scenario D: Breaking API Change

**Symptoms:** Users report compilation failures after upgrade

**Resolution:**
1. **Do NOT yank** (semver violation is not a safety issue)
2. Release hotfix with API restored or migration guide
3. Plan proper deprecation for next minor version

---

## Post-Incident Review

After every incident, conduct a brief review:

1. **What happened?** (Timeline of events)
2. **Why did it happen?** (Root cause analysis)
3. **How was it detected?** (User report, CI failure, monitoring)
4. **How can we prevent it?** (Process improvement, automation)

Document in `docs/incidents/YYYY-MM-DD_vX.Y.Z_incident.md`.

---

## Contacts

| Role | Contact |
|:-----|:--------|
| Primary Maintainer | GitHub Issues: https://github.com/edgevec/edgevec/issues |
| Security Issues | Open a private security advisory on GitHub |
| crates.io Support | https://crates.io/support |
| npm Support | https://www.npmjs.com/support |

> **Note:** For security vulnerabilities, use GitHub's private security advisory feature
> rather than public issues. Go to Security → Advisories → New draft advisory.

---

## Revision History

| Version | Date | Change |
|:--------|:-----|:-------|
| 1.0.0 | 2025-12-15 | Initial release (W18.1) |
| 1.1.0 | 2025-12-15 | Added partial rollback scenarios, fixed contacts (hostile review fixes) |

---

**Related Documents:**
- [RELEASE_CHECKLIST.md](./RELEASE_CHECKLIST.md)
- [SECURITY.md](../SECURITY.md) (if exists)
- [CONTRIBUTING.md](../CONTRIBUTING.md)
