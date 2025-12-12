# Week 1 - Day 2: FileHeader Serialization

**Objective:** Implement the binary file header and verify it with Property-Based Testing.

---

## Morning: RUST_ENGINEER (Implementation)

- [ ] Define `FileHeader` struct in `src/persistence/header.rs`.
    - [ ] Reference: `DATA_LAYOUT.md` Section 4.1.
    - [ ] Fields: `magic`, `version_major`, `version_minor`, `flags`, `vector_count`, `index_offset`, `metadata_offset`, `rng_seed`, `dimensions`, `header_crc`, `hnsw_m`, `hnsw_m0`, `reserved`.
    - [ ] Derive: `Debug`, `Clone`, `Copy`, `Pod`, `Zeroable` (bytemuck).
- [ ] Implement `FileHeader::new(dimensions: u32) -> Self`.
- [ ] Implement `FileHeader::as_bytes(&self) -> &[u8; 64]`.
- [ ] Implement `FileHeader::from_bytes(bytes: &[u8]) -> Result<Self, Error>`.
    - [ ] Validate Magic ("EVEC").
    - [ ] Validate Version.

## Afternoon: TESTER (Verification)

- [ ] Implement `PROP-PERSIST-001` in `tests/proptest_header.rs`.
    - [ ] Reference: `TEST_STRATEGY.md` Section 2.5.
    - [ ] Logic: Generate random headers → Serialize → Deserialize → Assert Equal.
- [ ] Create Fuzz Target `fuzz/fuzz_targets/header_parse.rs`.
    - [ ] Reference: `TEST_STRATEGY.md` Section 3.2 (FUZZ-004).
    - [ ] Feed random bytes to `FileHeader::from_bytes`.
    - [ ] Assert: Must return `Ok` or `Err`, NEVER panic.

## Deliverables

- [ ] `src/persistence/header.rs`
- [ ] `tests/proptest_header.rs` passing.
- [ ] `fuzz/fuzz_targets/header_parse.rs` running without crashes.

