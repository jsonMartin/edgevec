# Week 14 — Day 1 Tasks (Monday, Dec 23)

**Date:** 2025-12-23
**Focus:** WASM Progress Callback + P99 CI Start
**Agent:** WASM_SPECIALIST, BENCHMARK_SCIENTIST
**Status:** [REVISED]

---

## Day Objective

Add progress callback to existing WASM batch insert implementation and begin P99 latency tracking CI setup.

**Success Criteria:**
- Progress callback function implemented
- Browser demo created
- GitHub Actions workflow skeleton created

---

## Existing Implementation Acknowledgment

**WASM batch insert is ALREADY COMPLETE:**
- `insert_batch_flat()` at `src/wasm/mod.rs:261-311`
- `insertBatch()` (v2) at `src/wasm/mod.rs:341-348`
- 15 unit tests in `src/wasm/batch.rs`

**Today's work is ENHANCEMENT only, not new implementation.**

---

## Tasks

### W14.1: WASM Batch Insert Enhancement

**Priority:** P0 (Critical Path)
**Estimate:** 4h (base: 1.3h × 3x)
**Agent:** WASM_SPECIALIST

#### Scope

- [ ] **AC14.1.1:** Verify existing `insertBatch` works (VERIFY ONLY)
- [ ] **AC14.1.2:** Verify existing API accepts Float32Array (VERIFY ONLY)
- [ ] **AC14.1.3:** Verify existing API returns IDs (VERIFY ONLY)
- [ ] **AC14.1.4:** **NEW:** Implement progress callback
- [ ] **AC14.1.5:** Create browser demo

#### Implementation Specification

**Progress Callback (NEW):**

```rust
// Add to src/wasm/mod.rs

/// Batch insert with progress callback.
///
/// # Arguments
/// * `vectors` - JS Array of Float32Array vectors
/// * `on_progress` - JS function called with (inserted, total)
///
/// # Example (JavaScript)
/// ```javascript
/// const ids = index.insertBatchWithProgress(vectors, (done, total) => {
///     console.log(`Progress: ${done}/${total}`);
/// });
/// ```
#[wasm_bindgen(js_name = insertBatchWithProgress)]
pub fn insert_batch_with_progress(
    &mut self,
    vectors: Array,
    on_progress: &js_sys::Function,
) -> Result<batch::BatchInsertResult, JsValue> {
    let this = JsValue::NULL;
    let total = vectors.length();

    // Reuse existing batch logic but add progress calls
    let config = batch::BatchInsertConfig::new();

    // For now, call progress at start and end
    // Future: integrate with batch_insert internals for per-vector progress
    let _ = on_progress.call2(&this, &JsValue::from(0u32), &JsValue::from(total));

    let result = batch::insert_batch_impl(self, vectors, Some(config))?;

    let _ = on_progress.call2(&this, &JsValue::from(total), &JsValue::from(total));

    Ok(result)
}
```

**Browser Demo:**

```html
<!-- examples/wasm_batch_insert.html -->
<!DOCTYPE html>
<html>
<head>
    <title>EdgeVec Batch Insert Demo</title>
    <style>
        body { font-family: sans-serif; max-width: 800px; margin: 40px auto; }
        #progress { margin: 20px 0; }
        #results { background: #f0f0f0; padding: 20px; border-radius: 8px; }
        button { padding: 10px 20px; font-size: 16px; cursor: pointer; }
    </style>
</head>
<body>
    <h1>EdgeVec WASM Batch Insert</h1>
    <button id="run">Run Batch Insert (1000 vectors)</button>
    <div id="progress"></div>
    <div id="results"></div>

    <script type="module">
        import init, { EdgeVec, EdgeVecConfig } from '../pkg/edgevec.js';

        document.getElementById('run').onclick = async () => {
            await init();

            const config = new EdgeVecConfig();
            config.dimensions = 128;
            const index = new EdgeVec(config);

            // Generate 1000 random vectors
            const count = 1000;
            const dims = 128;
            const vectors = [];
            for (let i = 0; i < count; i++) {
                const vec = new Float32Array(dims);
                for (let j = 0; j < dims; j++) {
                    vec[j] = Math.random();
                }
                vectors.push(vec);
            }

            const progressDiv = document.getElementById('progress');
            const resultsDiv = document.getElementById('results');

            const start = performance.now();

            // Use insertBatchWithProgress if available, else fall back
            let result;
            if (index.insertBatchWithProgress) {
                result = index.insertBatchWithProgress(vectors, (inserted, total) => {
                    progressDiv.textContent = `Progress: ${inserted}/${total}`;
                });
            } else {
                progressDiv.textContent = 'Using legacy batch insert...';
                result = index.insertBatch(vectors);
            }

            const elapsed = performance.now() - start;

            resultsDiv.innerHTML = `
                <p><strong>Inserted:</strong> ${result.inserted} vectors</p>
                <p><strong>Time:</strong> ${elapsed.toFixed(2)}ms</p>
                <p><strong>Rate:</strong> ${(count / elapsed * 1000).toFixed(0)} vectors/sec</p>
                <p><strong>First 5 IDs:</strong> ${Array.from(result.ids.slice(0, 5)).join(', ')}</p>
            `;
        };
    </script>
</body>
</html>
```

#### Verification Commands

```bash
# Build WASM
wasm-pack build --target web --release

# Verify existing exports
grep "insertBatch" pkg/edgevec.d.ts

# Verify new export
grep "insertBatchWithProgress" pkg/edgevec.d.ts

# Run existing tests (should still pass)
cargo test --lib wasm::batch

# Open browser demo (requires local server)
python -m http.server 8080
# Then open http://localhost:8080/examples/wasm_batch_insert.html
```

---

### W14.2 (Part 1): P99 Latency CI Setup

**Priority:** P0 (Critical Path)
**Estimate:** 2h (part of 8h total)
**Agent:** BENCHMARK_SCIENTIST

#### Scope

- [ ] **AC14.2.1:** Create `.github/workflows/benchmark.yml`
- [ ] **AC14.2.2:** Verify existing `benches/baselines.json`

#### Implementation Specification

**GitHub Actions Workflow:**

```yaml
# .github/workflows/benchmark.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  benchmark:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-bench-${{ hashFiles('**/Cargo.lock') }}

      - name: Run benchmarks
        run: |
          cargo bench --bench search_bench -- --noplot
          cargo bench --bench insert_bench -- --noplot

      - name: Check regression
        run: python scripts/check_regression.py
        continue-on-error: false

      - name: Upload results
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: benchmark-results
          path: target/criterion/
```

#### Verification Commands

```bash
# Verify baselines exist
cat benches/baselines.json | python -m json.tool

# Validate workflow syntax (if actionlint available)
actionlint .github/workflows/benchmark.yml
```

---

## Day 1 Summary

**Total Effort:** 6h scheduled

**Deliverables:**
1. `insertBatchWithProgress()` function in `src/wasm/mod.rs`
2. Browser demo at `examples/wasm_batch_insert.html`
3. GitHub Actions workflow at `.github/workflows/benchmark.yml`

**Carryover to Day 2:**
- Complete regression detection script
- Test CI workflow on a PR

---

## HOSTILE_REVIEWER Pre-Flight (Day 1)

Before end of day:

- [ ] Existing batch insert still works (no regression)
- [ ] New progress callback compiles
- [ ] Browser demo loads without errors
- [ ] CI workflow syntax valid

---

**Status:** [REVISED]
**Next:** Complete W14.2 and start W14.3
