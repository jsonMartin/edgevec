# HOSTILE_REVIEWER: W25 Day 3 Desktop vs Mobile Discrepancy Report

**Date:** 2025-12-20
**Artifact:** Week 25 Day 3 iOS/Mobile Work
**Author:** WASM_SPECIALIST + RUST_ENGINEER
**Reviewer:** HOSTILE_REVIEWER
**Verdict:** REJECT

---

## Executive Summary

Week 25 Day 3 work introduced multiple regressions and cross-platform discrepancies. **Desktop functionality has broken navigation from index.html**, while mobile has different issues. The benchmark dashboard shows `NaN%` on mobile. This work is **NOT READY** for production.

---

## VERDICT

```
+---------------------------------------------------------------------+
|   HOSTILE_REVIEWER: REJECT                                          |
|                                                                     |
|   Artifact: W25.3 iOS/Mobile Compatibility Work                     |
|   Author: WASM_SPECIALIST                                           |
|                                                                     |
|   Critical Issues: 5                                                |
|   Major Issues: 8                                                   |
|   Minor Issues: 4                                                   |
|                                                                     |
|   Disposition:                                                      |
|   - REJECT: Multiple critical regressions detected                  |
|   - BLOCK: All navigation links broken on desktop                   |
|   - BLOCK: WASM filter parsing undefined on mobile                  |
|   - BLOCK: Benchmark values show NaN% on mobile                     |
|                                                                     |
+---------------------------------------------------------------------+
```

---

## FINDINGS

### CRITICAL (BLOCKING)

| ID | Issue | Platform | Location | Evidence |
|:---|:------|:---------|:---------|:---------|
| C1 | **Index navigation links non-clickable on DESKTOP** | Desktop | `index.html:*` | User report: "pages aren't connected to each other from the index" |
| C2 | **`parse_filter_js` is undefined on iOS Safari** | iOS | `filter-playground.html:1259` | `TypeError: wasmModule.parse_filter_js is not a function` |
| C3 | **Benchmark shows 0 values and +NaN% on mobile** | iOS | `benchmark-dashboard.html:1560-1577` | User report: "showing 0 as result and +nan%" |
| C4 | **Horizontal scroll breaking UI on iOS only** | iOS | `index.html` | User report: "displayed is broken horizontally MUST FIX on IOS only" |
| C5 | **Filter demo returns no results on mobile with same input** | iOS | `filter-playground.html` | User: "filter demo doesnt work with filters only on mobile on desktop it works" |

### MAJOR (MUST FIX)

| ID | Issue | Platform | Location | Evidence |
|:---|:------|:---------|:---------|:---------|
| M1 | Desktop links in index.html not working | Desktop | `index.html` | Navigation broken |
| M2 | WASM module path resolution fails silently | iOS | `filter-playground.html:1164-1181` | Falls through all paths without proper error |
| M3 | Export verification claims pass but function undefined | iOS | `filter-playground.html:1197-1210` | `parse_filter_js` in missingExports but passes |
| M4 | `get_filter_info_js` may not exist in WASM exports | iOS/Desktop | `filter-playground.html:1265-1269` | Called without existence check |
| M5 | Soft delete compact doesn't reset tombstones | iOS | User report | "DOESNT RESET THE TOMBSTONE IT LAGS" |
| M6 | High vector count causes lag (15k+ vectors) | iOS | `soft_delete.html` | User: "WITH 15K + VECTOR IT STARTS LAGGING" |
| M7 | Filter playground UI breaking on mobile | iOS | `filter-playground.html` | User: "UI MUST BE OPTIMIZED ASWELL IT HAS MANY ERROR" |
| M8 | Inconsistent behavior between platforms | Desktop/iOS | Multiple | Same input = different results |

### MINOR (SHOULD FIX)

| ID | Issue | Location | Impact |
|:---|:------|:---------|:-------|
| m1 | Version tag shows v0.3.0 in benchmark-dashboard | `benchmark-dashboard.html:1188` | Confusion |
| m2 | Footer version shows v0.5.0 in filter-playground | `filter-playground.html:998` | Outdated |
| m3 | Console logs in production code | Multiple files | Debug noise |
| m4 | `pkg/package.json` corruption persists | `pkg/package.json` | Blocks npm publish |

---

## ROOT CAUSE ANALYSIS

### C1: Desktop Navigation Broken

**Hypothesis:** The index.html file's navigation links may:
1. Be wrapped in elements with `pointer-events: none`
2. Have JavaScript event handlers preventing default
3. Be positioned under an overlay element (z-index issue)
4. Have broken `href` attributes

**Investigation Required:**
- Check `z-index` of `.bg-grid`, `.overlay`, or animated elements
- Verify `<a href>` links are not wrapped in disabled parent
- Check if JavaScript `preventDefault` is blocking clicks

**Location:** `index.html:*` - Need to trace why links work on mobile but not desktop

### C2: parse_filter_js undefined on iOS

**Root Cause:** The WASM module loads but the function binding is not included in the compiled output for the web target.

**Evidence:** User reported exact error:
```
TypeError: wasmModule.parse_filter_js is not a function.
(In 'wasmModule.parse_filter_js(filterStr)', 'wasmModule.parse_filter_js' is undefined)
```

**Likely Causes:**
1. `wasm-pack build --target web` not including all exports
2. Different WASM binary than expected
3. `pkg/edgevec.js` out of sync with `pkg/edgevec_bg.wasm`

### C3: Benchmark NaN% on Mobile

**Root Cause:** Division by zero or undefined value propagation.

**Location:** `benchmark-dashboard.html:1566`
```javascript
const overhead = ((filteredP50 - unfilteredP50) / unfilteredP50 * 100).toFixed(1);
```

**Issue:** If `unfilteredP50` is 0 or undefined, this produces `NaN` or `Infinity`.

**Fix Required:** Add null checks before division.

### C4: Horizontal Scroll on iOS

**Root Cause:** CSS `overflow-x: hidden` applied to wrong element or overridden.

**Evidence:** Works on desktop, breaks on iOS - indicates Safari-specific CSS interpretation.

### C5: Filter Demo Platform Discrepancy

**Root Cause:** Either:
1. Different WASM modules loaded on each platform
2. Path resolution differs (relative vs absolute)
3. Safari's stricter MIME type enforcement

---

## PLATFORM COMPARISON MATRIX

| Feature | Desktop | iOS Safari | Status |
|:--------|:--------|:-----------|:-------|
| Index Navigation | BROKEN | Works | INCONSISTENT |
| Filter Playground Load | Works | BROKEN | INCONSISTENT |
| Filter Parsing | Works | BROKEN | INCONSISTENT |
| Benchmark Values | Works | NaN% | INCONSISTENT |
| Horizontal Scroll | OK | BROKEN | INCONSISTENT |
| Soft Delete | Untested | Lag at 15k+ | DEGRADED |
| Theme/Styling | Works | Works | OK |
| Chart.js Render | Works | Works | OK |

---

## REQUIRED ACTIONS BEFORE RESUBMISSION

### Priority 1: Critical Fixes (Engineer MUST complete ALL)

1. **Fix Desktop Navigation (C1)**
   - Test: Click any demo link from index.html on desktop browser
   - Expected: Navigate to target page
   - Debug: Check z-index, pointer-events, event handlers

2. **Fix parse_filter_js Export (C2)**
   - Rebuild WASM: `wasm-pack build --target web --out-dir pkg`
   - Verify export: `grep "parse_filter_js" pkg/edgevec.js`
   - Test on iOS Safari after rebuild

3. **Fix Benchmark NaN (C3)**
   ```javascript
   const overhead = unfilteredP50 > 0
     ? ((filteredP50 - unfilteredP50) / unfilteredP50 * 100).toFixed(1)
     : '0.0';
   ```

4. **Fix iOS Horizontal Scroll (C4)**
   - Apply `overflow-x: hidden` to `html` and `body`
   - Test with iOS @supports query

5. **Ensure Platform Parity (C5)**
   - Same test input must produce same results on both platforms

### Priority 2: Major Fixes

6. Fix `get_filter_info_js` existence check
7. Add defensive null checks throughout benchmark code
8. Optimize soft delete for 15k+ vectors
9. Update all version tags to current version

### Priority 3: Verification

10. Create automated cross-platform test matrix
11. Document WASM build process for iOS compatibility
12. Test on multiple iOS devices (iPhone SE, iPhone 15)

---

## TESTING REQUIREMENTS BEFORE NEXT REVIEW

Engineer must complete ALL tests in the strict guide below and provide:

1. Screenshot of desktop index.html with working navigation
2. Console log from iOS Safari showing successful WASM load
3. Screenshot of benchmark with real values (not NaN%)
4. Filter playground working on iOS with complex filter

---

## HANDOFF

```
## HOSTILE_REVIEWER: Rejected

Artifact: W25.3 iOS/Mobile Compatibility Work
Status: REJECTED

Review Document: docs/reviews/2025-12-20_W25_DAY3_DISCREPANCY_REPORT_REJECTED.md

BLOCK: Next phase cannot proceed until issues resolved

Required Actions:
1. Fix desktop navigation (C1)
2. Fix parse_filter_js export for iOS (C2)
3. Fix benchmark NaN% (C3)
4. Fix iOS horizontal scroll (C4)
5. Ensure platform parity (C5)
6. Complete strict iOS test guide

Resubmit via: /review W25.3.4
```

---

**Reviewer:** HOSTILE_REVIEWER
**Kill Authority:** YES
**Verdict:** REJECT
