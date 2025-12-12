# HOSTILE_REVIEWER: Rejection — Week 6 Plan Final

**Date:** 2025-12-09
**Artifact:** Week 6 Plan Final (W6D26, W6D27, WEEKLY_TASK_PLAN)
**Author:** PLANNER
**Status:** ❌ REJECTED

---

## Summary

The plan defines a robust "Emergency Sprint" structure with clear parallelism and fallback strategies. The architecture correctly mandates "Zero-Copy" and "Flat Layout" to handle the memory constraints. However, a critical correctness error in the data loader specification regarding the SIFT1M file formats renders the "Recall" objective impossible to verify as specified.

---

## Findings

### Critical Issues: 1

- [C1] **Ground Truth Format Mismatch**
  - **Description:** `W6D26.md` Task 1 explicitly lists `.fvecs` and `.bvecs` as the required loader formats.
  - **Evidence:** `W6D26.md` Line 21: "Deliverable: A clearly defined data loader interface for `.fvecs` / `.bvecs` formats."
  - **Constraint Violated:** "Correctness Audit: Does the plan specify *how* Ground Truth is loaded?"
  - **Context:** SIFT1M and `siftsmall` use `.ivecs` (4-byte signed integers) for ground truth, not `.bvecs` (1-byte unsigned integers). `.bvecs` is used for the Billion-scale dataset (SIFT1B), which is not the target.
  - **Impact:** Implementing a `.bvecs` loader will fail to parse `sift_groundtruth.ivecs`. The `BenchmarkRunner` will be unable to load ground truth, making the "Recall >= 95%" acceptance criterion impossible to verify on Day 27 without emergency hotfixes.
  - **Required Action:** Update `W6D26.md` to specify `.ivecs` loader support instead of (or in addition to) `.bvecs`.

### Major Issues: 0

### Minor Issues: 1

- [m1] **Peak Memory Usage in `from_fvecs`**
  - **Description:** `FlatDataset::from_fvecs` implies loading raw bytes and then copying to `Vec<f32>`. For 512MB SIFT1M, this requires >1GB peak memory (Raw + Parsed) during the load phase.
  - **Impact:** Risk of OOM in constrained WASM environments (though likely fine with 2GB-4GB limits).
  - **Recommendation:** Ensure the implementation discards the raw buffer immediately after parsing, or streams the parsing if possible (though streaming is harder with `FileProvider` logic in WASM).

---

## Verdict

**REJECTED**

This artifact fails **1** critical quality gate (Correctness of Specification). The engineering team cannot succeed if they build the wrong file loader.

---

## Required Actions Before Resubmission

1. [ ] Update `W6D26.md` Task 1 to replace/add `.ivecs` format support for Ground Truth loading.
2. [ ] Verify `W6D27.md` references the correct format if applicable (it currently assumes "Loader" works).

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-09*
*Verdict: REJECTED*

