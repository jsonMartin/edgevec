# Week 1 - Day 1: Project Init & Fuzz Harness

**Objective:** Initialize the repository with strict quality gates and the fuzzing infrastructure.

---

## Morning: TESTER (Repo Setup)

- [ ] Initialize Cargo project `edgevec`.
    - [ ] `cargo init --lib`
    - [ ] Add dependencies: `thiserror`, `serde`, `bytemuck`.
    - [ ] Add dev-dependencies: `proptest`, `criterion`.
- [ ] Configure `Cargo.toml` for WASM optimization.
    - [ ] `[profile.release] lto = true`, `opt-level = "z"`.
- [ ] Create `CONTRIBUTING.md` with "Nvidia Grade" rules.
    - [ ] Explicitly ban `unwrap()` in library code.

## Afternoon: TESTER (Fuzzing)

- [ ] Install `cargo-fuzz`.
- [ ] Initialize fuzz directory: `cargo fuzz init`.
- [ ] Create dummy fuzz target `fuzz/fuzz_targets/dummy_harness.rs`.
    - [ ] Reference: `TEST_STRATEGY.md` Section 3.1.
    - [ ] Goal: Ensure the fuzzer compiles and runs.
- [ ] Set up CI workflow (GitHub Actions).
    - [ ] Job 1: `cargo test`
    - [ ] Job 2: `cargo clippy -- -D warnings`
    - [ ] Job 3: `cargo check --target wasm32-unknown-unknown`

## Deliverables

- [ ] `Cargo.toml` with dependencies.
- [ ] Working Fuzz Harness (`cargo fuzz run dummy_harness` passes).
- [ ] CI Pipeline passing green.

