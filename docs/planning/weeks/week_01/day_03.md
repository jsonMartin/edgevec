# Week 1 - Day 3: SectionHeader & Alignment

**Objective:** Implement section headers and verify memory layout assumptions using static assertions.

---

## Morning: RUST_ENGINEER (Implementation)

- [ ] Define `SectionHeader` struct in `src/persistence/section.rs`.
    - [ ] Reference: `DATA_LAYOUT.md` Section 4.4.
    - [ ] Fields: `section_type`, `section_len`, `reserved`.
- [ ] Implement `SectionHeader` serialization (unsafe cast / bytemuck).
- [ ] Add `static_assertions` crate.
- [ ] Implement Alignment Checks in `src/lib.rs`.
    - [ ] Reference: `DATA_LAYOUT.md` Section 7.
    - [ ] `const_assert_eq!(size_of::<FileHeader>(), 64);`
    - [ ] `const_assert_eq!(align_of::<VectorId>(), 8);`

## Afternoon: TESTER (Layout Verification)

- [ ] Implement `PROP-ALIGN-001` (New).
    - [ ] Verify that `SectionHeader` serialization matches manual byte packing.
    - [ ] Ensure no padding bytes are uninitialized (security risk).
- [ ] Verify `wasm32-unknown-unknown` layout consistency.
    - [ ] Run `cargo test --target wasm32-unknown-unknown` (requires `wasm-pack test` setup).
    - [ ] Confirm `usize` is 32-bit but `u64` fields remain 64-bit aligned.

## Deliverables

- [ ] `src/persistence/section.rs`.
- [ ] Compilation failing if `FileHeader` size != 64 (Static Assert verification).
- [ ] WASM layout verified.

