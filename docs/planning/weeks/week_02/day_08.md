# Week 2 - Day 8: Crash Recovery Logic

**Objective:** Implement the logic to read back the WAL and detect corruption.

---

## Morning: RUST_ENGINEER (Reader Implementation)

- [ ] Define `WalIterator` struct in `src/persistence/wal.rs`.
    - [ ] Holds a `BufReader<File>`.
- [ ] Implement `Iterator` for `WalIterator`.
    - [ ] Output: `Result<(WalEntry, Vec<u8>), WalError>`.
    - [ ] Logic:
        1.  Read 16 bytes (Header). If EOF, stop.
        2.  Read `payload_len` bytes.
        3.  Read 4 bytes (CRC).
        4.  Compute local CRC of (Header + Payload).
        5.  Compare local CRC vs read CRC.
        6.  If mismatch -> Return `WalError::ChecksumMismatch`.
        7.  If success -> Return `Ok((entry, payload))`.

## Afternoon: TESTER (Corruption Tests)

- [ ] Create `tests/unit_wal_recovery.rs`.
- [ ] Implement **UNIT-REC-001**: Clean Recovery.
    - [ ] Write 3 entries. Close. Re-open. Assert 3 entries read correctly.
- [ ] Implement **UNIT-REC-002**: Truncated File.
    - [ ] Write entry, cut file in half.
    - [ ] Assert iterator returns `WalError::Truncated` or handles EOF gracefully (architecture decision: discard incomplete tail?).
- [ ] Implement **UNIT-REC-003**: Bit Rot.
    - [ ] Write entry. Flip 1 bit in payload.
    - [ ] Assert iterator returns `WalError::ChecksumMismatch`.

## Deliverables

- [ ] `WalIterator` correctly validates CRCs.
- [ ] Corruption is detected, NOT ignored or panicking.

