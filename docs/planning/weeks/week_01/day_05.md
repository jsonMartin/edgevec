# Week 1 - Day 5: Integration - "The Empty File"

**Objective:** Prove the persistence chain by generating a valid, empty `.evec` file that can be read back.

---

## Morning: RUST_ENGINEER (Integration)

- [ ] Create `src/persistence/writer.rs`.
    - [ ] Function: `write_empty_index(config: HnswConfig) -> Vec<u8>`.
    - [ ] Logic:
        1. Create `FileHeader`.
        2. Calculate CRC32.
        3. Write Magic + Header + Footer.
- [ ] Create `src/persistence/reader.rs`.
    - [ ] Function: `read_index_header(data: &[u8]) -> Result<FileHeader, Error>`.
    - [ ] Logic: Check CRC32, Check Magic.

## Afternoon: TESTER (E2E Verification)

- [ ] Implement Integration Test `tests/e2e_empty_file.rs`.
    - [ ] Step 1: Call `write_empty_index`.
    - [ ] Step 2: Save to disk (temp file).
    - [ ] Step 3: Read back using `read_index_header`.
    - [ ] Step 4: Verify Config matches.
- [ ] Manual Hex Inspection.
    - [ ] Write a script to dump the generated file.
    - [ ] Verify visually: `45 56 45 43` (EVEC) at byte 0.

## Deliverables

- [ ] A generated binary file that follows `DATA_LAYOUT.md`.
- [ ] Code capable of reading that file.
- [ ] End of Week 1 Hostile Review.

