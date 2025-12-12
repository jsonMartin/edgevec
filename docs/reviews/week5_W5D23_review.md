# HOSTILE_REVIEWER: Approval — W5D23 (Soft Delete Logic)

**Date:** 2025-12-08
**Artifact:** W5D23 (Delete Logic & Tests)
**Author:** RUST_ENGINEER
**Status:** ✅ APPROVED

---

## Summary

Reviewed the implementation of "Soft Delete" logic in HNSW. The critical requirement was ensuring that "deleted" nodes (ghosts) continue to participate in graph traversal (routing) while being excluded from final search results.

**Scope:**
- `src/hnsw/search.rs`: Search logic modification.
- `tests/proptest_hnsw_delete.rs`: Connectivity verification.
- `docs/benchmarks/week5_delete_report.md`: Performance impact analysis.

---

## Findings

### Critical Issues: 0

### Major Issues: 0

### Minor Issues: 1
- [m1] **Unbounded Traversal Risk:** The current logic excludes ghosts from `ctx.results`, which means they do not count towards the `ef` budget. In a pathological case (99% deleted graph), a search could traverse the entire graph to find `k` neighbors, potentially blowing past latency budgets.
  - **Status:** Accepted for now.
  - **Mitigation:** W5.5/Week 6 Vacuum/Compaction tasks will address this by physically removing ghosts.

---

## Logic Audit (The "Ghost" Check)

- **Requirement:** `search_layer` must traverse ghosts but not return them.
- **Verification:**
  - `ctx.candidates.push(...)` is unconditional (Lines 143, 205). ✅ **Ghosts are traversed.**
  - `ctx.results.push(...)` is guarded by `!self.provider.is_deleted(...)` (Lines 144, 207). ✅ **Ghosts are filtered.**
  - **Pruning Logic:** `ctx.results` (the pruning bound) only contains *living* nodes. This correctly forces the search to expand until `ef` *living* candidates are found, preserving recall.

## Correctness Audit

- **Requirement:** Routing via Ghost test must pass.
- **Verification:**
  - `test_ghost_routing_manual_construction` implements the `A -> B(deleted) -> C` scenario.
  - Test Execution: **PASSED**.
  - Logic confirms `C` is found via `B`, but `B` is not returned.

## Performance Audit

- **Requirement:** Acceptable slowdown.
- **Observation:** 50% delete ratio leads to ~1.95x latency increase (69µs -> 135µs).
- **Assessment:** This is theoretically consistent (traversing 2x nodes to find k valid ones). Given the absolute latency (135µs) is well within the 10ms budget, this is **ACCEPTABLE**.

---

## Verdict

**APPROVED**

The implementation correctly handles soft deletions by preserving graph connectivity ("ghost routing"). The performance penalty is linear and understood. The code is safe and tested.

---

## Next Steps

- **Proceed to W5D24 (Advanced Fuzzing).**
- Ensure `Vacuum` logic is scheduled (Week 6) to prevent unbounded ghost accumulation.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-08*

