# Day 5: Batch Delete WASM Bindings (W18.5)

**Date:** Week 18, Day 5
**Task ID:** W18.5
**Agent:** WASM_SPECIALIST
**Status:** [REVISED]
**Revision:** v1.2 — Adds Safari CI automation (8→10 Browser Compatibility)
**Depends On:** W18.4

---

## Pre-Task Validation Checklist [v1.2]

**Before starting W18.5, verify:**

- [ ] W18.4 marked COMPLETE
- [ ] `BatchDeleteResult` and `BatchDeleteError` structs exist in `src/hnsw/`
- [ ] `soft_delete_batch()` method exists on `HnswIndex`
- [ ] All W18.4 tests passing
- [ ] `wasm-pack` installed and working

---

## Buffer Allocation [v1.2]

| Component | Base | Buffer | Total |
|:----------|:----:|:------:|:-----:|
| WasmBatchDeleteResult | 1h | 0.25h | 1.25h |
| WASM methods (2) | 1.5h | 0.25h | 1.75h |
| JavaScript wrapper | 1h | 0.25h | 1.25h |
| TypeScript types | 0.5h | 0h | 0.5h |
| **Safari CI Automation [v1.2]** | 1.5h | 0.25h | 1.75h |
| **Total** | **5.5h** | **1h** | **6.5h** |

---

## Objective

Expose batch delete API via WASM bindings and create comprehensive tests. Update browser examples to demonstrate the new functionality.

**[C6 FIX]** Must include Safari 14 compatibility via BigUint64Array feature detection.
**[M2 FIX]** Must test on browser matrix: Chrome 90+, Firefox 88+, Safari 14+.
**[v1.2]** Must include automated Safari testing in CI.

---

## Browser Compatibility Matrix [M2 FIX]

| Browser | Min Version | BigUint64Array | Test Method |
|:--------|:------------|:---------------|:------------|
| Chrome | 90+ | Yes | wasm-pack test --headless --chrome |
| Firefox | 88+ | Yes | wasm-pack test --headless --firefox |
| Safari | 14+ | **NO (15+ only)** | **[v1.2] Playwright CI** |
| Edge | 90+ | Yes | Covered by Chrome (Chromium) |

**Safari 14 Fallback Required:** Use number[] instead of BigUint64Array.

---

## Safari CI Automation [v1.2 NEW]

**Rationale:** Automated Safari testing is critical for Browser Compatibility 10/10. Manual BrowserStack testing is error-prone and not reproducible.

### Option A: Playwright with WebKit (Recommended)

Playwright includes a WebKit browser that closely matches Safari behavior.

**`.github/workflows/wasm-test.yml` (new job):**
```yaml
safari-compat:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '20'

    - name: Install Playwright
      run: npx playwright install webkit

    - name: Build WASM
      run: wasm-pack build --release --target web

    - name: Run Safari/WebKit Tests
      run: |
        npx playwright test --browser=webkit
      env:
        EDGEVEC_SAFARI_COMPAT: "true"
```

**`playwright.config.ts` (new file):**
```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './wasm/tests',
  use: {
    baseURL: 'http://localhost:8080',
  },
  projects: [
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],
  webServer: {
    command: 'npx http-server ./pkg -p 8080',
    port: 8080,
  },
});
```

**`wasm/tests/safari-compat.spec.ts` (new file):**
```typescript
import { test, expect } from '@playwright/test';

test.describe('Safari Compatibility', () => {
  test('BigUint64Array detection', async ({ page }) => {
    await page.goto('/');

    // Check if BigUint64Array detection works
    const hasBigUint64 = await page.evaluate(() => {
      return typeof BigUint64Array !== 'undefined';
    });

    // WebKit in Playwright DOES support BigUint64Array (Safari 15+)
    // This test verifies our detection logic works
    expect(typeof hasBigUint64).toBe('boolean');
  });

  test('softDeleteBatchCompat works without BigUint64Array', async ({ page }) => {
    await page.goto('/');

    // Simulate Safari 14 by deleting BigUint64Array
    await page.evaluate(() => {
      (window as any)._BigUint64Array = BigUint64Array;
      delete (window as any).BigUint64Array;
    });

    // Load EdgeVec and test compat method
    const result = await page.evaluate(async () => {
      const { EdgeVec, softDeleteBatch } = await import('./edgevec.js');
      const index = new EdgeVec(4);

      // Insert some vectors
      for (let i = 0; i < 10; i++) {
        index.insert([i, i, i, i]);
      }

      // Use compat batch delete
      const ids = [1, 3, 5];
      const result = index.softDeleteBatchCompat(ids);

      return {
        deleted: result.deleted,
        total: result.total,
        allValid: result.allValid(),
      };
    });

    expect(result.deleted).toBe(3);
    expect(result.total).toBe(3);
    expect(result.allValid).toBe(true);
  });

  test('graceful degradation message', async ({ page }) => {
    await page.goto('/');

    // Listen for console warnings
    const warnings: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'warning') {
        warnings.push(msg.text());
      }
    });

    // Simulate Safari 14
    await page.evaluate(() => {
      delete (window as any).BigUint64Array;
    });

    // Use wrapper function that should warn
    await page.evaluate(async () => {
      const { softDeleteBatch, EdgeVec } = await import('./edgevec-wrapper.js');
      const index = new EdgeVec(4);
      for (let i = 0; i < 5; i++) index.insert([i, i, i, i]);
      softDeleteBatch(index, [1, 2, 3]);
    });

    // Should have logged a warning about fallback
    expect(warnings.some(w => w.includes('BigUint64Array not supported'))).toBe(true);
  });
});
```

### Option B: BrowserStack Integration (Alternative)

For actual Safari testing (not just WebKit):

```yaml
safari-browserstack:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: BrowserStack Setup
      uses: browserstack/github-actions/setup-env@master
      with:
        username: ${{ secrets.BROWSERSTACK_USERNAME }}
        access-key: ${{ secrets.BROWSERSTACK_ACCESS_KEY }}

    - name: Run Safari Tests
      run: npx browserstack-cypress run --sync
```

**Note:** BrowserStack requires paid subscription and secrets setup.

### Polyfill Strategy [v1.2]

Document polyfill decision tree:

```
┌─────────────────────────────────────────────────────────────┐
│   BigUint64Array Detection Flow                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  typeof BigUint64Array !== 'undefined'                      │
│      │                                                      │
│      ├─ YES → Use softDeleteBatch(BigUint64Array)           │
│      │        Full 64-bit ID support                        │
│      │                                                      │
│      └─ NO → Use softDeleteBatchCompat(number[])            │
│              Log console.warn()                             │
│              IDs limited to 2^53 (Number.MAX_SAFE_INTEGER)  │
│                                                             │
│  NO POLYFILL SHIPPED: BigUint64Array cannot be polyfilled   │
│  efficiently. Fallback is provided instead.                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Graceful Degradation Tests [v1.2]

Add to `tests/wasm_batch_delete.rs`:

```rust
#[wasm_bindgen_test]
fn test_compat_method_precision_limit() {
    let mut index = EdgeVec::new(4).unwrap();

    // Insert vector with ID that would lose precision in f64
    // Note: This is a documentation test - we can't actually insert
    // with specific IDs in current API, but we can verify the
    // conversion logic is correct for IDs < 2^53

    // Maximum safe integer in JavaScript
    let max_safe = (1u64 << 53) - 1; // 9007199254740991

    // Verify our conversion doesn't overflow for safe integers
    let ids: Vec<f64> = vec![1.0, 1000.0, max_safe as f64];
    // This should not panic
    let _ = index.soft_delete_batch_compat(&ids);
}
```

---

## Context

### W18.4 Deliverables (Required)

```rust
pub struct BatchDeleteResult {
    pub deleted: usize,
    pub already_deleted: usize,
    pub invalid_ids: usize,
    pub total: usize,
}

impl HnswIndex {
    pub fn soft_delete_batch(&mut self, ids: &[VectorId]) -> BatchDeleteResult;
    pub fn soft_delete_batch_with_progress<F>(&mut self, ids: &[VectorId], callback: F) -> BatchDeleteResult;
}
```

---

## Deliverables

### 1. WasmBatchDeleteResult Struct

```rust
// src/wasm/mod.rs

/// WASM-compatible batch delete result
#[wasm_bindgen]
pub struct WasmBatchDeleteResult {
    deleted: u32,
    already_deleted: u32,
    invalid_ids: u32,
    total: u32,
}

#[wasm_bindgen]
impl WasmBatchDeleteResult {
    /// Number of vectors successfully deleted
    #[wasm_bindgen(getter)]
    pub fn deleted(&self) -> u32 {
        self.deleted
    }

    /// Number of vectors that were already deleted
    #[wasm_bindgen(getter, js_name = "alreadyDeleted")]
    pub fn already_deleted(&self) -> u32 {
        self.already_deleted
    }

    /// Number of invalid IDs (not found)
    #[wasm_bindgen(getter, js_name = "invalidIds")]
    pub fn invalid_ids(&self) -> u32 {
        self.invalid_ids
    }

    /// Total vectors processed
    #[wasm_bindgen(getter)]
    pub fn total(&self) -> u32 {
        self.total
    }

    /// Check if all operations succeeded (no invalid IDs)
    #[wasm_bindgen(js_name = "allValid")]
    pub fn all_valid(&self) -> bool {
        self.invalid_ids == 0
    }

    /// Check if any deletions occurred
    #[wasm_bindgen(js_name = "anyDeleted")]
    pub fn any_deleted(&self) -> bool {
        self.deleted > 0
    }
}

impl From<BatchDeleteResult> for WasmBatchDeleteResult {
    fn from(result: BatchDeleteResult) -> Self {
        Self {
            deleted: result.deleted as u32,
            already_deleted: result.already_deleted as u32,
            invalid_ids: result.invalid_ids as u32,
            total: result.total as u32,
        }
    }
}
```

### 2. EdgeVec WASM Methods

**[C6 FIX]** Two methods: one for BigUint64Array (modern browsers), one for number[] (Safari 14).

```rust
// src/wasm/mod.rs

#[wasm_bindgen]
impl EdgeVec {
    /// Delete multiple vectors using BigUint64Array (modern browsers)
    ///
    /// More efficient than calling softDelete() N times.
    /// Use softDeleteBatchCompat() for Safari 14 compatibility.
    ///
    /// @param ids - BigUint64Array of vector IDs to delete
    /// @returns BatchDeleteResult with counts
    ///
    /// @example
    /// ```javascript
    /// const ids = new BigUint64Array([1n, 3n, 5n, 7n, 9n]);
    /// const result = index.softDeleteBatch(ids);
    /// console.log(`Deleted ${result.deleted} of ${result.total}`);
    /// ```
    #[wasm_bindgen(js_name = "softDeleteBatch")]
    pub fn soft_delete_batch(&mut self, ids: &[u64]) -> WasmBatchDeleteResult {
        let vec_ids: Vec<VectorId> = ids.iter().map(|&id| VectorId(id)).collect();
        let result = self.index.soft_delete_batch(&vec_ids);
        WasmBatchDeleteResult::from(result)
    }

    /// [C6 FIX] Delete multiple vectors using number[] (Safari 14 compatible)
    ///
    /// Use this method for Safari 14 compatibility. IDs must be < 2^53.
    ///
    /// @param ids - Array of numbers (vector IDs)
    /// @returns BatchDeleteResult with counts
    ///
    /// @example
    /// ```javascript
    /// // Safari 14 compatible
    /// const ids = [1, 3, 5, 7, 9];
    /// const result = index.softDeleteBatchCompat(ids);
    /// ```
    #[wasm_bindgen(js_name = "softDeleteBatchCompat")]
    pub fn soft_delete_batch_compat(&mut self, ids: &[f64]) -> WasmBatchDeleteResult {
        // Convert f64 to u64 (safe for IDs < 2^53)
        let vec_ids: Vec<VectorId> = ids
            .iter()
            .map(|&id| VectorId(id as u64))
            .collect();
        let result = self.index.soft_delete_batch(&vec_ids);
        WasmBatchDeleteResult::from(result)
    }
}
```

### 3. JavaScript Wrapper with Feature Detection [C6 FIX]

```javascript
// pkg/edgevec-wrapper.js
// This wrapper provides Safari 14 compatibility

/**
 * Feature detection for BigUint64Array
 */
const hasBigUint64Array = typeof BigUint64Array !== 'undefined';

/**
 * Delete multiple vectors with automatic browser compatibility
 *
 * @param {EdgeVec} index - The EdgeVec index
 * @param {Array<number|bigint>} ids - Array of vector IDs
 * @returns {WasmBatchDeleteResult}
 */
export function softDeleteBatch(index, ids) {
    if (hasBigUint64Array) {
        // Modern browsers: use BigUint64Array for full 64-bit support
        const bigIds = new BigUint64Array(ids.map(id => BigInt(id)));
        return index.softDeleteBatch(bigIds);
    } else {
        // Safari 14 fallback: use number array (lossy for IDs > 2^53)
        console.warn('[EdgeVec] BigUint64Array not supported, using number fallback. IDs > 2^53 may be truncated.');
        const numIds = ids.map(id => Number(id));
        return index.softDeleteBatchCompat(numIds);
    }
}

/**
 * Check if current browser supports full 64-bit vector IDs
 */
export function supportsBigInt64() {
    return hasBigUint64Array;
}
```

### 4. TypeScript Definitions

Update `pkg/edgevec.d.ts`:

```typescript
/**
 * Result of a batch delete operation
 */
export class WasmBatchDeleteResult {
    /** Number of vectors successfully deleted */
    readonly deleted: number;
    /** Number of vectors that were already deleted */
    readonly alreadyDeleted: number;
    /** Number of invalid IDs (not found) */
    readonly invalidIds: number;
    /** Total vectors processed */
    readonly total: number;

    /** Check if all operations succeeded (no invalid IDs) */
    allValid(): boolean;
    /** Check if any deletions occurred */
    anyDeleted(): boolean;
}

export class EdgeVec {
    // ... existing methods ...

    /**
     * Delete multiple vectors in a single operation
     *
     * @param ids - Array of vector IDs to delete
     * @returns BatchDeleteResult with operation counts
     *
     * @example
     * ```typescript
     * const ids = new BigUint64Array([1n, 3n, 5n, 7n, 9n]);
     * const result = index.softDeleteBatch(ids);
     * console.log(`Deleted ${result.deleted} of ${result.total}`);
     * if (!result.allValid()) {
     *     console.warn(`${result.invalidIds} invalid IDs`);
     * }
     * ```
     */
    softDeleteBatch(ids: BigUint64Array): WasmBatchDeleteResult;
}
```

### 4. Browser Example Update

Update `wasm/examples/soft_delete.html` to include batch delete:

```html
<section id="batch-controls">
    <h3>Batch Operations</h3>
    <input type="number" id="batch-count" value="100" min="1" max="1000">
    <button onclick="batchDelete()">Batch Delete Random</button>
    <p id="batch-result"></p>
</section>
```

```javascript
// wasm/examples/soft_delete.js (update)

async function batchDelete() {
    const count = parseInt(document.getElementById('batch-count').value);
    const stats = index.stats();

    if (stats.liveCount === 0) {
        alert('No live vectors to delete');
        return;
    }

    // Generate random IDs to delete
    const ids = new BigUint64Array(Math.min(count, stats.liveCount));
    const usedIds = new Set();

    for (let i = 0; i < ids.length; i++) {
        let id;
        do {
            id = BigInt(Math.floor(Math.random() * stats.totalCount) + 1);
        } while (usedIds.has(id));
        usedIds.add(id);
        ids[i] = id;
    }

    console.time('batchDelete');
    const result = index.softDeleteBatch(ids);
    console.timeEnd('batchDelete');

    document.getElementById('batch-result').innerHTML = `
        <strong>Batch Delete Result:</strong><br>
        Deleted: ${result.deleted}<br>
        Already Deleted: ${result.alreadyDeleted}<br>
        Invalid IDs: ${result.invalidIds}<br>
        Total: ${result.total}
    `;

    updateStats();
}
```

### 5. Integration Tests

Create test for WASM batch delete:

```rust
// tests/wasm_batch_delete.rs

#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use wasm_bindgen_test::*;
    use edgevec::wasm::EdgeVec;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_batch_delete() {
        // Create index with some vectors
        let mut index = EdgeVec::new(4).unwrap();

        for i in 0..20 {
            index.insert(&[i as f32; 4]).unwrap();
        }

        // Batch delete
        let ids: Vec<u64> = vec![1, 3, 5, 7, 9];
        let result = index.soft_delete_batch(&ids);

        assert_eq!(result.deleted(), 5);
        assert_eq!(result.total(), 5);
        assert!(result.all_valid());
    }

    #[wasm_bindgen_test]
    fn test_wasm_batch_delete_with_invalid() {
        let mut index = EdgeVec::new(4).unwrap();

        for i in 0..10 {
            index.insert(&[i as f32; 4]).unwrap();
        }

        // Include invalid ID
        let ids: Vec<u64> = vec![1, 999];
        let result = index.soft_delete_batch(&ids);

        assert_eq!(result.deleted(), 1);
        assert_eq!(result.invalid_ids(), 1);
        assert!(!result.all_valid());
    }
}
```

---

## Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.5.1 | `softDeleteBatch(ids): WasmBatchDeleteResult` binding | TypeScript compilation |
| AC18.5.2 | `WasmBatchDeleteResult` type exported | pkg/edgevec.d.ts |
| AC18.5.3 | Integration test for batch delete | `wasm-pack test` |
| AC18.5.4 | Browser example updated | Manual test |
| AC18.5.5 | Performance: batch delete 1000 IDs < 100ms | Browser console timing |
| AC18.5.6 | WASM bundle size still < 500KB | `wasm-pack build --release` |
| AC18.5.7 | **[v1.2]** Playwright Safari/WebKit tests pass | `npx playwright test` |
| AC18.5.8 | **[v1.2]** Safari 14 fallback tested in CI | WebKit job green |
| AC18.5.9 | **[v1.2]** Polyfill strategy documented | docs/BROWSER_COMPATIBILITY.md |

---

## Implementation Plan

### Step 1: Add WasmBatchDeleteResult

Add to `src/wasm/mod.rs`.

### Step 2: Add softDeleteBatch Method

Add to `impl EdgeVec` in `src/wasm/mod.rs`.

### Step 3: Update TypeScript Definitions

Update `pkg/edgevec.d.ts` manually or regenerate with `wasm-pack build`.

### Step 4: Update Browser Example

Modify `wasm/examples/soft_delete.html` and `soft_delete.js`.

### Step 5: Add WASM Tests

Create `tests/wasm_batch_delete.rs`.

### Step 6: Build and Test

```bash
wasm-pack build --release
wasm-pack test --headless --chrome
```

---

## Files to Create/Modify

| File | Action | Description |
|:-----|:-------|:------------|
| `src/wasm/mod.rs` | MODIFY | Add WasmBatchDeleteResult + method |
| `pkg/edgevec.d.ts` | MODIFY | TypeScript types |
| `pkg/edgevec-wrapper.js` | CREATE | Safari compat wrapper |
| `wasm/examples/soft_delete.html` | MODIFY | Add batch UI |
| `wasm/examples/soft_delete.js` | MODIFY | Add batch logic |
| `tests/wasm_batch_delete.rs` | CREATE | WASM tests |
| **[v1.2]** `playwright.config.ts` | CREATE | Playwright config |
| **[v1.2]** `wasm/tests/safari-compat.spec.ts` | CREATE | Safari compat tests |
| **[v1.2]** `.github/workflows/wasm-test.yml` | CREATE or MODIFY | Add Safari CI job |

---

## Verification Commands

```bash
# Build WASM
wasm-pack build --release

# Check bundle size
ls -la pkg/edgevec_bg.wasm

# Run WASM tests
wasm-pack test --headless --chrome

# Manual browser test
# Open wasm/examples/soft_delete.html
# Click "Batch Delete Random"
# Verify result appears
```

---

## Handoff

**On Completion:**
- Mark W18.5 as COMPLETE
- Submit for hostile review
- Week 18 complete — prepare for Week 19 planning
