# ARM64 Cross-Compilation Guide

**Version:** 1.0.0
**Created:** Week 20 (2025-12-23)
**Target:** `aarch64-unknown-linux-gnu`

---

## Overview

EdgeVec supports ARM64 cross-compilation for mobile and embedded platforms. This guide covers:
- Setting up the cross-compilation toolchain
- Building for ARM64
- Running tests under QEMU emulation
- CI configuration

---

## Prerequisites

### Required Tools

| Tool | Version | Purpose |
|:-----|:--------|:--------|
| Rust | 1.70+ (MSRV) | Compiler |
| Docker | 20.10+ | Container runtime for `cross` |
| cross | latest | Cross-compilation tool |

### Platform Support

| Platform | Status | Notes |
|:---------|:-------|:------|
| Ubuntu 22.04+ | Fully supported | CI platform |
| macOS (Intel/ARM) | Supported | Requires Docker Desktop |
| Windows (WSL2) | Supported | Requires Docker Desktop + WSL2 |
| Windows (native) | Limited | Use WSL2 for best results |

---

## Quick Start

### 1. Install Rust ARM64 Target

```bash
rustup target add aarch64-unknown-linux-gnu
```

### 2. Install Cross Tool

```bash
# Install from git (recommended for latest features)
cargo install cross --git https://github.com/cross-rs/cross

# Verify installation
cross --version
```

### 3. Build for ARM64

```bash
# Release build
cross build --target aarch64-unknown-linux-gnu --release

# Debug build
cross build --target aarch64-unknown-linux-gnu
```

### 4. Run Tests Under QEMU

```bash
# Run all tests (uses QEMU automatically)
cross test --target aarch64-unknown-linux-gnu --release

# Run specific test
cross test --target aarch64-unknown-linux-gnu --release simd

# Run with verbose output
cross test --target aarch64-unknown-linux-gnu --release -- --nocapture
```

---

## Detailed Setup

### Ubuntu/Debian

```bash
# 1. Install Docker
sudo apt update
sudo apt install -y docker.io
sudo usermod -aG docker $USER
# Log out and back in for group changes

# 2. Install Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. Add ARM64 target
rustup target add aarch64-unknown-linux-gnu

# 4. Install cross
cargo install cross --git https://github.com/cross-rs/cross

# 5. Verify
cross build --target aarch64-unknown-linux-gnu --release
```

### macOS

```bash
# 1. Install Docker Desktop
# Download from: https://www.docker.com/products/docker-desktop/

# 2. Start Docker Desktop and ensure it's running

# 3. Install Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 4. Add ARM64 target
rustup target add aarch64-unknown-linux-gnu

# 5. Install cross
cargo install cross --git https://github.com/cross-rs/cross

# 6. Verify
cross build --target aarch64-unknown-linux-gnu --release
```

### Windows (WSL2)

```powershell
# 1. Enable WSL2
wsl --install

# 2. Install Docker Desktop with WSL2 backend
# Download from: https://www.docker.com/products/docker-desktop/
# Enable "Use the WSL 2 based engine" in settings
```

```bash
# In WSL2 terminal:

# 3. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 4. Add ARM64 target
rustup target add aarch64-unknown-linux-gnu

# 5. Install cross
cargo install cross --git https://github.com/cross-rs/cross

# 6. Verify
cross build --target aarch64-unknown-linux-gnu --release
```

---

## CI Configuration

EdgeVec uses GitHub Actions for ARM64 CI. The workflow is defined in `.github/workflows/arm-ci.yml`.

### Jobs

| Job | Purpose | Timeout |
|:----|:--------|:--------|
| `arm64-build` | Cross-compile for ARM64 | 10 min |
| `arm64-test` | Run tests under QEMU | 15 min |
| `arm64-lint` | Clippy for ARM64 | 10 min |
| `x86-regression` | Verify x86 not broken | 10 min |

### Environment Variables

```yaml
env:
  PROPTEST_CASES: "16"    # Reduced for QEMU speed
  NUM_VECTORS: "500"      # Reduced for QEMU speed
```

### Triggering CI

ARM CI runs automatically on:
- Push to `main` branch
- Pull requests to `main` branch

---

## NEON SIMD Detection

> **Note:** The `capabilities()` API is being implemented in Week 20 Day 2.
> The example below shows the planned API. Until Day 2 is complete, use the
> internal detection mechanism described in the verification section.

EdgeVec detects ARM NEON SIMD at runtime:

```rust
// Coming in Week 20 Day 2 (neon.rs module creation)
use edgevec::simd::capabilities;

let caps = capabilities();
println!("NEON available: {}", caps.neon);
```

### Verification Under QEMU

```bash
# Run SIMD detection test with output
cross test --target aarch64-unknown-linux-gnu --release simd -- --nocapture
```

Expected output on ARM64:
```
NEON available: true
```

---

## Performance Considerations

### QEMU Emulation Overhead

| Operation | Native ARM64 | QEMU (x86 host) | Slowdown |
|:----------|:-------------|:----------------|:---------|
| Build | 1x | ~2-3x | Moderate |
| Tests | 1x | ~5-10x | Significant |
| Benchmarks | 1x | ~10-20x | Not recommended |

**Recommendation:** Use QEMU for correctness testing only. Run performance benchmarks on native ARM64 hardware.

### CI Optimizations

The CI workflow applies these optimizations for QEMU:
- `--test-threads=1` — Single-threaded execution (more stable)
- `PROPTEST_CASES=16` — Reduced proptest iterations
- `NUM_VECTORS=500` — Smaller test datasets

---

## Troubleshooting

### Docker Not Running

**Symptom:**
```
error: failed to run custom build command for `...`
```

**Solution:**
```bash
# Start Docker
sudo systemctl start docker  # Linux
# Or start Docker Desktop on macOS/Windows
```

### Cross Image Pull Fails

**Symptom:**
```
Error response from daemon: pull access denied
```

**Solution:**
```bash
# Login to GitHub Container Registry (if rate limited)
docker login ghcr.io

# Or use Docker Hub mirror
export CROSS_CONTAINER_OPTS="--platform linux/arm64"
```

### QEMU Test Timeout

**Symptom:**
```
Test timed out after 60 seconds
```

**Solution:**
```bash
# Increase timeout
cross test --target aarch64-unknown-linux-gnu --release -- --test-threads=1

# Or run specific tests
cross test --target aarch64-unknown-linux-gnu --release test_name
```

### NEON Detection Returns False

**Symptom:**
```
NEON available: false (on ARM64)
```

**Possible Causes:**
1. Running on x86 (expected behavior)
2. QEMU not emulating NEON (check QEMU version)
3. Old ARM64 without NEON (very rare)

**Verification:**
```bash
# Check architecture
cross run --target aarch64-unknown-linux-gnu --release -- uname -m
# Should output: aarch64
```

### Clippy Warnings on ARM64

**Symptom:**
```
warning: unused conditional compilation
```

**Solution:**
Ensure ARM-specific code is properly gated:
```rust
#[cfg(target_arch = "aarch64")]
fn arm_specific_function() {
    // ...
}
```

---

## Advanced Configuration

### Custom Cross Configuration

Create `Cross.toml` in project root:

```toml
[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main"

[build.env]
passthrough = ["PROPTEST_CASES", "NUM_VECTORS"]
```

### Local QEMU Installation (Alternative to Docker)

```bash
# Ubuntu
sudo apt install qemu-user qemu-user-static

# Configure linker
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="qemu-aarch64 -L /usr/aarch64-linux-gnu"

# Build without cross
cargo build --target aarch64-unknown-linux-gnu
```

---

## Related Documentation

- [SIMD Safety](./SIMD_SAFETY.md) — Safety documentation for SIMD code
- [Architecture](../architecture/ARCHITECTURE.md) — System architecture
- [WASM Boundary](../architecture/WASM_BOUNDARY.md) — WASM interface specification

---

## Version History

| Version | Date | Changes |
|:--------|:-----|:--------|
| 1.0.0 | 2025-12-23 | Initial ARM cross-compilation guide |

---

*EdgeVec ARM64 Cross-Compilation Guide*
*Week 20: ARM Infrastructure Sprint*
