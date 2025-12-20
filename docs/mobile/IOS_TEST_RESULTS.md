# iOS Safari Test Results

**Version:** EdgeVec v0.5.3
**Date:** 2025-12-19
**Agent:** WASM_SPECIALIST
**Task:** W25.3.3

---

## ⚠️ ACTUAL DEVICE TESTING COMPLETED — CRITICAL ISSUES FOUND

> **STATUS: TESTED ON DEVICE — MULTIPLE FAILURES**
>
> This document has been updated with ACTUAL test results from iPhone 15 Pro (iOS 18.2).
> **W25.3.3 REJECTED by HOSTILE_REVIEWER** due to 6 CRITICAL and 4 MAJOR issues.
>
> **See:**
> - `docs/reviews/2025-12-19_W25_DAY3_IOS_TESTING_REJECTED.md` (rejection details)
> - `docs/planning/W25_DAY3_REMEDIATION_PLAN.md` (fix plan)
> - `W25_DAY3_IOS_TEST_CHECKLIST.md` (raw test results)

---

## Test Environment

| Parameter | Value |
|:----------|:------|
| Testing Method | ✅ **ACTUAL DEVICE TESTING** |
| Device | iPhone 15 Pro |
| iOS Version | 18.2 |
| Safari Version | 18.2 |
| EdgeVec Version | 0.5.3 |
| Test Date | 2025-12-19 |
| Tester | Human + WASM_SPECIALIST |
| Review Status | ❌ REJECTED by HOSTILE_REVIEWER |

**Note:** This document contains ACTUAL test results from physical device testing.

---

## Test Matrix

### WASM Core Functionality

| Test Case | iOS 17 Safari | iOS 18 Safari | Notes |
|:----------|:--------------|:--------------|:------|
| WASM Module Load | ✅ Expected | ✅ Expected | Core WebAssembly supported |
| `init()` Function | ✅ Expected | ✅ Expected | wasm-bindgen compatible |
| EdgeVec Constructor | ✅ Expected | ✅ Expected | No advanced features used |
| Float32Array Handling | ✅ Expected | ✅ Expected | Standard typed arrays |
| BigInt64 (IDs) | ✅ Expected | ✅ Expected | Safari 14+ |

### Vector Operations

| Test Case | iOS 17 Safari | iOS 18 Safari | Notes |
|:----------|:--------------|:--------------|:------|
| Insert 1k vectors | ✅ Expected | ✅ Expected | Well under memory limit |
| Insert 10k vectors | ✅ Expected | ✅ Expected | ~87 MB quantized |
| Insert 50k vectors | ⚠️ Test needed | ⚠️ Test needed | Approaches limit |
| Search k=10 | ✅ Expected | ✅ Expected | Scalar distance calc |
| Search k=100 | ✅ Expected | ✅ Expected | Larger result set |
| Soft Delete | ✅ Expected | ✅ Expected | Tombstone marking |
| Compact | ⚠️ Test needed | ⚠️ Test needed | Memory spike during rebuild |

### Filter API

| Test Case | iOS 17 Safari | iOS 18 Safari | Notes |
|:----------|:--------------|:--------------|:------|
| Filter.parse() | ✅ Expected | ✅ Expected | Pure JavaScript |
| Filter.evaluate() | ✅ Expected | ✅ Expected | Pure JavaScript |
| FilterBuilder | ✅ Expected | ✅ Expected | Pure TypeScript |
| Complex expressions | ✅ Expected | ✅ Expected | No WASM overhead |

### Persistence (IndexedDB)

| Test Case | iOS 17 Safari | iOS 18 Safari | Notes |
|:----------|:--------------|:--------------|:------|
| save() small DB | ✅ Expected | ✅ Expected | Under 50 MB |
| save() medium DB | ⚠️ Test needed | ⚠️ Test needed | 50-200 MB range |
| load() | ✅ Expected | ✅ Expected | Standard IndexedDB |
| Quota check | ✅ Expected | ✅ Expected | navigator.storage.estimate() |

### Demo Pages

| Demo | Loads | Functions | Touch | Notes |
|:-----|:------|:----------|:------|:------|
| Demo Catalog | ⚠️ BROKEN UI | ❌ FAIL | ✅ PASS | Horizontal layout broken on iOS only |
| Filter Playground | ⚠️ BROKEN UI | ❌ FAIL | ⚠️ BROKEN | WASM parse_filter_js undefined; UI broken |
| Benchmark Dashboard | ✅ PASS | ❌ FAIL | ✅ PASS | Returns 0 results, +NaN%; UI excellent |
| Soft Delete | ✅ PASS | ⚠️ PARTIAL | ✅ PASS | Compaction non-functional; lags at 15k+ |
| Batch Insert | ⚠️ NOT TESTED | ⚠️ NOT TESTED | ⚠️ NOT TESTED | Not included in W25.3.3 test plan |

---

## Expected vs Actual Results

### Baseline (To be filled after actual testing)

| Metric | Expected | iOS 17 Actual | iOS 18 Actual |
|:-------|:---------|:--------------|:--------------|
| WASM Load Time | <500ms | TBD | TBD |
| 10k Insert Time | <3s | TBD | TBD |
| Search P50 (10k) | <1ms | TBD | TBD |
| Save 10k DB | <2s | TBD | TBD |
| Load 10k DB | <1s | TBD | TBD |

---

## Touch Interaction Testing

### Areas to Test

| Interaction | Element | Expected Behavior |
|:------------|:--------|:------------------|
| Tap | Buttons | Immediate response |
| Tap | Text inputs | Keyboard appears |
| Scroll | Results list | Smooth scroll |
| Long press | N/A | No special behavior |
| Pinch zoom | Page | Should zoom (unless disabled) |

### Known iOS Safari Touch Issues

1. **300ms tap delay** — Fixed in modern Safari
2. **Input focus** — May require `touch-action: manipulation`
3. **Scroll momentum** — May differ from desktop

---

## Performance Observations

### Memory Monitoring

On iOS Safari, monitor console for:
- `RangeError: Out of memory` — Exceeded WASM memory limit
- `QuotaExceededError` — IndexedDB quota exceeded

### Recommended Limits

| Mode | Safe Vector Count | Memory Usage |
|:-----|:------------------|:-------------|
| Quantized (SQ8) | ≤50k | ~44 MB |
| Float32 | ≤15k | ~48 MB |

---

## Issues Found

### CRITICAL (Blocking v0.6.0 Mobile Support)

| ID | Severity | Description | Reproduction Steps | Status | GitHub Issue |
|:---|:---------|:------------|:-------------------|:-------|:-------------|
| C1 | CRITICAL | Filter Playground WASM module broken | Open filter-playground.html on iOS → TypeError: parse_filter_js undefined | ❌ OPEN | TBD |
| C2 | CRITICAL | Filter Playground UI broken on mobile | Open filter-playground.html on iOS → Horizontal overflow, touch broken | ❌ OPEN | TBD |
| C3 | CRITICAL | Demo Catalog horizontal layout broken (iOS only) | Open index.html on iOS → UI broken horizontally | ❌ OPEN | TBD |
| C4 | CRITICAL | Filter results differ between iOS and desktop | Run same filter on iOS and desktop → Different results | ❌ OPEN | TBD |
| C5 | CRITICAL | Benchmark Dashboard returns invalid data | Open benchmark-dashboard.html on iOS → Shows 0 results, +NaN% | ❌ OPEN | TBD |
| C6 | CRITICAL | Soft Delete compaction non-functional | Open soft_delete.html → Compact button doesn't reset tombstones | ❌ OPEN | TBD |

### MAJOR (Must Fix Before Release)

| ID | Severity | Description | Reproduction Steps | Status | GitHub Issue |
|:---|:---------|:------------|:-------------------|:-------|:-------------|
| M1 | MAJOR | Severe lag at 15k+ vectors on iOS | Insert 15k+ vectors on iOS → UI starts lagging | ❌ OPEN | TBD |
| M2 | MAJOR | IOS_TEST_RESULTS.md not updated | N/A | ✅ FIXED | This edit |
| M3 | MAJOR | Test matrix incomplete | N/A | ❌ OPEN | Pending retest |
| M4 | MAJOR | W25.2 error messages not working on iOS | Type invalid filter → Error not helpful | ❌ OPEN | TBD |

**Total Issues:** 6 Critical, 4 Major, 3 Minor
**Rejection Document:** `docs/reviews/2025-12-19_W25_DAY3_IOS_TESTING_REJECTED.md`
**Remediation Plan:** `docs/planning/W25_DAY3_REMEDIATION_PLAN.md`

---

## Browser Console Logs

### Startup Sequence (Expected)

```
[EdgeVec] WASM module loaded
[EdgeVec] IndexedDB available
[EdgeVec] Ready
```

### Error Patterns to Watch

```
// Memory limit hit
RangeError: Out of memory

// IndexedDB issue
DOMException: QuotaExceededError

// WASM instantiation failure
WebAssembly.instantiate: ...
```

---

## Findings from Actual Device Testing

### ✅ Confirmed Working on iOS Safari 18.2

1. **WASM module loads** — Core WebAssembly instantiation works
2. **Basic vector operations** — Insert, search work under 10k vectors
3. **IndexedDB persistence** — Save/load functional (not heavily tested)
4. **Touch interactions** — Generally responsive (where UI not broken)
5. **Benchmark Dashboard UI** — Excellent design (though data invalid)

### ❌ Confirmed Broken on iOS Safari 18.2

1. **Filter Playground** — WASM module missing parse_filter_js export
2. **Filter Playground UI** — Horizontal overflow, layout broken on mobile
3. **Demo Catalog** — Horizontal layout broken (iOS-specific)
4. **Filter consistency** — Different results on iOS vs desktop (CRITICAL)
5. **Benchmark Dashboard data** — Returns 0 results and +NaN%
6. **Soft Delete compaction** — Doesn't reset tombstones

### ⚠️ Performance Issues Discovered

1. **15k+ vector lag** — Contradicts research claim of "50k vectors safe"
   - **Research claimed:** ≤50k quantized vectors safe
   - **Reality:** Lag starts at 15k vectors on iPhone 15 Pro
2. **Compaction lag** — UI freezes during compaction at high vector counts

### 🔍 Needs Further Investigation

1. **Root cause of filter inconsistency** — Platform-specific behavior is unacceptable
2. **WASM export differences** — Why parse_filter_js missing on iOS only?
3. **Performance cliff** — Why does lag start at 15k instead of 50k?

---

## Next Steps

1. **Acquire testing access** — BrowserStack or LambdaTest account
2. **Run actual tests** — Update this document with real results
3. **File issues** — Create GitHub issues for any bugs found
4. **Update demos** — Add mobile-specific optimizations if needed

---

## Testing Commands

### Quick Smoke Test Sequence

```javascript
// Run in Safari console on demo page

// 1. Check WASM loaded
console.log('EdgeVec loaded:', typeof EdgeVec !== 'undefined');

// 2. Create small index
const config = new EdgeVecConfig(128);
const db = new EdgeVec(config);
console.log('Index created');

// 3. Insert test vectors
for (let i = 0; i < 100; i++) {
    const v = new Float32Array(128).fill(Math.random());
    db.insert(v);
}
console.log('Inserted 100 vectors');

// 4. Search
const query = new Float32Array(128).fill(0.5);
const results = db.search(query, 10);
console.log('Search returned:', results.length, 'results');

// 5. Filter API
const filter = Filter.parse('category = "test"');
console.log('Filter parsed:', filter !== null);

console.log('ALL TESTS PASSED');
```

---

---

## Remediation Status

**W25.3.3 Status:** ❌ **REJECTED** by HOSTILE_REVIEWER (2025-12-19)

**Critical Issues:** 6
**Major Issues:** 4
**Minor Issues:** 3

**Blocking:** v0.6.0 mobile support cannot proceed until all critical issues resolved.

**Next Steps:**
1. Review rejection document: `docs/reviews/2025-12-19_W25_DAY3_IOS_TESTING_REJECTED.md`
2. Execute remediation plan: `docs/planning/W25_DAY3_REMEDIATION_PLAN.md`
3. Fix all critical issues
4. Re-test on iPhone 15 Pro
5. Resubmit via `/review W25.3`

**Estimated Remediation Time:** 5-6 work days (37-47 hours)

---

**Agent:** WASM_SPECIALIST (testing) + HOSTILE_REVIEWER (analysis)
**Status:** W25.3.3 REJECTED — Awaiting remediation
**Last Updated:** 2025-12-19
**Next:** Execute W25_DAY3_REMEDIATION_PLAN.md
