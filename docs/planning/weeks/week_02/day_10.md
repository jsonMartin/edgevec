# Week 2 - Day 10: Vector Storage Integration

**Objective:** Connect the WAL to the in-memory `VectorStorage`.

---

## Morning: RUST_ENGINEER (Integration)

- [ ] Update `VectorStorage` struct.
    - [ ] Add optional `wal: Option<WalAppender>`.
- [ ] Update `VectorStorage::insert`.
    - [ ] Step 1: Create `InsertPayload`.
    - [ ] Step 2: If WAL enabled, append payload.
    - [ ] Step 3: If WAL write fails, return Error (do NOT modify memory).
    - [ ] Step 4: Update in-memory `data` and `tombstones`.
- [ ] Implement `VectorStorage::recover(path: Path) -> Result<Self, Error>`.
    - [ ] Open WAL.
    - [ ] Replay all entries into memory.
    - [ ] Return restored `VectorStorage`.

## Afternoon: TESTER (Integration Test)

- [ ] Create `tests/integration_storage_durability.rs`.
- [ ] Implement **INT-DUR-001**: Persistence Cycle.
    - [ ] Create Storage. Insert 100 vectors. Drop Storage.
    - [ ] `VectorStorage::recover(path)`.
    - [ ] Assert: Storage has 100 vectors.
    - [ ] Assert: Data matches exactly.

## Deliverables

- [ ] `VectorStorage` is now durable.
- [ ] Integration test confirms data survives "restart".

