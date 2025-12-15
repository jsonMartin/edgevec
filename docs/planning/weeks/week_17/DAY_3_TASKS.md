# Day 3 Tasks — W17.3: Browser Example + Cross-Browser Testing

**Date:** Week 17, Day 3
**Task ID:** W17.3
**Agent:** WASM_SPECIALIST
**Estimate:** 6h (2h base × 3x)
**Priority:** P1
**Status:** PENDING

---

## Objective

Create a working browser example demonstrating soft delete functionality and verify cross-browser compatibility per the W15.4 browser matrix.

---

## Prerequisites

- [ ] W17.1 complete (WASM bindings implemented)
- [ ] W17.2 complete (Tests passing)
- [x] Browser compatibility matrix from W15.4 (`docs/BROWSER_COMPATIBILITY.md`)
- [x] Existing example structure in `wasm/examples/`

---

## Implementation Checklist

### 1. Create Soft Delete Example (`wasm/examples/soft_delete.html`)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>EdgeVec Soft Delete Demo</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .card {
            background: white;
            border-radius: 8px;
            padding: 20px;
            margin: 10px 0;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        button {
            background: #2563eb;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            margin: 5px;
        }
        button:hover { background: #1d4ed8; }
        button:disabled { background: #9ca3af; cursor: not-allowed; }
        button.danger { background: #dc2626; }
        button.danger:hover { background: #b91c1c; }
        .stats { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
        .stat { text-align: center; padding: 10px; background: #f0f9ff; border-radius: 4px; }
        .stat-value { font-size: 24px; font-weight: bold; color: #2563eb; }
        .stat-label { font-size: 12px; color: #64748b; }
        .warning { background: #fef3c7; color: #92400e; padding: 10px; border-radius: 4px; margin: 10px 0; }
        .log { background: #1e1e1e; color: #d4d4d4; padding: 15px; border-radius: 4px;
               font-family: monospace; font-size: 12px; max-height: 200px; overflow-y: auto; }
        .log-entry { margin: 2px 0; }
        .log-entry.success { color: #4ade80; }
        .log-entry.error { color: #f87171; }
        .log-entry.info { color: #60a5fa; }
        #results { list-style: none; padding: 0; }
        #results li { padding: 8px; background: #f8fafc; margin: 4px 0; border-radius: 4px; }
    </style>
</head>
<body>
    <h1>EdgeVec Soft Delete Demo</h1>
    <p>Demonstrating v0.3.0 soft delete and compaction features.</p>

    <div class="card">
        <h2>Controls</h2>
        <button onclick="insertVectors(100)">Insert 100 Vectors</button>
        <button onclick="insertVectors(1000)">Insert 1000 Vectors</button>
        <button class="danger" onclick="deleteRandom(0.1)">Delete 10%</button>
        <button class="danger" onclick="deleteRandom(0.3)">Delete 30%</button>
        <button onclick="searchSimilar()">Search Top 10</button>
        <button onclick="runCompaction()">Compact</button>
    </div>

    <div class="card">
        <h2>Index Statistics</h2>
        <div class="stats">
            <div class="stat">
                <div class="stat-value" id="total">0</div>
                <div class="stat-label">Total Vectors</div>
            </div>
            <div class="stat">
                <div class="stat-value" id="live">0</div>
                <div class="stat-label">Live Vectors</div>
            </div>
            <div class="stat">
                <div class="stat-value" id="deleted">0</div>
                <div class="stat-label">Deleted (Tombstones)</div>
            </div>
        </div>
        <div class="stats" style="margin-top: 10px;">
            <div class="stat">
                <div class="stat-value" id="ratio">0%</div>
                <div class="stat-label">Tombstone Ratio</div>
            </div>
            <div class="stat">
                <div class="stat-value" id="compaction">No</div>
                <div class="stat-label">Needs Compaction</div>
            </div>
            <div class="stat">
                <div class="stat-value" id="size">0 KB</div>
                <div class="stat-label">Memory Est.</div>
            </div>
        </div>
        <div id="warning-box" class="warning" style="display: none;"></div>
    </div>

    <div class="card">
        <h2>Search Results</h2>
        <ul id="results">
            <li>Click "Search Top 10" to search</li>
        </ul>
    </div>

    <div class="card">
        <h2>Activity Log</h2>
        <div class="log" id="log"></div>
    </div>

    <script type="module">
        import init, { WasmIndex } from '../pkg/edgevec.js';

        let index = null;
        let dimension = 128;
        let insertedIds = [];

        window.log = function(message, type = 'info') {
            const logDiv = document.getElementById('log');
            const entry = document.createElement('div');
            entry.className = `log-entry ${type}`;
            entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
            logDiv.appendChild(entry);
            logDiv.scrollTop = logDiv.scrollHeight;
        };

        window.updateStats = function() {
            if (!index) return;

            const total = index.liveCount() + index.deletedCount();
            document.getElementById('total').textContent = total;
            document.getElementById('live').textContent = index.liveCount();
            document.getElementById('deleted').textContent = index.deletedCount();
            document.getElementById('ratio').textContent =
                (index.tombstoneRatio() * 100).toFixed(1) + '%';
            document.getElementById('compaction').textContent =
                index.needsCompaction() ? 'Yes' : 'No';
            document.getElementById('size').textContent =
                Math.round(total * dimension * 4 / 1024) + ' KB';

            const warning = index.compactionWarning();
            const warningBox = document.getElementById('warning-box');
            if (warning) {
                warningBox.textContent = warning;
                warningBox.style.display = 'block';
            } else {
                warningBox.style.display = 'none';
            }
        };

        window.insertVectors = async function(count) {
            const start = performance.now();
            for (let i = 0; i < count; i++) {
                const vector = new Float32Array(dimension);
                for (let j = 0; j < dimension; j++) {
                    vector[j] = Math.random();
                }
                const id = index.insert(vector);
                insertedIds.push(id);
            }
            const elapsed = performance.now() - start;
            log(`Inserted ${count} vectors in ${elapsed.toFixed(1)}ms`, 'success');
            updateStats();
        };

        window.deleteRandom = function(ratio) {
            const liveIds = insertedIds.filter(id => !index.isDeleted(id));
            const toDelete = Math.floor(liveIds.length * ratio);

            // Shuffle and take first N
            const shuffled = liveIds.sort(() => Math.random() - 0.5);
            const targets = shuffled.slice(0, toDelete);

            const start = performance.now();
            let deleted = 0;
            for (const id of targets) {
                if (index.softDelete(id)) {
                    deleted++;
                }
            }
            const elapsed = performance.now() - start;
            log(`Deleted ${deleted} vectors in ${elapsed.toFixed(1)}ms`, 'success');
            updateStats();
        };

        window.searchSimilar = function() {
            const query = new Float32Array(dimension);
            for (let i = 0; i < dimension; i++) {
                query[i] = Math.random();
            }

            const start = performance.now();
            const results = index.search(query, 10);
            const elapsed = performance.now() - start;

            const resultsList = document.getElementById('results');
            resultsList.innerHTML = '';

            if (results.length === 0) {
                resultsList.innerHTML = '<li>No results (all vectors deleted?)</li>';
            } else {
                results.forEach((r, i) => {
                    const li = document.createElement('li');
                    li.textContent = `#${i + 1}: ID ${r.vectorId} (distance: ${r.distance.toFixed(4)})`;
                    resultsList.appendChild(li);
                });
            }

            log(`Search returned ${results.length} results in ${elapsed.toFixed(2)}ms`, 'info');
        };

        window.runCompaction = async function() {
            log('Starting compaction...', 'info');
            const start = performance.now();

            try {
                const result = index.compact();
                const elapsed = performance.now() - start;

                log(`Compaction complete: removed ${result.tombstones_removed} tombstones, ` +
                    `new size: ${result.new_size}, took ${result.duration_ms}ms (total: ${elapsed.toFixed(1)}ms)`,
                    'success');

                // Update insertedIds to only include live ones
                insertedIds = insertedIds.filter(id => {
                    try {
                        return !index.isDeleted(id);
                    } catch {
                        return false; // ID no longer exists
                    }
                });

                updateStats();
            } catch (e) {
                log(`Compaction failed: ${e}`, 'error');
            }
        };

        // Initialize
        async function main() {
            await init();
            index = new WasmIndex(dimension, 16, 200);
            log('EdgeVec initialized (dimension: 128, M: 16, ef_construction: 200)', 'success');
            updateStats();
        }

        main().catch(e => log(`Init failed: ${e}`, 'error'));
    </script>
</body>
</html>
```

### 2. Create JavaScript Module (`wasm/examples/soft_delete.js`)

```javascript
// soft_delete.js - Modular version for advanced usage
import init, { WasmIndex } from '../pkg/edgevec.js';

export class SoftDeleteDemo {
    constructor(dimension = 128) {
        this.dimension = dimension;
        this.index = null;
        this.insertedIds = [];
    }

    async initialize() {
        await init();
        this.index = new WasmIndex(this.dimension, 16, 200);
        return this;
    }

    insert(count) {
        const ids = [];
        for (let i = 0; i < count; i++) {
            const vector = new Float32Array(this.dimension);
            for (let j = 0; j < this.dimension; j++) {
                vector[j] = Math.random();
            }
            const id = this.index.insert(vector);
            ids.push(id);
            this.insertedIds.push(id);
        }
        return ids;
    }

    deleteRandom(ratio) {
        const liveIds = this.insertedIds.filter(id => !this.index.isDeleted(id));
        const toDelete = Math.floor(liveIds.length * ratio);
        const shuffled = liveIds.sort(() => Math.random() - 0.5);
        const targets = shuffled.slice(0, toDelete);

        let deleted = 0;
        for (const id of targets) {
            if (this.index.softDelete(id)) {
                deleted++;
            }
        }
        return deleted;
    }

    search(k = 10) {
        const query = new Float32Array(this.dimension);
        for (let i = 0; i < this.dimension; i++) {
            query[i] = Math.random();
        }
        return this.index.search(query, k);
    }

    compact() {
        return this.index.compact();
    }

    getStats() {
        return {
            total: this.index.liveCount() + this.index.deletedCount(),
            live: this.index.liveCount(),
            deleted: this.index.deletedCount(),
            tombstoneRatio: this.index.tombstoneRatio(),
            needsCompaction: this.index.needsCompaction(),
            compactionWarning: this.index.compactionWarning()
        };
    }

    dispose() {
        if (this.index) {
            this.index.free();
            this.index = null;
        }
    }
}
```

### 3. Browser Compatibility Test Matrix

Test the example in all browsers from W15.4:

| Browser | Version | Status | Notes |
|:--------|:--------|:-------|:------|
| Chrome | 90+ | PENDING | Primary target |
| Firefox | 90+ | PENDING | WASM validation |
| Safari | 15+ | PENDING | WebKit engine |
| Edge | 90+ | PENDING | Chromium-based |

#### Test Checklist Per Browser

- [ ] Page loads without errors
- [ ] Insert 100 vectors works
- [ ] Insert 1000 vectors works
- [ ] Delete 10% works
- [ ] Delete 30% works
- [ ] Search returns results (excludes deleted)
- [ ] Compaction completes
- [ ] Stats update correctly
- [ ] Warning appears when threshold exceeded
- [ ] No console errors
- [ ] Memory doesn't leak (check DevTools)
- [ ] **Memory warning shown when attempting compact on >10k vectors**

#### Memory Warning Implementation

The example must show a warning when user attempts to compact indices with >10k vectors:

```javascript
window.runCompaction = async function() {
    const total = index.liveCount() + index.deletedCount();
    if (total > 10000) {
        if (!confirm(`Compaction on ${total} vectors may use significant memory (est. ${Math.round(total * 128 * 4 / 1024 / 1024)}MB). Continue?`)) {
            log('Compaction cancelled by user', 'info');
            return;
        }
    }
    // ... compaction code
};
```

### 4. Update Existing Examples Index

Update `wasm/examples/index.html` to include link to new example.

---

## Acceptance Criteria Verification

| AC | Verification | Expected |
|:---|:-------------|:---------|
| AC17.3.1 | File exists | YES |
| AC17.3.2 | Manual test | Visible |
| AC17.3.3 | Manual test | Visible |
| AC17.3.4 | Chrome test | PASS |
| AC17.3.5 | Firefox test | PASS |
| AC17.3.6 | Safari test | PASS |
| AC17.3.7 | Edge test | PASS |
| AC17.3.8 | DevTools | No errors |

---

## Output

### Artifacts Generated

- [ ] `wasm/examples/soft_delete.html` — Interactive demo
- [ ] `wasm/examples/soft_delete.js` — JS module
- [ ] Browser test results documented

### Status After Completion

```
✅ W17.3 COMPLETE
Next: W17.4 (Release Prep)
```

---

**Status:** PENDING
**Next:** `/wasm-bind example_soft_delete`
