# Week 14 — Day 2 Tasks (Tuesday, Dec 24)

**Date:** 2025-12-24
**Focus:** Complete P99 CI + Start Competitive Benchmarks
**Agent:** BENCHMARK_SCIENTIST
**Status:** [REVISED]

---

## Day Objective

Complete P99 latency tracking CI setup and begin executing competitive benchmarks with actual performance data.

**Success Criteria:**
- Regression detection script complete
- CI workflow tested
- EdgeVec competitive benchmark running

---

## Tasks

### W14.2 (Part 2): P99 CI Completion

**Priority:** P0 (Critical Path)
**Estimate:** 6h (remaining from 8h total)
**Agent:** BENCHMARK_SCIENTIST

#### Scope

- [ ] **AC14.2.3:** Complete regression detection script
- [ ] **AC14.2.4:** Verify regression >10% fails CI
- [ ] **AC14.2.5:** Results uploaded as artifact
- [ ] **AC14.2.6:** Add README badge

#### Implementation Specification

**Regression Detection Script:**

```python
#!/usr/bin/env python3
# scripts/check_regression.py
"""
Check benchmark results for performance regressions.
Fails if P99 latency exceeds baseline by threshold percentage.
"""

import json
import sys
import os
from pathlib import Path

def load_baselines(path: str) -> dict:
    """Load baseline thresholds from JSON."""
    with open(path) as f:
        return json.load(f)

def parse_criterion_results(criterion_dir: str) -> dict:
    """Parse Criterion benchmark results."""
    results = {}
    criterion_path = Path(criterion_dir)

    for bench_dir in criterion_path.iterdir():
        if not bench_dir.is_dir():
            continue

        estimate_file = bench_dir / "new" / "estimates.json"
        if estimate_file.exists():
            with open(estimate_file) as f:
                data = json.load(f)
                # Extract mean and standard deviation
                mean_ns = data.get("mean", {}).get("point_estimate", 0)
                std_ns = data.get("std_dev", {}).get("point_estimate", 0)

                # Calculate P99 estimate (mean + 2.33 * std_dev)
                p99_ns = mean_ns + 2.33 * std_ns

                results[bench_dir.name] = {
                    "mean_us": mean_ns / 1000,
                    "p99_us": p99_ns / 1000,
                }

    return results

def check_regressions(baselines: dict, results: dict, threshold_multiplier: float) -> list:
    """Compare results against baselines, return list of failures."""
    failures = []

    for bench_name, baseline in baselines.get("benchmarks", {}).items():
        if bench_name not in results:
            print(f"WARNING: No results for {bench_name}")
            continue

        result = results[bench_name]

        # Check P99 latency
        if "p99" in baseline:
            baseline_p99 = baseline["p99"]
            actual_p99 = result["p99_us"]
            threshold = baseline_p99 * threshold_multiplier

            pct_change = (actual_p99 - baseline_p99) / baseline_p99 * 100 if baseline_p99 > 0 else 0

            status = "PASS" if actual_p99 <= threshold else "FAIL"
            print(f"{bench_name}: P99 = {actual_p99:.2f}{baseline['unit']} "
                  f"(baseline: {baseline_p99}{baseline['unit']}, "
                  f"threshold: {threshold:.2f}{baseline['unit']}, "
                  f"change: {pct_change:+.1f}%) [{status}]")

            if actual_p99 > threshold:
                failures.append({
                    "benchmark": bench_name,
                    "metric": "p99",
                    "baseline": baseline_p99,
                    "threshold": threshold,
                    "actual": actual_p99,
                    "change_pct": pct_change,
                    "unit": baseline["unit"],
                })

    return failures

def main():
    baselines_path = "benches/baselines.json"
    criterion_dir = "target/criterion"

    if not os.path.exists(baselines_path):
        print(f"ERROR: Baselines file not found: {baselines_path}")
        sys.exit(1)

    if not os.path.exists(criterion_dir):
        print(f"WARNING: Criterion results not found: {criterion_dir}")
        print("Skipping regression check (no benchmark results)")
        sys.exit(0)

    baselines = load_baselines(baselines_path)
    threshold_multiplier = baselines.get("thresholds", {}).get("regression_multiplier", 1.1)

    results = parse_criterion_results(criterion_dir)

    print("=" * 60)
    print("BENCHMARK REGRESSION CHECK")
    print(f"Threshold: {(threshold_multiplier - 1) * 100:.0f}% above baseline")
    print("=" * 60)

    failures = check_regressions(baselines, results, threshold_multiplier)

    if failures:
        print("\n" + "=" * 60)
        print("REGRESSIONS DETECTED!")
        print("=" * 60)
        for f in failures:
            print(f"  FAIL: {f['benchmark']} {f['metric']}")
            print(f"        Baseline: {f['baseline']}{f['unit']}")
            print(f"        Threshold: {f['threshold']:.2f}{f['unit']}")
            print(f"        Actual: {f['actual']:.2f}{f['unit']}")
            print(f"        Change: {f['change_pct']:+.1f}%")
        sys.exit(1)
    else:
        print("\n" + "=" * 60)
        print("ALL BENCHMARKS PASSED")
        print("=" * 60)
        sys.exit(0)

if __name__ == "__main__":
    main()
```

**README Badge:**

```markdown
<!-- Add to README.md after status badges -->
[![Performance](https://github.com/matte1782/edgevec/actions/workflows/benchmark.yml/badge.svg)](https://github.com/matte1782/edgevec/actions/workflows/benchmark.yml)
```

#### Verification Commands

```bash
# Test script locally (after running benchmarks)
cargo bench --bench search_bench -- --noplot
python scripts/check_regression.py

# Test synthetic regression (AC14.2.4):
# 1. Backup baselines
cp benches/baselines.json benches/baselines.json.bak

# 2. Set artificially low threshold
cat > benches/baselines.json << 'EOF'
{
  "version": "1.0.0-test",
  "thresholds": { "regression_multiplier": 1.1 },
  "benchmarks": {
    "search_10k": { "p99": 0.001, "unit": "ms" }
  }
}
EOF

# 3. Run check (should FAIL)
python scripts/check_regression.py
echo "Exit code: $?"  # Should be 1

# 4. Restore baselines
mv benches/baselines.json.bak benches/baselines.json
```

---

### W14.3 (Part 1): Competitive Benchmarks Setup

**Priority:** P1
**Estimate:** 1h (start)
**Agent:** BENCHMARK_SCIENTIST

#### Scope

- [ ] **AC14.3.1:** Verify EdgeVec benchmark runs

#### Verification Commands

```bash
cd benches/competitive

# Install dependencies (if not done)
npm install

# Test EdgeVec adapter
node -e "const {EdgeVecAdapter} = require('./adapters/edgevec.js'); console.log('EdgeVec OK')"

# Run EdgeVec-only benchmark
node harness.js --library=edgevec --vectors=1000
```

---

## Day 2 Summary

**Total Effort:** 7h scheduled

**Deliverables:**
1. Complete `scripts/check_regression.py`
2. README badge added
3. CI workflow tested with synthetic regression
4. EdgeVec benchmark verified

**Carryover to Day 3:**
- Install and test competitor adapters
- Run full competitive benchmark suite

---

## HOSTILE_REVIEWER Pre-Flight (Day 2)

Before end of day:

- [ ] `check_regression.py` exits 1 on >10% regression (tested)
- [ ] CI workflow syntax valid
- [ ] Badge URL responsive
- [ ] EdgeVec adapter runs successfully

---

**Status:** [REVISED]
**Next:** Complete W14.3 competitive benchmarks
