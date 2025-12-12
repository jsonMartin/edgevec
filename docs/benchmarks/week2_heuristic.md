# Week 2 Heuristic Selection Baseline Report

**Date:** 2025-12-06
**Benchmark:** W2.2 Heuristic Neighbor Selection (Fixed)
**Environment:** Local Dev Machine (Windows x64)
**Metadata:**
- **CPU:** Intel Core Ultra 9 285H
- **RAM:** High-End Mobile Workstation
- **OS:** Microsoft Windows NT 10.0.26200 (Windows 11)
- **Rust:** 1.90.0

## 1. Summary

This report establishes the true baseline for Heuristic Neighbor Selection by comparing it against a "Simple" selection mode (Top-M, no diversity check).

- **Simple Selection (64 candidates):** 470 ns
- **Heuristic Selection (64 candidates):** 6,560 ns (6.56 µs)
- **True Overhead:** ~14x
- **Constraint Check:** ❌ The <5x overhead constraint is violated for this specific component.

**Analysis:** The high overhead is inherent. Simple selection performs **zero** distance calculations (reusing heap distances), while Heuristic selection performs $O(M \cdot C)$ full vector distance calculations to ensure diversity. While the ratio is high, the absolute latency (6.56 µs) is likely acceptable within the 10ms search budget.

## 2. Results

### 2.1 Latency vs Candidate Count

Dataset: Random vectors, `M=16`.

| Candidates | Simple (ns) | Heuristic (ns) | Heuristic+Extend (ns) | Overhead (H/S) |
|:---|:---|:---|:---|:---|
| 32 | 183 | 2,298 | 2,389 | 12.6x |
| 64 | 471 | 6,557 | 6,681 | 13.9x |
| 128 | 777 | 15,571 | 15,263 | 20.0x |

**Key Findings:**
1.  **Simple is extremely fast:** It only involves sorting the candidates (heap pop) and copying.
2.  **Heuristic pays for diversity:** The cost is dominated by `L2Squared::distance` calls in the diversity loop.
3.  **Extension is cheap:** The difference between "Heuristic" and "Heuristic+Extend" is minimal (<5%) or within noise, confirming the efficiency of the bitset and the sparse connectivity of the test graph.

## 3. Detailed Metrics (64 Candidates)

| Metric | Simple | Heuristic | Delta |
|:---|:---|:---|:---|
| **Mean Latency** | 471 ns | 6,557 ns | +1,292% |
| **Throughput** | 2.1 M/s | 152 K/s | -92% |
| **Est. P99** | ~480 ns | ~6,700 ns | |

## 4. Constraint Analysis

| Constraint | Target | Actual | Verdict |
|:---|:---|:---|:---|
| **Overhead** | < 5x Simple | ~14x | ❌ FAIL |
| **Absolute Latency** | N/A | 6.6 µs | ✅ FAST |
| **Memory Alloc** | 0 in hot loop | 0 | ✅ PASS |

**Recommendation:**
The "Overhead < 5x" constraint is physically impossible if "Simple" does 0 distance calculations and "Heuristic" does meaningful geometric checks. The constraint should be revised to apply to **End-to-End Search Latency**, where the overhead of the heuristic will be amortized by the cost of graph traversal.

## 5. Conclusion

The implementation is correct and optimized (zero allocations), but the algorithmic cost of diversity checks is significantly higher than simple sorting. The component is ready for integration, but the strict component-level performance constraint needs relaxation.

**Status:** ⚠️ BASELINE ESTABLISHED (Constraint Violation Noted)

## 6. Reproduction

```bash
cargo bench --bench heuristic_bench
```
