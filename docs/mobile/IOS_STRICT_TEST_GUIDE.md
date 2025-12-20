# STRICT iOS Safari Testing Guide

**Version:** 1.0.0
**Target:** v0.5.4 iOS Compatibility Validation
**Authority:** HOSTILE_REVIEWER
**Status:** MANDATORY

---

## OVERVIEW

This guide contains the **COMPLETE** set of tests required to validate EdgeVec works correctly on iOS Safari. ALL tests must pass before the work can be approved.

**Failure of ANY test = REJECT**

---

## PREREQUISITES

### Required Equipment

| Item | Requirement |
|:-----|:------------|
| iOS Device | iPhone 12+ or iPad (2020+) |
| iOS Version | 17.0 or higher |
| Browser | Safari (not Chrome/Firefox) |
| Network | Same WiFi as development machine |
| Mac (optional) | For Web Inspector debugging |

### Server Setup

On your development machine, start the HTTP server:

```bash
cd edgevec
python -m http.server 8080 --bind 0.0.0.0
```

Find your local IP:
```bash
# Windows
ipconfig

# macOS/Linux
ifconfig | grep "inet "
```

Your test URL will be: `http://<YOUR_IP>:8080/wasm/examples/`

---

## PART 1: DESKTOP REGRESSION TESTS (Do FIRST)

Before testing iOS, verify desktop still works.

### Test D1: Desktop Navigation

**URL:** `http://localhost:8080/wasm/examples/index.html`

| Step | Action | Expected | Pass/Fail |
|:-----|:-------|:---------|:----------|
| D1.1 | Load index.html | Page displays with all demo cards | [ ] |
| D1.2 | Click "Filter Playground" card | Navigates to filter-playground.html | [ ] |
| D1.3 | Click back, then "Benchmark Dashboard" | Navigates to benchmark-dashboard.html | [ ] |
| D1.4 | Click back, then "Soft Delete Demo" | Navigates to soft_delete.html | [ ] |
| D1.5 | Right-click any link, "Open in new tab" | Opens correct page | [ ] |

**IF ANY D1.x FAILS:** Desktop navigation is broken. Fix before proceeding.

### Test D2: Desktop Filter Playground

**URL:** `http://localhost:8080/wasm/examples/filter-playground.html`

| Step | Action | Expected | Pass/Fail |
|:-----|:-------|:---------|:----------|
| D2.1 | Wait for WASM load | "WASM module loaded" message appears | [ ] |
| D2.2 | Type: `category = "test"` | Parsed successfully, AST shown | [ ] |
| D2.3 | Type: `price > 100 AND rating >= 4.5` | Parsed successfully | [ ] |
| D2.4 | Type: `invalid!!!` | Error message with position indicator | [ ] |
| D2.5 | Click example button "Simple Equals" | Filter loads and parses | [ ] |

**Record all outputs for comparison with iOS.**

### Test D3: Desktop Benchmark Dashboard

**URL:** `http://localhost:8080/wasm/examples/benchmark-dashboard.html`

| Step | Action | Expected | Pass/Fail |
|:-----|:-------|:---------|:----------|
| D3.1 | Wait for page load | Charts render with actual data | [ ] |
| D3.2 | Check "Search Latency" values | Shows numeric ms values (not 0, not NaN) | [ ] |
| D3.3 | Click "Run Benchmark" in Filter section | Metrics show numeric values | [ ] |
| D3.4 | Check "Filter Overhead" | Shows percentage (e.g., "+15.2%", not "+NaN%") | [ ] |
| D3.5 | Scroll entire page | No layout breaks, all sections visible | [ ] |

---

## PART 2: iOS SAFARI TESTS (Core Validation)

### Test M1: iOS Demo Catalog

**URL:** `http://<YOUR_IP>:8080/wasm/examples/index.html`

| Step | Action | Expected | Pass/Fail |
|:-----|:-------|:---------|:----------|
| M1.1 | Load page | "EdgeVec" title visible | [ ] |
| M1.2 | Check horizontal scroll | NO horizontal scroll bar present | [ ] |
| M1.3 | Swipe left/right | Page should NOT scroll horizontally | [ ] |
| M1.4 | Count demo cards | At least 6 visible (scrolling ok) | [ ] |
| M1.5 | Tap "Filter Playground" card | Navigates to filter-playground.html | [ ] |
| M1.6 | Tap back, then "Benchmark Dashboard" | Navigates correctly | [ ] |
| M1.7 | Tap all other demo links | All navigate correctly | [ ] |
| M1.8 | Verify neon/cyberpunk styling | Animations smooth, colors correct | [ ] |

**CRITICAL:** M1.2 and M1.3 validate the horizontal scroll fix. Must pass.

### Test M2: iOS Filter Playground (CRITICAL)

**URL:** `http://<YOUR_IP>:8080/wasm/examples/filter-playground.html`

| Step | Action | Expected | Pass/Fail |
|:-----|:-------|:---------|:----------|
| M2.1 | Load page | "Filter Playground" visible | [ ] |
| M2.2 | Wait for WASM load | Green "WASM module loaded" message | [ ] |
| M2.3 | Check console* | "[EdgeVec]" log messages visible | [ ] |
| M2.4 | Tap input field | iOS keyboard appears | [ ] |
| M2.5 | Type: `category = "test"` | "Valid filter expression" status | [ ] |
| M2.6 | Check AST output | JSON tree with "field": "category" | [ ] |
| M2.7 | Clear and type: `price > 100` | Parses successfully | [ ] |
| M2.8 | Clear and type: `invalid!!!` | Error message appears | [ ] |
| M2.9 | Tap "Simple Equals" example | Filter loads and parses | [ ] |
| M2.10 | Tap "Complex Nested" example | Parses without error | [ ] |

**CRITICAL COMPARISON:**
- M2.5 result MUST match D2.2 result
- M2.6 JSON MUST be identical to desktop
- If WASM fails to load, record exact error message

*Console access requires Mac with Web Inspector connected.

### Test M3: iOS Benchmark Dashboard (CRITICAL)

**URL:** `http://<YOUR_IP>:8080/wasm/examples/benchmark-dashboard.html`

| Step | Action | Expected | Pass/Fail |
|:-----|:-------|:---------|:----------|
| M3.1 | Load page | Page loads without freeze | [ ] |
| M3.2 | Wait for charts | Chart.js graphs render | [ ] |
| M3.3 | Check hero stat values | Numeric values (not 0, not NaN) | [ ] |
| M3.4 | Scroll to Filter section | Section visible | [ ] |
| M3.5 | Tap "Run Benchmark" | Metrics populate | [ ] |
| M3.6 | Check "Parse Time" | Numeric ms value (e.g., "0.123ms") | [ ] |
| M3.7 | Check "Unfiltered Search" | Numeric ms value | [ ] |
| M3.8 | Check "Filtered Search" | Numeric ms value | [ ] |
| M3.9 | Check "Filter Overhead" | Percentage (NOT "+NaN%") | [ ] |
| M3.10 | Change vectors to 10,000 | Runs without crash | [ ] |

**CRITICAL:** M3.9 is the NaN% bug. Must show valid percentage.

### Test M4: iOS Soft Delete Demo

**URL:** `http://<YOUR_IP>:8080/wasm/examples/soft_delete.html`

| Step | Action | Expected | Pass/Fail |
|:-----|:-------|:---------|:----------|
| M4.1 | Load page | Demo interface visible | [ ] |
| M4.2 | Tap "Insert" | Vector count increases | [ ] |
| M4.3 | Insert 100 vectors | Count shows 100, no lag | [ ] |
| M4.4 | Tap "Delete" (delete some) | Tombstone count increases | [ ] |
| M4.5 | Tap "Compact" | Tombstones reset to 0 | [ ] |
| M4.6 | Tap "Search" | Results appear | [ ] |
| M4.7 | Insert 5,000 more vectors | May be slow but no freeze | [ ] |
| M4.8 | Insert to 15,000 total | Note performance (lag is expected) | [ ] |

**Note:** Lag at 15k+ vectors is known limitation, not a blocking issue.

---

## PART 3: CROSS-PLATFORM PARITY CHECKS

After completing Parts 1 and 2, verify identical behavior.

### Parity Test P1: Filter Output Match

1. On Desktop: Type `category = "books" AND price < 50`
2. Screenshot the parsed JSON output
3. On iOS: Type identical filter
4. Compare JSON output

| Check | Result |
|:------|:-------|
| JSON structure identical | [ ] |
| Field names identical | [ ] |
| Operators identical | [ ] |
| Values identical | [ ] |

### Parity Test P2: Error Message Match

1. On Desktop: Type `category = incomplete`
2. Record error message and position
3. On iOS: Type identical invalid filter
4. Compare error output

| Check | Result |
|:------|:-------|
| Error type matches | [ ] |
| Position indicator matches | [ ] |
| Suggestion (if any) matches | [ ] |

### Parity Test P3: Benchmark Value Sanity

Compare numeric ranges between platforms:

| Metric | Desktop Value | iOS Value | Within 3x? |
|:-------|:-------------|:----------|:-----------|
| Parse Time | ___ms | ___ms | [ ] |
| Unfiltered Search | ___ms | ___ms | [ ] |
| Filtered Search | ___ms | ___ms | [ ] |
| Overhead % | ___% | ___% | [ ] |

**Note:** iOS values may be slower but should be same order of magnitude.

---

## PART 4: STRESS TESTS (Optional but Recommended)

### Stress S1: Rapid Navigation

1. Quickly tap back and forth between index and filter playground
2. Do this 10 times
3. No crashes, no memory errors

| Result | Pass/Fail |
|:-------|:----------|
| No crashes | [ ] |
| Pages load each time | [ ] |

### Stress S2: Large Filter

Type this filter on iOS:
```
(category = "electronics" AND price > 100 AND price < 500) OR (category = "books" AND rating > 4.5 AND author CONTAINS "Smith") OR (category = "music" AND year > 2020)
```

| Result | Pass/Fail |
|:-------|:----------|
| Parses successfully | [ ] |
| No memory error | [ ] |
| Parse time < 100ms | [ ] |

### Stress S3: Orientation Change

1. Start in portrait
2. Rotate to landscape
3. Rotate back to portrait

| Check | Pass/Fail |
|:------|:----------|
| Layout adapts correctly | [ ] |
| No elements cut off | [ ] |
| Horizontal scroll doesn't appear | [ ] |

---

## PART 5: RESULTS COLLECTION

### Device Information

| Field | Value |
|:------|:------|
| Device Model | ________________ |
| iOS Version | ________________ |
| Safari Version | ________________ |
| Test Date | ________________ |
| Tester Name | ________________ |

### Test Summary

| Part | Tests Passed | Tests Failed | Notes |
|:-----|:-------------|:-------------|:------|
| D (Desktop) | ___/15 | ___/15 | |
| M1 (Catalog) | ___/8 | ___/8 | |
| M2 (Filter) | ___/10 | ___/10 | |
| M3 (Benchmark) | ___/10 | ___/10 | |
| M4 (Soft Delete) | ___/8 | ___/8 | |
| P (Parity) | ___/3 | ___/3 | |
| **TOTAL** | ___/54 | ___/54 | |

### Blocking Issues Found

List any FAILED tests that are blocking:

1. _______________________________________
2. _______________________________________
3. _______________________________________

### Screenshots Required

| Screenshot | Captured? |
|:-----------|:----------|
| Desktop index with all links visible | [ ] |
| iOS index without horizontal scroll | [ ] |
| iOS filter playground with valid parse | [ ] |
| iOS benchmark with numeric values (not NaN) | [ ] |
| Console logs showing [EdgeVec] messages | [ ] |

---

## APPROVAL CRITERIA

**For HOSTILE_REVIEWER approval, ALL of the following must be true:**

1. Desktop Tests (D1-D3): ALL PASS
2. iOS Core Tests (M1-M3): ALL PASS
3. Parity Tests (P1-P3): ALL PASS
4. No horizontal scroll on iOS
5. parse_filter_js works on iOS
6. Benchmark shows real values (not NaN%)

**Any failure = REJECT. No exceptions.**

---

## AFTER TESTING

1. Fill out all sections above
2. Take required screenshots
3. Submit to HOSTILE_REVIEWER for final approval
4. If approved, proceed with version bump to v0.5.4

---

**Document:** IOS_STRICT_TEST_GUIDE.md
**Authority:** HOSTILE_REVIEWER
**Version:** 1.0.0
