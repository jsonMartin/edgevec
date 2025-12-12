# Week 2 - Day 9: The Boss Fight (Property Tests)

**Objective:** Verify the durability guarantees using Property-Based Testing.

---

## Morning: TESTER (PropTest Implementation)

- [ ] Create `tests/proptest_wal.rs`.
- [ ] Implement **PROP-WAL-001**: Write-Crash-Recover.
    - [ ] Reference: `TEST_STRATEGY.md` Section 2.5.
    - [ ] **Scenario A: Clean Shutdown**
        1. Generate a random list of payloads (Vec<u8>).
        2. Write them all to a `Vec<u8>` (simulating file) using `WalAppender`.
        3. Create a `WalIterator` from the buffer.
        4. Assert: Recovered items == Original items.
    - [ ] **Scenario B: The Torn Write**
        1. Generate a valid WAL buffer with N entries.
        2. **Simulate Crash:** Randomly truncate the buffer at byte offset `len - K` (where K < size of last entry).
        3. Run `WalIterator` on the truncated buffer.
        4. Assert: Recovered items == N-1 entries (perfect prefix).
        5. Assert: **NO PANIC** on the partial tail.
        6. Assert: The partial tail is handled safely (either silently ignored or returns specific `IncompleteEntry` error, depending on design).

## Afternoon: TESTER (Fuzzing Integration)

- [ ] Update `fuzz/fuzz_targets/wal_replay.rs` (or create if new).
- [ ] Goal: Feed random garbage to `WalIterator`.
- [ ] Assertion: Iterator must return `Ok` or `Err`, but NEVER Panic.
- [ ] Run fuzzer for 15 minutes to verify stability.

## Deliverables

- [ ] `PROP-WAL-001` passing with 1000+ iterations (covering both Clean and Torn scenarios).
- [ ] Fuzzer confirms parser safety.
