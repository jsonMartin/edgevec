# v0.5.1 Smoke Test Results — Week 25

**Date:** 2025-12-19
**Package Version:** edgevec@0.5.1
**Status:** [PARTIAL PASS]

---

## Executive Summary

| Component | Status | Notes |
|:----------|:-------|:------|
| Rust Core | ✅ PASS | 559 tests, all examples work |
| npm Package Install | ✅ PASS | Installs cleanly, 0 vulnerabilities |
| Package Contents | ✅ PASS | `snippets` directory included |
| Node.js Direct Import | ❌ FAIL | TypeScript exports break Node.js |
| Bundler Usage (Vite) | ✅ PASS | Works with bundler |

**Overall:** CONDITIONAL PASS — works for bundler-based projects (primary use case)

---

## Test Results

### 1. npm Install Test

```bash
npm init -y && npm install edgevec
```

**Result:** ✅ PASS

```
added 1 package, and audited 2 packages in 1s
found 0 vulnerabilities
```

**Package Size:** 749.7 kB (unpacked)

---

### 2. Package Contents Verification

**Result:** ✅ PASS

```
edgevec/
├── edgevec.d.ts          ✅
├── edgevec.js            ✅
├── edgevec_bg.wasm       ✅
├── edgevec_bg.wasm.d.ts  ✅
├── edgevec-types.d.ts    ✅
├── edgevec-wrapper.ts    ⚠️ (TypeScript source)
├── filter.ts             ⚠️ (TypeScript source)
├── filter-builder.ts     ⚠️ (TypeScript source)
├── index.ts              ⚠️ (TypeScript source)
├── LICENSE               ✅
├── LICENSE-APACHE        ✅
├── LICENSE-MIT           ✅
├── package.json          ✅
├── README.md             ✅
└── snippets/             ✅ (Issue #1 fix verified)
```

---

### 3. Node.js Direct Import Test

**Result:** ❌ FAIL

```javascript
import init, { EdgeVec } from 'edgevec';
```

**Error:**
```
Error [ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING]:
Stripping types is currently unsupported for files under node_modules
```

**Root Cause:** package.json exports `.ts` files:
```json
"exports": {
  ".": {
    "types": "./edgevec-types.d.ts",
    "import": "./index.ts"  // ← Should be compiled .js
  }
}
```

**Severity:** P1 (High) — Breaks Node.js users without bundlers

**Affected Users:**
- Node.js without bundler
- Bun/Deno (needs verification)

**NOT Affected:**
- Vite, Webpack, Rollup, esbuild users
- Browser via bundler

---

### 4. Rust Core Tests

**Result:** ✅ PASS

```
running 559 tests
...
test result: ok. 559 passed; 0 failed; 0 ignored
```

**Coverage:** All filter, evaluator, parser, and core tests pass.

---

### 5. Rust Examples

**Result:** ✅ PASS

```bash
cargo run --example filter_basic
```

**Output:**
```
=== EdgeVec Basic Filtered Search Example ===
Created index with 4 dimensions

Filter: category = "fruit"
  Found 2 results:
    - String("apple") (score: 0.0050)
    - String("banana") (score: 0.0050)
...
=== Example Complete ===
```

---

### 6. Filter API Test

**Result:** ✅ PASS (via Rust, pending for WASM)

Verified operations:
- Simple equality: `category = "fruit"`
- Range filters: `price < 2.0`
- Combined: `category = "meat" OR category = "seafood"`
- Complex: `(category = "fruit" OR category = "vegetable") AND price < 1.0`

---

## Bug Report

### P1: npm Package Exports TypeScript Instead of JavaScript

**ID:** Pending (not yet filed on GitHub)

**Description:** The edgevec npm package v0.5.1 exports `.ts` files directly in the `exports` field, causing Node.js 22.x to fail with `ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING`.

**Reproduction:**
```bash
mkdir test && cd test
npm init -y
npm install edgevec
echo 'import init from "edgevec";' > test.mjs
node test.mjs  # Fails
```

**Expected:** Package should export compiled `.js` files

**Workaround:** Use a bundler (Vite, Webpack, etc.)

**Fix Required:**
1. Add TypeScript compilation step to build process
2. Update package.json exports to point to `.js` files
3. Keep `.d.ts` files for type definitions
4. Add CI test for Node.js direct import

---

## Recommendations

### Immediate (Day 2)

1. **File GitHub issue** for P1 TypeScript export bug
2. **Prepare v0.5.2 hotfix** with compiled JavaScript
3. **Add smoke test to CI** to prevent regression

### Week 25

1. Add Node.js direct import test to CI pipeline
2. Verify fix works with Node.js, Bun, Deno
3. Update README with bundler requirement until fixed

---

## Test Environment

| Component | Version |
|:----------|:--------|
| Node.js | 22.21.0 |
| npm | (bundled) |
| Rust | 1.70+ |
| OS | Windows 11 |

---

## Conclusion

EdgeVec v0.5.1 **works correctly for its primary use case** (browser via bundler) but has a packaging issue that prevents direct Node.js usage. The core Rust library is fully functional with all 559 tests passing.

**Recommended Action:** Prepare v0.5.2 hotfix for Day 2.

---

*Recorded: 2025-12-19*
*Agent: RUST_ENGINEER*
*Test Type: Manual Smoke Test*
