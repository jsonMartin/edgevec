# Week 1 Overview: The Skeleton & The Verify Loop

**Dates:** Dec 8 - Dec 12, 2025
**Objective:** Establish the project foundation where verification precedes implementation.
**Status:** ✅ APPROVED

---

## Goal: "We are NOT building the engine. We are building the *verifier*."

By the end of this week, we will not have a working vector database. We **WILL** have:
1.  A CI pipeline that rejects unsafe code.
2.  A fuzzing harness waiting for inputs.
3.  The ability to generate and verify valid `.evec` file headers.
4.  The ability to verify `VectorStorage` memory layouts.

## Daily Schedule

| Day | Theme | Owner | Key Deliverable |
|:----|:------|:------|:----------------|
| **Day 1** | Project Init & Fuzz Harness | TESTER | `cargo fuzz run` works |
| **Day 2** | `FileHeader` & Props | ENGINEER | `PROP-PERSIST-001` pass |
| **Day 3** | `SectionHeader` & Alignment | ENGINEER | `static_assert` checks |
| **Day 4** | HNSW Stub & Graph Invariants | TESTER | Graph validation logic |
| **Day 5** | Integration: "The Empty File" | ENGINEER | Valid 64-byte file on disk |

## Critical Constraints

1.  **No `unsafe` without Miri:** Every block of `unsafe` code must be accompanied by a Miri test case.
2.  **No Features without Tests:** We do not write the implementation until the test exists (TDD).
3.  **WASM Compatibility:** All code must compile for `wasm32-unknown-unknown` (checked via CI).

---

## Output Artifacts

- `src/lib.rs` (Skeleton)
- `src/persistence/header.rs` (Implemented)
- `tests/proptest_header.rs` (Implemented)
- `fuzz/fuzz_targets/header_parse.rs` (Implemented)

