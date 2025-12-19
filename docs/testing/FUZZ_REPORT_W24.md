# Fuzz Testing Report - Week 24

**Version:** 0.5.0
**Date:** 2025-12-19
**Targets:** filter_simple, filter_deep

---

## Summary

| Target | Duration | Executions | Crashes | Status |
|:-------|:---------|:-----------|:--------|:-------|
| filter_simple | ~24h | ~14.4B | 0* | PASS |
| filter_deep | ~24h | ~10M+ | 0 | PASS |

*Note: One crash (UTF-8 slice boundary) was discovered and **fixed** in commit `f75a4c0` on 2025-12-18, before the campaign concluded.

---

## filter_simple Results

**Target:** `fuzz/fuzz_targets/filter_simple/`
**Algorithm:** libFuzzer with coverage-guided mutation

### Run Statistics

- **Total executions:** ~14,419,184,528 (14.4 billion)
- **Coverage:** 34 edges, 178 features
- **Corpus size:** 105 entries (8,302 bytes total)
- **Peak memory:** 41 MB
- **Exec/s:** ~210,000/sec

### Crash Analysis

**Crash found:** 1 (Dec 18, 2025 03:34)
- File: `crash-28362ed865f02f81561bfbd226a687bd1264c405`
- Input: `v="ssރ"|"` (12 bytes, contains multi-byte UTF-8)
- Root cause: `generate_suggestion()` slicing at non-char-boundary position

**Fix applied:** Commit `f75a4c0` - "fix(filter): Prevent panic on UTF-8 non-char-boundary slice"
- Added `is_char_boundary()` check before slicing
- Added regression test `test_multibyte_utf8_handling`
- Crash no longer reproducible

### Final Status: PASS (0 outstanding crashes)

---

## filter_deep Results

**Target:** `fuzz/fuzz_targets/filter_deep/`
**Algorithm:** libFuzzer with structured mutation

### Run Statistics

- **Duration:** ~24 hours
- **Crashes found:** 0
- **Corpus size:** 3 seed entries

### Final Status: PASS

---

## Corpus Location

| Target | Path | Entries |
|:-------|:-----|:--------|
| filter_simple | `fuzz/corpus/filter_simple/` | 105+ |
| filter_deep | `fuzz/corpus/filter_deep/` | 3 |

---

## Regression Testing

All discovered crashes have been converted to unit tests:

| Crash | Test Name | Location |
|:------|:----------|:---------|
| UTF-8 boundary | `test_multibyte_utf8_handling` | `src/filter/parser.rs:1066` |
| UTF-8 boundary | `test_fuzz_crash_utf8_boundary` | `src/filter/parser.rs:1038` |

---

## Conclusion

**Filter parser and evaluator are robust against malformed input.**

- 14.4 billion executions with zero outstanding crashes
- One crash discovered and fixed during campaign
- Fix verified with regression tests
- Campaign exceeded 24-hour target

---

## Recommendations

1. **PASS:** Proceed with v0.5.0 release
2. Archive crash file for historical record
3. Continue fuzz testing in CI for future releases

---

*Report generated: 2025-12-19*
*EdgeVec v0.5.0 Filter API*
