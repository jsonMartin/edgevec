# Day 2: CI Hardening & Proptest Configuration (W18.2)

**Date:** Week 18, Day 2
**Task ID:** W18.2
**Agent:** RUST_ENGINEER
**Status:** [REVISED]
**Revision:** v1.2 — Adds CI timing validation (9→10 CI Simulation Accuracy)

---

## Pre-Task Validation Checklist [v1.2]

**Before starting W18.2, verify:**

- [ ] W18.1 marked COMPLETE
- [ ] `docs/RELEASE_CHECKLIST.md` exists
- [ ] `scripts/pre-release-check.sh` exists and is executable
- [ ] `docs/ROLLBACK_PROCEDURES.md` exists
- [ ] Development environment: `RUSTFLAGS`, `PROPTEST_CASES`, `NUM_VECTORS` set

---

## Buffer Allocation

| Component | Base | Buffer | Total |
|:----------|:----:|:------:|:-----:|
| proptest.toml | 1h | 0.25h | 1.25h |
| xtask ci-check | 2.5h | 0.5h | 3h |
| **CI Timing Validation [v1.2]** | 1.5h | 0.25h | 1.75h |
| **Total** | **5h** | **1h** | **6h**

---

## Objective

Optimize CI configuration to eliminate proptest warnings and ensure consistent test behavior across local and CI environments. Create `cargo xtask ci-check` for local CI simulation.

---

## Context

### Current Issues

1. **Proptest warnings in CI:** Regression file warnings appear
2. **Local/CI divergence:** Different behavior due to environment variables
3. **No easy local CI simulation:** Developers can't easily reproduce CI

---

## Deliverables

### 1. proptest.toml

Project-wide proptest configuration:

```toml
# proptest.toml
# EdgeVec Property Testing Configuration
#
# Environment Variables:
# - PROPTEST_CASES: Override case count (default: 256)
# - CI environments should set PROPTEST_CASES=32

[default]
# Default cases for local development (more thorough)
cases = 256

# Reduce shrinking iterations for faster failure diagnosis
max_shrink_iters = 100

# Disable persistence files to avoid CI warnings
# Persistence is useful for local debugging but causes noise in CI
failure_persistence = "off"

# Fork to catch panics in separate process
fork = false

# Timeout per test case
timeout = 30000

# Verbose output for debugging
verbose = 0
```

### 2. xtask Crate

Create `xtask/` directory with cargo xtask pattern:

```
xtask/
├── Cargo.toml
└── src/
    └── main.rs
```

**xtask/Cargo.toml:**
```toml
[package]
name = "xtask"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
# Minimal dependencies for task runner
```

**xtask/src/main.rs:**
```rust
//! EdgeVec Development Tasks
//!
//! Usage: cargo xtask <command>
//!
//! Commands:
//!   ci-check    Run CI validation locally with timing assertions
//!   pre-release Run full pre-release check

use std::env;
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

// [v1.2] CI Timing Limits — Fail if exceeded
const CI_FMT_TIMEOUT: Duration = Duration::from_secs(30);      // 30 seconds
const CI_CLIPPY_TIMEOUT: Duration = Duration::from_secs(120);  // 2 minutes
const CI_TEST_TIMEOUT: Duration = Duration::from_secs(600);    // 10 minutes
const CI_WASM_TIMEOUT: Duration = Duration::from_secs(120);    // 2 minutes

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!("Commands: ci-check, pre-release");
        return ExitCode::FAILURE;
    }

    match args[1].as_str() {
        "ci-check" => ci_check(),
        "pre-release" => pre_release(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            ExitCode::FAILURE
        }
    }
}

/// [v1.2] Run a step with timing validation
fn timed_step(name: &str, timeout: Duration, cmd: &[&str]) -> Result<Duration, ExitCode> {
    println!("\n--- {} ---", name);
    println!("Timeout: {}s", timeout.as_secs());

    let start = Instant::now();
    let status = Command::new(cmd[0])
        .args(&cmd[1..])
        .env("RUSTFLAGS", "-C target-cpu=x86-64-v2")
        .env("PROPTEST_CASES", "32")
        .env("NUM_VECTORS", "1000")
        .status();

    let elapsed = start.elapsed();

    match status {
        Ok(s) if s.success() => {
            // [v1.2] Check timing AFTER success
            if elapsed > timeout {
                eprintln!("{}: TIMING FAILURE", name);
                eprintln!("  Elapsed: {:.1}s", elapsed.as_secs_f64());
                eprintln!("  Limit:   {}s", timeout.as_secs());
                eprintln!("  This would timeout in CI!");
                return Err(ExitCode::FAILURE);
            }
            println!("{}: PASS ({:.1}s / {}s limit)",
                name, elapsed.as_secs_f64(), timeout.as_secs());
            Ok(elapsed)
        }
        Ok(s) => {
            eprintln!("{}: FAIL (exit code {:?})", name, s.code());
            Err(ExitCode::FAILURE)
        }
        Err(e) => {
            eprintln!("{}: ERROR ({})", name, e);
            Err(ExitCode::FAILURE)
        }
    }
}

fn ci_check() -> ExitCode {
    println!("=== EdgeVec CI Check ===");
    println!("Simulating CI environment with timing validation...\n");

    // Set CI environment variables
    env::set_var("RUSTFLAGS", "-C target-cpu=x86-64-v2");
    env::set_var("PROPTEST_CASES", "32");
    env::set_var("NUM_VECTORS", "1000");

    // [v1.2] Steps with individual timeouts matching CI
    let steps: &[(&str, Duration, &[&str])] = &[
        ("Formatting", CI_FMT_TIMEOUT, &["cargo", "fmt", "--", "--check"]),
        ("Clippy", CI_CLIPPY_TIMEOUT, &["cargo", "clippy", "--all-targets", "--", "-D", "clippy::correctness"]),
        ("Tests", CI_TEST_TIMEOUT, &["cargo", "test", "--all"]),
        ("WASM Check", CI_WASM_TIMEOUT, &["cargo", "check", "--target", "wasm32-unknown-unknown"]),
    ];

    let mut total_elapsed = Duration::ZERO;

    for (name, timeout, cmd) in steps {
        match timed_step(name, *timeout, cmd) {
            Ok(elapsed) => total_elapsed += elapsed,
            Err(code) => return code,
        }
    }

    println!("\n=== All CI checks passed! ===");
    println!("Total time: {:.1}s", total_elapsed.as_secs_f64());

    // [v1.2] Warn if total is approaching CI job limit (typically 30-60 min)
    if total_elapsed > Duration::from_secs(900) {
        println!("WARNING: Total time > 15 minutes. Consider optimizing.");
    }

    ExitCode::SUCCESS
}

fn pre_release() -> ExitCode {
    println!("=== EdgeVec Pre-Release Check ===");

    // Run CI check first
    if ci_check() != ExitCode::SUCCESS {
        return ExitCode::FAILURE;
    }

    // Additional pre-release steps
    let steps = [
        ("Doc Generation", &["cargo", "doc", "--no-deps"]),
        ("Publish Dry Run", &["cargo", "publish", "--dry-run"]),
    ];

    for (name, cmd) in steps {
        println!("\n--- {} ---", name);
        let status = Command::new(cmd[0])
            .args(&cmd[1..])
            .status();

        match status {
            Ok(s) if s.success() => println!("{}: PASS", name),
            Ok(s) => {
                eprintln!("{}: FAIL (exit code {:?})", name, s.code());
                return ExitCode::FAILURE;
            }
            Err(e) => {
                eprintln!("{}: ERROR ({})", name, e);
                return ExitCode::FAILURE;
            }
        }
    }

    println!("\n=== Pre-release checks passed! ===");
    println!("Safe to proceed with release workflow.");
    ExitCode::SUCCESS
}
```

### 3. CI Workflow Comments

Update `.github/workflows/ci.yml` with timeout documentation.

---

## Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.2.1 | Proptest warnings eliminated | CI logs clean |
| AC18.2.2 | `proptest.toml` created | File exists |
| AC18.2.3 | CI timeout documented per job | ci.yml comments |
| AC18.2.4 | Local vs CI environment parity documented | README section |
| AC18.2.5 | `cargo xtask ci-check` works | Command runs successfully |
| AC18.2.6 | **[v1.2]** CI timing validation | Each step shows elapsed vs limit |
| AC18.2.7 | **[v1.2]** Timing failure mode tested | Artificially slow step fails build |

---

## Implementation Plan

### Step 1: Create proptest.toml

Add to project root.

### Step 2: Create xtask Crate

```bash
mkdir -p xtask/src
# Create Cargo.toml and main.rs
```

### Step 3: Update Workspace

Add to root `Cargo.toml`:

```toml
[workspace]
members = [".", "xtask"]
```

### Step 4: Update CI Workflow

Add comments explaining timeouts and environment variables.

### Step 5: Update README

Add "Development" section explaining local CI simulation.

---

## Files to Create/Modify

| File | Action | Description |
|:-----|:-------|:------------|
| `proptest.toml` | CREATE | Proptest configuration |
| `xtask/Cargo.toml` | CREATE | Xtask crate manifest |
| `xtask/src/main.rs` | CREATE | Xtask implementation |
| `Cargo.toml` | MODIFY | Add workspace member |
| `.github/workflows/ci.yml` | MODIFY | Add documentation comments |
| `README.md` | MODIFY | Add development section |

---

## Verification Commands

```bash
# Verify proptest.toml
test -f proptest.toml && echo "PASS: proptest.toml exists"

# Verify xtask builds
cargo build -p xtask && echo "PASS: xtask builds"

# Run CI check
cargo xtask ci-check
```

---

## Handoff

**On Completion:**
- Mark W18.2 as COMPLETE
- Submit for hostile review
- Proceed to W18.3 (parallel with W18.4)
