# Week 9: Alpha Release Execution Plan

**Date Range:** 2025-12-13 to 2025-12-19 (Days 41-45)
**Milestone:** v0.2.0-alpha.1 Public Release
**Author:** PLANNER
**Status:** [REVISED]
**Last Updated:** 2025-12-12

---

## Objective

Execute controlled alpha release of EdgeVec, establish public repository, and prepare for community feedback.

---

## Week 9 Task Summary

| ID | Task | Owner | Est. (3x) | Dependencies | Status |
|:---|:-----|:------|:----------|:-------------|:-------|
| W9.1 | Pre-release checklist validation | RUST_ENGINEER | 6h | W8 complete | [ ] |
| W9.2 | GitHub repository setup | DOCWRITER | 3h | W9.1 | [ ] |
| W9.3 | Create git tag & GitHub release | RUST_ENGINEER | 3h | W9.2 | [ ] |
| W9.4 | npm package publish | WASM_SPECIALIST | 3h | W9.3 | [ ] |
| W9.5 | Community announcement | DOCWRITER | 3h | W9.4 | [ ] |
| W9.6 | Monitor feedback & triage | ALL | 12h | W9.5 | [ ] |

**Total Estimated:** 30 hours (with 3x buffer)

---

## W9.1: Pre-Release Checklist Validation (6 hours)

### Code Quality Checks

```bash
# Run all tests
cargo test --all-features
# Expected: All tests pass

# Run clippy
cargo clippy -- -D warnings
# Expected: No warnings

# Check formatting
cargo fmt --check
# Expected: No formatting issues

# Build release
cargo build --release
# Expected: Builds successfully
```

### Package Quality Checks

```bash
# WASM build
cd wasm && wasm-pack build --target web --release
# Expected: Builds successfully

# npm pack test
cd wasm && npm pack --dry-run
# Expected: Package valid, ~148KB gzipped

# TypeScript check
cd wasm && npx tsc --noEmit
# Expected: No type errors
```

### Documentation Checks

- [ ] README.md is accurate and up-to-date
- [ ] CHANGELOG.md has v0.2.0-alpha.1 entry
- [ ] KNOWN_LIMITATIONS.md is complete
- [ ] PERFORMANCE_GUIDE.md has example output
- [ ] API documentation builds (`cargo doc --no-deps`)

### Acceptance Criteria

| Check | Pass/Fail | Expected Output | Notes |
|:------|:----------|:----------------|:------|
| `cargo test` | [ ] | `test result: ok. X passed` | All tests pass |
| `cargo clippy` | [ ] | `Finished` with 0 warnings | -D warnings |
| `cargo fmt --check` | [ ] | No output (exit 0) | All formatted |
| `wasm-pack build` | [ ] | `[INFO]: Your wasm pkg is ready` | WASM OK |
| `npm pack --dry-run` | [ ] | ~19 files, ~148KB gzipped | Valid pkg |
| README accurate | [ ] | Version 0.2.0-alpha.1 mentioned | Manual |
| CHANGELOG complete | [ ] | v0.2.0-alpha.1 entry exists | Manual |

---

## W9.2: GitHub Repository Setup (3 hours)

### Repository Configuration

**Note:** GitHub repository does not exist yet. User will create it.

**Repository Name:** `edgevec` (or user's preference)
**Visibility:** Public
**License:** MIT (already in repo)

### Required Files

- [ ] `.gitignore` (already exists)
- [ ] `LICENSE` (already exists - MIT)
- [ ] `README.md` (already exists)
- [ ] `CHANGELOG.md` (created in W8.13)
- [ ] `CONTRIBUTING.md` (optional, can be added later)

### GitHub Settings to Configure

1. **About section:**
   - Description: "High-performance vector search for Browser, Node, and Edge"
   - Topics: `vector-search`, `hnsw`, `wasm`, `rust`, `nearest-neighbor`, `embeddings`

2. **Features to enable:**
   - Issues: ON
   - Discussions: OFF (for alpha)
   - Wiki: OFF

3. **Branch protection (optional for alpha):**
   - Protect `main` branch
   - Require PR reviews before merging

### Commands for User

```bash
# Initialize git (if not already)
git init

# Add remote (user replaces with their repo URL)
git remote add origin https://github.com/YOUR_USERNAME/edgevec.git

# Push initial commit
git add -A
git commit -m "Initial commit: EdgeVec v0.2.0-alpha.1"
git push -u origin main
```

### Acceptance Criteria

| Check | Pass/Fail | Verification Method | Notes |
|:------|:----------|:--------------------|:------|
| Repository created | [ ] | `gh repo view` returns info | Public visibility |
| All files pushed | [ ] | `git status` shows "nothing to commit" | Clean state |
| README displays correctly | [ ] | View on GitHub web UI | Markdown renders |
| License visible | [ ] | LICENSE file shows MIT | GitHub detects it |

---

## W9.3: Create Git Tag & GitHub Release (3 hours)

### Tag Creation

```bash
# Create annotated tag
git tag -a v0.2.0-alpha.1 -m "Release v0.2.0-alpha.1

Alpha release of EdgeVec: High-performance vector search for Browser, Node, and Edge.

Highlights:
- HNSW indexing with O(log n) search
- Scalar Quantization (SQ8) for 3.6x memory reduction
- Sub-millisecond search at 100k scale
- WASM-native with IndexedDB persistence
- 148 KB gzipped bundle

See CHANGELOG.md for full details."

# Push tag
git push origin v0.2.0-alpha.1
```

### GitHub Release Draft

**Title:** v0.2.0-alpha.1 — Initial Alpha Release

**Body:**
```markdown
## EdgeVec v0.2.0-alpha.1

High-performance vector search for Browser, Node, and Edge.

### Highlights

- **Sub-millisecond search** at 100k vectors (329µs quantized, 572µs float32)
- **3.6x memory reduction** with Scalar Quantization (SQ8)
- **148 KB bundle** (70% under 500KB target)
- **WASM-native** with IndexedDB persistence
- **Zero network latency** — runs 100% locally

### Performance (768d vectors, k=10)

| Scale | Float32 | Quantized (SQ8) |
|:------|:--------|:----------------|
| 10k | 203 µs | **88 µs** |
| 50k | 480 µs | **167 µs** |
| 100k | 572 µs | **329 µs** |

### Quick Start

```javascript
import { EdgeVecClient } from '@edgevec/core';

const client = await EdgeVecClient.create({ dimensions: 768 });
client.insert(new Float32Array(768).fill(0.1));
const results = client.search(new Float32Array(768).fill(0.1), 10);
```

### Known Limitations

This is an **alpha release**. See [KNOWN_LIMITATIONS.md](docs/KNOWN_LIMITATIONS.md) for details.

### Links

- [Documentation](README.md)
- [Performance Guide](docs/PERFORMANCE_GUIDE.md)
- [Changelog](CHANGELOG.md)

---

**Full Changelog:** See [CHANGELOG.md](CHANGELOG.md)
```

### Acceptance Criteria

| Check | Pass/Fail | Verification Method | Notes |
|:------|:----------|:--------------------|:------|
| Tag created | [ ] | `git tag -l v0.2.0-alpha.1` shows tag | Annotated tag |
| Tag pushed | [ ] | `git ls-remote --tags origin` shows tag | Remote updated |
| GitHub release draft created | [ ] | `gh release view v0.2.0-alpha.1` works | Via GitHub CLI |
| Release notes accurate | [ ] | Performance numbers match CHANGELOG | Manual check |

---

## W9.4: npm Package Publish (3 hours)

### Pre-Publish Verification

```bash
# Navigate to wasm directory
cd wasm

# Verify package.json version
cat package.json | grep version
# Expected: "0.2.0-alpha.1"

# Test pack
npm pack --dry-run
# Expected: Lists files, ~148KB

# Test install from tarball
npm pack
mkdir /tmp/test-install && cd /tmp/test-install
npm init -y
npm install ../path/to/edgevec-core-0.2.0-alpha.1.tgz
node -e "const { EdgeVecClient } = require('@edgevec/core'); console.log('SUCCESS');"
```

### Publish Commands

**Note:** User must have npm account and be logged in.

```bash
# Login to npm (if not already)
npm login

# Publish with public access
npm publish --access public

# Verify publication
npm view @edgevec/core
```

### Rollback Procedure

If critical bug discovered within 72 hours:

```bash
# Unpublish (only works within 72 hours)
npm unpublish @edgevec/core@0.2.0-alpha.1

# Or deprecate (if >72 hours)
npm deprecate @edgevec/core@0.2.0-alpha.1 "Critical bug - please use 0.2.0-alpha.2"
```

### Acceptance Criteria

| Check | Pass/Fail | Verification Method | Notes |
|:------|:----------|:--------------------|:------|
| Pre-publish dry run | [ ] | `npm pack --dry-run` lists ~19 files | Must pass first |
| Local tarball test | [ ] | Install from .tgz, import works | Independent verify |
| npm login successful | [ ] | `npm whoami` returns username | Auth verified |
| npm publish successful | [ ] | `npm publish` exits 0 | No errors |
| `npm install @edgevec/core` works | [ ] | Fresh `npm init && npm i` | Test in /tmp |
| Package page shows correct version | [ ] | `npm view @edgevec/core version` = 0.2.0-alpha.1 | Via CLI |

---

## W9.5: Community Announcement (3 hours)

### Announcement Template

```markdown
🚀 EdgeVec v0.2.0-alpha.1 Released!

High-performance vector search for Browser, Node.js, and Edge devices.

✨ Features:
• HNSW indexing with O(log n) search
• First-class WASM support (148 KB bundle)
• Automatic IndexedDB persistence
• 3.6x memory reduction with SQ8 quantization
• Sub-millisecond search at 100k scale

📦 Install: `npm install @edgevec/core`

🔗 GitHub: [link]
📖 Docs: [link]

#vectorsearch #wasm #rust #javascript #typescript
```

### Channels (User Discretion)

1. **GitHub:** Release published (automated with tag)
2. **Twitter/X:** Brief announcement with performance highlight
3. **Reddit:** r/rust, r/webdev, r/javascript (check subreddit rules)
4. **Hacker News:** "Show HN" post (optional)
5. **LinkedIn:** Professional announcement

### Acceptance Criteria

| Check | Pass/Fail | Verification Method | Notes |
|:------|:----------|:--------------------|:------|
| GitHub release published | [ ] | `gh release list` shows v0.2.0-alpha.1 | Not draft |
| At least 1 social announcement | [ ] | Screenshot/link saved | User discretion |

---

## W9.6: Monitor Feedback & Triage (12 hours)

### Monitoring Checklist

**Day 1-2 (Critical Period):**
- [ ] Check GitHub issues every 4 hours
- [ ] Monitor npm download stats
- [ ] Respond to any critical bugs within 4 hours

**Day 3-5 (Stabilization):**
- [ ] Daily GitHub issue check
- [ ] Collect feedback for FAQ
- [ ] Identify common questions

### Issue Triage Labels

| Label | Priority | Response Time | Action |
|:------|:---------|:--------------|:-------|
| `critical` | P0 | 4 hours | Hotfix release |
| `bug` | P1 | 24 hours | Investigate |
| `question` | P2 | 48 hours | Answer or doc update |
| `enhancement` | P3 | Backlog | Add to v0.3.0 planning |

### Hotfix Procedure

If critical bug found:

```bash
# Create hotfix branch
git checkout -b hotfix/alpha.1-critical

# Fix bug
# ... make changes ...

# Update version
# Cargo.toml: version = "0.2.0-alpha.2"
# package.json: version = "0.2.0-alpha.2"

# Commit and tag
git commit -m "Hotfix: [description]"
git tag -a v0.2.0-alpha.2 -m "Hotfix release"
git push origin hotfix/alpha.1-critical
git push origin v0.2.0-alpha.2

# Publish hotfix
cd wasm && npm publish --access public
```

### Acceptance Criteria

| Check | Pass/Fail | Verification Method | Notes |
|:------|:----------|:--------------------|:------|
| No critical bugs (72h) | [ ] | GitHub Issues filtered by `critical` label | Zero open |
| All P0/P1 issues addressed | [ ] | No open `bug` issues older than 24h | Triaged |
| FAQ started (if questions arise) | [ ] | `docs/FAQ.md` created if >3 questions | Optional |

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|:-----|:------------|:-------|:-----------|
| npm publish fails | Low | High | Test with `--dry-run` first |
| Critical bug found | Medium | High | Hotfix buffer (Day 3-5) |
| Low adoption | Medium | Low | Community outreach |
| Negative feedback | Low | Medium | Respond professionally |
| GitHub repo issues | Low | Medium | Have backup plan |

---

## Success Criteria

| Metric | Target | Measurement |
|:-------|:-------|:------------|
| npm publish | Success | npm registry |
| GitHub release | Published | GitHub |
| Critical bugs (72h) | 0 | Issue tracker |
| All tests pass | 100% | CI |

---

## Week 9 Dependencies on User Actions

The following require **user action** (cannot be automated):

1. **Create GitHub repository** — User must create and configure
2. **npm login** — User must authenticate with npm account
3. **Social media posts** — User discretion on channels
4. **Monitor feedback** — User must check periodically

---

## Post-Week 9 Preview

```
Week 10: Community Feedback Integration
├── Collect alpha feedback
├── Prioritize bug fixes
└── Plan v0.3.0 features

Week 11: v0.3.0 Beta Planning
├── Bulk insert API design
├── Delete operation architecture
└── Performance regression tests

Week 12+: v0.3.0 Implementation
├── Implement bulk insert
├── Implement soft delete
└── Prepare beta release
```

---

## Approvals

| Role | Name | Signature | Date |
|:-----|:-----|:----------|:-----|
| PLANNER | AI_PLANNER | ✓ | 2025-12-12 |
| HOSTILE_REVIEWER | | [PENDING] | |

---

**END OF WEEK 9 RELEASE PLAN**
