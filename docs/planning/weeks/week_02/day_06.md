# Week 2 - Day 6: WAL Data Structures

**Objective:** Implement the binary format for the Write-Ahead Log.

---

## Morning: RUST_ENGINEER (Struct Implementation)

- [ ] Define `WalEntry` struct in `src/persistence/entry.rs`.
    - [ ] Reference: `DATA_LAYOUT.md` Section 4.2.
    - [ ] Fields: `sequence` (u64), `entry_type` (u8), `_pad` ([u8; 3]), `payload_len` (u32).
    - [ ] Note: The payload itself follows this header.
- [ ] Define `InsertPayload` struct.
    - [ ] Reference: `DATA_LAYOUT.md` Section 4.3.
    - [ ] Fields: `vector_id` (u64), `dimensions` (u32), `vector_data` (Vec<f32> implicitly).
- [ ] Implement `WalEntry::new(seq: u64, type: u8, len: u32) -> Self`.
- [ ] Implement `WalEntry::write<W: Write>(&self, writer: &mut W) -> Result<(), Error>`.
    - [ ] Must use Little Endian for all fields.

## Afternoon: TESTER (Serialization Verification)

- [ ] Create `tests/unit_wal_entry.rs`.
- [ ] Implement **UNIT-WAL-001**: Size verification.
    - [ ] Assert `size_of::<WalEntry>() == 16`.
- [ ] Implement **UNIT-WAL-002**: Serialization Roundtrip.
    - [ ] Create `WalEntry`.
    - [ ] Write to `Vec<u8>`.
    - [ ] Read back.
    - [ ] Assert equality.
- [ ] Implement **UNIT-WAL-003**: Payload Alignment.
    - [ ] Verify payload starts immediately after the header (offset 16).

## Deliverables

- [ ] `src/persistence/entry.rs` compiles.
- [ ] `cargo test unit_wal_entry` passes.

