//! EdgeVec Development Tasks
//!
//! Usage: cargo xtask <command>
//!
//! Commands:
//!   ci-check    Run CI validation locally with timing assertions
//!   pre-release Run full pre-release check
//!
//! W18.2: CI Hardening — Local CI simulation with timing validation

use std::env;
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

// CI Timing Limits — Match GitHub Actions job timeouts
// These values are calibrated to catch issues that would timeout in CI
const CI_FMT_TIMEOUT: Duration = Duration::from_secs(30); // 30 seconds
const CI_CLIPPY_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes
const CI_TEST_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes
const CI_WASM_TIMEOUT: Duration = Duration::from_secs(120); // 2 minutes

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return ExitCode::FAILURE;
    }

    match args[1].as_str() {
        "ci-check" => ci_check(),
        "pre-release" => pre_release(),
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("EdgeVec Development Tasks");
    eprintln!();
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  ci-check    Run CI validation locally with timing assertions");
    eprintln!("  pre-release Run full pre-release check (ci-check + docs + publish dry-run)");
    eprintln!("  help        Show this help message");
    eprintln!();
    eprintln!("Environment Variables:");
    eprintln!("  RUSTFLAGS       Compiler flags (default: -C target-cpu=x86-64-v2)");
    eprintln!("  PROPTEST_CASES  Number of proptest cases (default: 32 for CI)");
    eprintln!("  NUM_VECTORS     Number of vectors for integration tests (default: 1000)");
}

/// Run a step with timing validation
///
/// Returns the elapsed duration if successful, or an error code if the step fails
/// or exceeds the timeout.
fn timed_step(name: &str, timeout: Duration, cmd: &[&str]) -> Result<Duration, ExitCode> {
    println!();
    println!("--- {} ---", name);
    println!("Command: {}", cmd.join(" "));
    println!("Timeout: {}s", timeout.as_secs());
    println!();

    let start = Instant::now();

    let mut command = Command::new(cmd[0]);
    command.args(&cmd[1..]);

    // Set CI environment variables
    command.env(
        "RUSTFLAGS",
        env::var("RUSTFLAGS").unwrap_or_else(|_| "-C target-cpu=x86-64-v2".to_string()),
    );
    command.env(
        "PROPTEST_CASES",
        env::var("PROPTEST_CASES").unwrap_or_else(|_| "32".to_string()),
    );
    command.env(
        "NUM_VECTORS",
        env::var("NUM_VECTORS").unwrap_or_else(|_| "1000".to_string()),
    );

    let status = command.status();
    let elapsed = start.elapsed();

    match status {
        Ok(s) if s.success() => {
            // Check timing AFTER success
            if elapsed > timeout {
                eprintln!();
                eprintln!("TIMING FAILURE: {}", name);
                eprintln!("  Elapsed: {:.1}s", elapsed.as_secs_f64());
                eprintln!("  Limit:   {}s", timeout.as_secs());
                eprintln!("  This would timeout in CI!");
                eprintln!();
                return Err(ExitCode::FAILURE);
            }
            println!();
            println!(
                "{}: PASS ({:.1}s / {}s limit)",
                name,
                elapsed.as_secs_f64(),
                timeout.as_secs()
            );
            Ok(elapsed)
        }
        Ok(s) => {
            eprintln!();
            eprintln!("{}: FAIL (exit code {:?})", name, s.code());
            Err(ExitCode::FAILURE)
        }
        Err(e) => {
            eprintln!();
            eprintln!("{}: ERROR ({})", name, e);
            Err(ExitCode::FAILURE)
        }
    }
}

/// Run CI validation locally
///
/// This simulates the GitHub Actions CI environment:
/// - Sets RUSTFLAGS for x86-64-v2 compatibility
/// - Sets PROPTEST_CASES=32 for faster property tests
/// - Sets NUM_VECTORS=1000 for faster integration tests
/// - Validates each step completes within CI timeout limits
fn ci_check() -> ExitCode {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           EdgeVec CI Check — Local Simulation                 ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Environment:");
    println!(
        "  RUSTFLAGS:      {}",
        env::var("RUSTFLAGS").unwrap_or_else(|_| "-C target-cpu=x86-64-v2 (default)".to_string())
    );
    println!(
        "  PROPTEST_CASES: {}",
        env::var("PROPTEST_CASES").unwrap_or_else(|_| "32 (default)".to_string())
    );
    println!(
        "  NUM_VECTORS:    {}",
        env::var("NUM_VECTORS").unwrap_or_else(|_| "1000 (default)".to_string())
    );

    // CI steps with individual timeouts matching GitHub Actions
    let steps: &[(&str, Duration, &[&str])] = &[
        (
            "Formatting",
            CI_FMT_TIMEOUT,
            &["cargo", "fmt", "--", "--check"],
        ),
        (
            "Clippy",
            CI_CLIPPY_TIMEOUT,
            &[
                "cargo",
                "clippy",
                "--all-targets",
                "--",
                "-D",
                "clippy::correctness",
                "-W",
                "clippy::suspicious",
            ],
        ),
        ("Tests", CI_TEST_TIMEOUT, &["cargo", "test", "--all"]),
        (
            "WASM Check",
            CI_WASM_TIMEOUT,
            &["cargo", "check", "--target", "wasm32-unknown-unknown"],
        ),
    ];

    let mut total_elapsed = Duration::ZERO;

    for (name, timeout, cmd) in steps {
        match timed_step(name, *timeout, cmd) {
            Ok(elapsed) => total_elapsed += elapsed,
            Err(code) => {
                println!();
                println!("╔═══════════════════════════════════════════════════════════════╗");
                println!("║                    CI CHECK FAILED                            ║");
                println!("╚═══════════════════════════════════════════════════════════════╝");
                return code;
            }
        }
    }

    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                  All CI checks passed!                        ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total time: {:.1}s", total_elapsed.as_secs_f64());

    // Warn if total is approaching CI job limit (typically 30-60 min)
    if total_elapsed > Duration::from_secs(900) {
        println!();
        println!("WARNING: Total time > 15 minutes. Consider optimizing.");
    }

    ExitCode::SUCCESS
}

/// Run full pre-release validation
///
/// This runs ci_check plus additional pre-release checks:
/// - Documentation generation
/// - Cargo publish dry run
fn pre_release() -> ExitCode {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║         EdgeVec Pre-Release Check                             ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    // Run CI check first
    if ci_check() != ExitCode::SUCCESS {
        return ExitCode::FAILURE;
    }

    println!();
    println!("--- Additional Pre-Release Checks ---");

    // Additional pre-release steps (no strict timing)
    let steps: &[(&str, &[&str])] = &[
        ("Documentation", &["cargo", "doc", "--no-deps"]),
        ("Publish Dry Run", &["cargo", "publish", "--dry-run"]),
    ];

    for (name, cmd) in steps {
        println!();
        println!("--- {} ---", name);
        let status = Command::new(cmd[0]).args(&cmd[1..]).status();

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

    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║              Pre-release checks passed!                       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Safe to proceed with release workflow.");
    println!("See docs/RELEASE_CHECKLIST.md for next steps.");

    ExitCode::SUCCESS
}
