# Week 2 Overview: The Durable Foundation

**Dates:** Dec 15 - Dec 19, 2025
**Objective:** Implement the Write-Ahead Log (WAL) and basic Vector Storage to ensure data survives crashes.
**Status:** [APPROVED]

---

## Goal: "If it's not on disk, it doesn't exist."

By the end of this week, we will have a crash-safe storage engine. We will **NOT** have a graph (HNSW) yet. We focus purely on the durability guarantee:
1.  **Atomicity:** Writes either fully happen or don't.
2.  **Integrity:** Corrupted data is detected (CRC32).
3.  **Recovery:** We can rebuild memory state from disk logs.

## Daily Schedule

| Day | Theme | Owner | Key Deliverable |
|:----|:------|:------|:----------------|
| **Day 6** | WAL Data Structures | RUST_ENGINEER | `WalEntry` serialization tests passing |
| **Day 7** | The Appender | RUST_ENGINEER | Append-only file writing working |
| **Day 8** | Crash Recovery | RUST_ENGINEER | Read-back logic with checksum validation |
| **Day 9** | **The Boss Fight: Properties** | TESTER | `PROP-WAL-001` (Write -> Crash -> Recover) |
| **Day 10**| Storage Integration | RUST_ENGINEER | `VectorStorage` backed by WAL |

## Critical Constraints

1.  **Byte-Perfect Layouts:** All structs must match `DATA_LAYOUT.md` exactly.
2.  **No Panics:** Recovery code must never panic on corrupted data; it must return `Result::Err`.
3.  **Checksums Mandatory:** Every WAL entry has a CRC32. No unchecked reads.
4.  **No HNSW:** Do not touch graph code. Focus on storage.

---

## Output Artifacts

- `src/persistence/wal.rs` (Core logic)
- `src/persistence/entry.rs` (Structs)
- `tests/proptest_wal.rs` (The Boss Fight)

