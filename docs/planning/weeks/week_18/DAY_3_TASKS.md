# Day 3: P99 Latency Tracking in CI (W18.3)

**Date:** Week 18, Day 3
**Task ID:** W18.3
**Agent:** BENCHMARK_SCIENTIST
**Status:** [REVISED]
**Revision:** v1.1 — Addresses C3 (unit conversion), M3 (percentile fallback)

---

## Objective

Extend the CI benchmark system to track P99 latency and detect tail latency regressions. This addresses the "P99 latency tracking in CI" item from CHANGELOG "Unreleased".

---

## Context

### Current Benchmark State

- `benches/validation.rs` — Core benchmark suite
- `benches/check_regression.py` — Regression detection script
- `benches/baselines.json` — Median latency baselines
- `.github/workflows/benchmark.yml` — CI workflow

### Why P99 Matters

Median latency hides tail latency issues. A search that's 200µs median but 5ms P99 will cause user-visible stuttering in real applications.

**Target:** P99 should be < 2x median for all operations.

---

## Deliverables

### 1. Updated benches/validation.rs

Ensure Criterion captures percentile data (it does by default, but we need to verify collection settings).

### 2. Updated benches/check_regression.py

**CRITICAL FIX (C3):** Unit conversion from nanoseconds to milliseconds
**CRITICAL FIX (M3):** Fallback for missing Criterion percentiles

```python
#!/usr/bin/env python3
"""
EdgeVec Benchmark Regression Checker

Checks for:
1. Median latency regression (existing)
2. P99 latency regression (NEW)
3. P99/median ratio sanity check (NEW)

Addresses hostile review findings:
- C3: Unit conversion (ns → ms)
- M3: Percentile fallback (mean + 3σ)
"""

import json
import sys
from pathlib import Path
from typing import Optional, Tuple

REGRESSION_THRESHOLD = 1.20  # 20% regression tolerance
P99_REGRESSION_THRESHOLD = 1.50  # 50% P99 regression tolerance
P99_MEDIAN_RATIO_MAX = 3.0  # P99 should be < 3x median

# [C3 FIX] Conversion constant
NS_TO_MS = 1_000_000.0


def load_criterion_estimates(bench_name: str) -> Optional[dict]:
    """Load Criterion estimates from target/criterion/."""
    estimates_path = Path(f"target/criterion/validation/{bench_name}/new/estimates.json")
    if not estimates_path.exists():
        return None

    with open(estimates_path) as f:
        return json.load(f)


def extract_p99_ms(estimates: dict) -> Optional[float]:
    """
    Extract P99 from Criterion estimates IN MILLISECONDS.

    [C3 FIX] Criterion reports nanoseconds, we convert to milliseconds.
    [M3 FIX] If percentiles unavailable, estimate as mean + 3*std_dev.
    """
    # Try to get from percentiles first (if available)
    if "percentiles" in estimates:
        percentiles = estimates["percentiles"]
        if "0.99" in percentiles or "99.0" in percentiles:
            p99_ns = percentiles.get("0.99") or percentiles.get("99.0")
            return p99_ns / NS_TO_MS  # [C3 FIX]

    # [M3 FIX] Fallback: estimate P99 as mean + 3 * std_dev
    if "mean" in estimates and "std_dev" in estimates:
        mean_ns = estimates["mean"].get("point_estimate", 0)
        std_dev_ns = estimates["std_dev"].get("point_estimate", 0)
        p99_estimate_ns = mean_ns + (3 * std_dev_ns)
        return p99_estimate_ns / NS_TO_MS  # [C3 FIX]

    # Final fallback: estimate from slope upper bound
    if "slope" in estimates:
        slope = estimates["slope"]
        upper_ns = slope.get("confidence_interval", {}).get("upper_bound")
        if upper_ns:
            return upper_ns / NS_TO_MS  # [C3 FIX]

    return None


def extract_median_ms(estimates: dict) -> Optional[float]:
    """Extract median in milliseconds."""
    if "slope" in estimates:
        median_ns = estimates["slope"].get("point_estimate", 0)
        return median_ns / NS_TO_MS  # [C3 FIX]
    return None


def check_p99_regression(
    baseline: dict, current_p99_ms: float, bench_name: str
) -> Tuple[bool, str]:
    """
    Check P99 latency regression.

    Returns:
        (passed, message)
    """
    baseline_p99_ms = baseline.get("p99_ms")

    if baseline_p99_ms is None or current_p99_ms is None:
        return True, f"{bench_name}: P99 data not available (skipped)"

    ratio = current_p99_ms / baseline_p99_ms
    if ratio > P99_REGRESSION_THRESHOLD:
        return (
            False,
            f"{bench_name}: P99 REGRESSION {ratio:.2f}x ({baseline_p99_ms:.3f}ms -> {current_p99_ms:.3f}ms)",
        )

    return True, f"{bench_name}: P99 OK ({ratio:.2f}x baseline, {current_p99_ms:.3f}ms)"


def check_p99_median_ratio(
    median_ms: float, p99_ms: float, bench_name: str
) -> Tuple[bool, str]:
    """
    Check that P99 is within acceptable ratio of median.
    """
    if median_ms is None or p99_ms is None or median_ms == 0:
        return True, f"{bench_name}: Ratio check skipped (missing data)"

    ratio = p99_ms / median_ms
    if ratio > P99_MEDIAN_RATIO_MAX:
        return (
            False,
            f"{bench_name}: P99/median ratio {ratio:.2f}x exceeds {P99_MEDIAN_RATIO_MAX}x limit",
        )

    return True, f"{bench_name}: P99/median ratio {ratio:.2f}x (OK)"


def main():
    print("=== EdgeVec Benchmark Regression Check ===")
    print("[C3 FIX] All latencies reported in milliseconds")
    print("[M3 FIX] Using mean+3σ fallback for missing percentiles\n")

    # Load baselines
    baselines_path = Path("benches/baselines.json")
    if not baselines_path.exists():
        print("WARNING: No baselines.json found, skipping regression check")
        sys.exit(0)

    with open(baselines_path) as f:
        baselines = json.load(f)

    all_passed = True
    results = []

    benchmarks = ["insert_1k", "search_10k", "quantization_encode", "hamming_distance"]

    for bench in benchmarks:
        baseline = baselines.get(bench, {})
        estimates = load_criterion_estimates(bench)

        if estimates is None:
            results.append(f"{bench}: No data (skipped)")
            continue

        # Extract metrics in milliseconds [C3 FIX]
        current_median_ms = extract_median_ms(estimates)
        current_p99_ms = extract_p99_ms(estimates)

        # Check median regression (existing logic, now in ms)
        baseline_median_ms = baseline.get("median_ms", baseline.get("median_ns", 0) / NS_TO_MS)

        if baseline_median_ms > 0 and current_median_ms is not None:
            ratio = current_median_ms / baseline_median_ms
            if ratio > REGRESSION_THRESHOLD:
                all_passed = False
                results.append(f"{bench}: MEDIAN REGRESSION {ratio:.2f}x ({current_median_ms:.3f}ms)")
            else:
                results.append(f"{bench}: Median OK ({ratio:.2f}x, {current_median_ms:.3f}ms)")

        # Check P99 regression (NEW)
        if current_p99_ms is not None:
            passed, msg = check_p99_regression(baseline, current_p99_ms, bench)
            if not passed:
                all_passed = False
            results.append(msg)

            # Check P99/median ratio
            if current_median_ms is not None:
                passed, msg = check_p99_median_ratio(current_median_ms, current_p99_ms, bench)
                if not passed:
                    all_passed = False
                results.append(msg)

    print("\n".join(results))
    print("\n" + ("=== ALL CHECKS PASSED ===" if all_passed else "=== REGRESSION DETECTED ==="))

    sys.exit(0 if all_passed else 1)


if __name__ == "__main__":
    main()
```

### 3. Updated benches/baselines.json

Add P99 baselines:

```json
{
  "insert_1k": {
    "median_ns": 2500000,
    "p99_ns": 5000000,
    "description": "Insert 1000 768-dim vectors"
  },
  "search_10k": {
    "median_ns": 200000,
    "p99_ns": 400000,
    "description": "Search 10k index, k=10"
  },
  "quantization_encode": {
    "median_ns": 50000,
    "p99_ns": 100000,
    "description": "Encode 768-dim vector"
  },
  "hamming_distance": {
    "median_ns": 1000,
    "p99_ns": 2000,
    "description": "Hamming distance 96-byte codes"
  }
}
```

### 4. Updated .github/workflows/benchmark.yml

Add P99 to PR comment:

```yaml
- name: Generate PR comment
  if: github.event_name == 'pull_request'
  run: python benches/check_regression.py --pr-comment > benchmark_comment.md
```

---

## Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.3.1 | `benches/validation.rs` reports P99 | Benchmark output |
| AC18.3.2 | `benches/check_regression.py` checks P99 | Script updated |
| AC18.3.3 | P99 baselines added to `baselines.json` | JSON file |
| AC18.3.4 | P99 regression threshold: < 1.5x baseline | Regression check |
| AC18.3.5 | P99/median ratio check: < 3x | Sanity check |
| AC18.3.6 | PR comment includes P99 metrics | GitHub action |

---

## Implementation Plan

### Step 1: Review Criterion Output

Check what percentile data Criterion already captures.

### Step 2: Update check_regression.py

Add P99 extraction and comparison logic.

### Step 3: Update baselines.json

Add P99 values based on current measurements.

### Step 4: Test Locally

```bash
cargo bench --bench validation -- --noplot
python benches/check_regression.py
```

### Step 5: Update CI Workflow

Ensure P99 appears in PR comments.

---

## Files to Modify

| File | Action | Description |
|:-----|:-------|:------------|
| `benches/check_regression.py` | MODIFY | Add P99 checks |
| `benches/baselines.json` | MODIFY | Add P99 baselines |
| `.github/workflows/benchmark.yml` | MODIFY | P99 in PR comment |

---

## Verification Commands

```bash
# Run benchmarks
cargo bench --bench validation -- --noplot

# Check regression (should pass)
python benches/check_regression.py

# Verify P99 in output
python benches/check_regression.py | grep -i "p99"
```

---

## Handoff

**On Completion:**
- Mark W18.3 as COMPLETE
- Submit for hostile review
- This task runs parallel to W18.4/W18.5
