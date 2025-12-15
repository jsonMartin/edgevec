// EdgeVec Batch Delete Demo (W18.5)
// Demonstrates batch delete API with browser compatibility

import init, { EdgeVecConfig } from '../../pkg/edgevec.js';
import { softDeleteBatch, getBrowserInfo, supportsBigUint64 } from '../../pkg/edgevec-wrapper.js';

// Global state
let index = null;
let startTime = Date.now();
let totalInserted = 0;

// Get browser info and update UI
function initBrowserInfo() {
    const info = getBrowserInfo();
    const modeEl = document.getElementById('browserMode');

    if (info.mode === 'modern') {
        modeEl.textContent = 'Modern';
        modeEl.classList.add('supported');
        log(`Browser supports BigUint64Array - using optimal batch delete`, 'success');
    } else {
        modeEl.textContent = 'Compat';
        modeEl.classList.remove('supported');
        log(`Safari 14 compat mode - IDs limited to 2^53`, 'warn');
        if (info.recommendation) {
            log(info.recommendation, 'warn');
        }
    }
}

// Logging utility
function log(msg, level = 'info') {
    const now = new Date();
    const elapsed = now - startTime;
    const timestamp = new Date(elapsed).toISOString().substr(11, 8);

    const logDiv = document.getElementById('terminalOutput');
    const line = document.createElement('div');
    line.className = 'log-line';

    line.innerHTML = `
        <span class="log-time">[${timestamp}]</span>
        <span class="log-level ${level}">${level.toUpperCase()}</span>
        <span class="log-message">${msg}</span>
    `;

    logDiv.appendChild(line);
    logDiv.scrollTop = logDiv.scrollHeight;
}

// Update stats display
function updateStats() {
    if (!index) return;

    try {
        const liveCount = index.liveCount();
        const deletedCount = index.deletedCount();
        const total = liveCount + deletedCount;
        const ratio = index.tombstoneRatio();

        document.getElementById('totalVectors').textContent = total.toLocaleString();
        document.getElementById('liveCount').textContent = liveCount.toLocaleString();
        document.getElementById('deletedCount').textContent = deletedCount.toLocaleString();
        document.getElementById('tombstoneRatio').textContent = `${(ratio * 100).toFixed(1)}%`;

        // Update button states
        const hasLive = liveCount > 0;
        document.getElementById('batchDeleteBtn').disabled = !hasLive;
        document.getElementById('deleteAllBtn').disabled = !hasLive;
        document.getElementById('compactBtn').disabled = !index.needsCompaction();

        // Check for compaction warning
        const warning = index.compactionWarning();
        if (warning) {
            log(warning, 'warn');
        }
    } catch (e) {
        log(`Stats update error: ${e.message}`, 'error');
    }
}

// Setup index with initial vectors
async function setupIndex() {
    const count = parseInt(document.getElementById('vectorCount').value);

    try {
        log(`Creating new EdgeVec index (128 dimensions)...`, 'info');

        const config = new EdgeVecConfig(128);
        config.m = 16;
        config.ef_construction = 200;
        index = new config.constructor(config);

        log(`Inserting ${count.toLocaleString()} random vectors...`, 'info');

        const insertStart = performance.now();
        totalInserted = 0;

        for (let i = 0; i < count; i++) {
            const vec = new Float32Array(128);
            for (let j = 0; j < 128; j++) {
                vec[j] = Math.random() * 2 - 1;
            }
            index.insert(vec);
            totalInserted++;

            if (totalInserted % 100 === 0) {
                log(`  Inserted ${totalInserted}/${count}...`, 'info');
            }
        }

        const insertTime = (performance.now() - insertStart).toFixed(0);
        log(`Setup complete: ${totalInserted} vectors in ${insertTime}ms`, 'success');
        log(`Average: ${(parseFloat(insertTime) / totalInserted).toFixed(3)}ms per vector`, 'info');

        updateStats();

    } catch (e) {
        log(`Setup failed: ${e.message}`, 'error');
        console.error(e);
    }
}

// Batch delete random vectors
async function batchDelete() {
    const batchSize = parseInt(document.getElementById('batchSize').value);

    if (!index) {
        log('No index - run Setup first', 'error');
        return;
    }

    const liveCount = index.liveCount();
    if (liveCount === 0) {
        log('No live vectors to delete', 'warn');
        return;
    }

    try {
        // Generate random IDs to delete
        const actualSize = Math.min(batchSize, liveCount);
        log(`Generating ${actualSize} random IDs to delete...`, 'info');

        const ids = [];
        const used = new Set();
        const totalCount = index.liveCount() + index.deletedCount();

        while (ids.length < actualSize) {
            const id = Math.floor(Math.random() * totalCount) + 1;
            if (!used.has(id)) {
                used.add(id);
                ids.push(id);
            }
        }

        log(`Calling softDeleteBatch with ${ids.length} IDs...`, 'info');

        // Use wrapper for browser compatibility
        const deleteStart = performance.now();
        const result = softDeleteBatch(index, ids);
        const deleteTime = (performance.now() - deleteStart).toFixed(3);

        log(`Batch delete complete in ${deleteTime}ms`, 'success');
        log(`  Deleted: ${result.deleted}`, 'success');
        log(`  Already deleted: ${result.alreadyDeleted}`, 'info');
        log(`  Invalid IDs: ${result.invalidIds}`, result.invalidIds > 0 ? 'warn' : 'info');
        log(`  Total/Unique: ${result.total}/${result.uniqueCount}`, 'info');

        // Update result card
        const resultCard = document.getElementById('lastBatchResult');
        resultCard.style.display = 'block';

        document.getElementById('resultDeleted').textContent = result.deleted;
        document.getElementById('resultAlreadyDeleted').textContent = result.alreadyDeleted;
        document.getElementById('resultInvalidIds').textContent = result.invalidIds;
        document.getElementById('resultTotalUnique').textContent = `${result.total} / ${result.uniqueCount}`;
        document.getElementById('resultDuration').textContent = `${deleteTime}ms`;

        // Set badge based on result
        const badge = document.getElementById('resultBadge');
        if (result.allValid()) {
            badge.textContent = 'All Valid';
            badge.className = 'result-badge badge-success';
        } else if (result.anyDeleted()) {
            badge.textContent = 'Partial Success';
            badge.className = 'result-badge badge-warning';
        } else {
            badge.textContent = 'No Deletions';
            badge.className = 'result-badge badge-error';
        }

        updateStats();

    } catch (e) {
        log(`Batch delete failed: ${e.message}`, 'error');
        console.error(e);
    }
}

// Delete all live vectors
async function deleteAllLive() {
    if (!index) {
        log('No index - run Setup first', 'error');
        return;
    }

    const liveCount = index.liveCount();
    if (liveCount === 0) {
        log('No live vectors to delete', 'warn');
        return;
    }

    try {
        log(`Deleting all ${liveCount} live vectors...`, 'warn');

        // Generate IDs for all vectors
        const totalCount = index.liveCount() + index.deletedCount();
        const allIds = Array.from({ length: totalCount }, (_, i) => i + 1);

        const deleteStart = performance.now();
        const result = softDeleteBatch(index, allIds);
        const deleteTime = (performance.now() - deleteStart).toFixed(3);

        log(`Delete all complete in ${deleteTime}ms`, 'success');
        log(`  Deleted: ${result.deleted}`, 'success');
        log(`  Already deleted: ${result.alreadyDeleted}`, 'info');

        updateStats();

    } catch (e) {
        log(`Delete all failed: ${e.message}`, 'error');
        console.error(e);
    }
}

// Compact the index
async function compactIndex() {
    if (!index) {
        log('No index - run Setup first', 'error');
        return;
    }

    if (!index.needsCompaction()) {
        log('Compaction not needed (tombstone ratio below threshold)', 'info');
        return;
    }

    try {
        log('Starting compaction...', 'warn');

        const result = index.compact();

        log(`Compaction complete in ${result.durationMs}ms`, 'success');
        log(`  Tombstones removed: ${result.tombstonesRemoved}`, 'success');
        log(`  New size: ${result.newSize}`, 'info');

        // Update compaction result card
        const compactCard = document.getElementById('compactionResult');
        compactCard.style.display = 'block';

        document.getElementById('compactRemoved').textContent = result.tombstonesRemoved;
        document.getElementById('compactNewSize').textContent = result.newSize;
        document.getElementById('compactDuration').textContent = `${result.durationMs}ms`;

        updateStats();

    } catch (e) {
        log(`Compaction failed: ${e.message}`, 'error');
        console.error(e);
    }
}

// Reset demo
function resetDemo() {
    log('Resetting demo...', 'info');

    if (index) {
        index.free();
        index = null;
    }

    totalInserted = 0;

    // Clear stats
    document.getElementById('totalVectors').textContent = '0';
    document.getElementById('liveCount').textContent = '0';
    document.getElementById('deletedCount').textContent = '0';
    document.getElementById('tombstoneRatio').textContent = '0%';

    // Hide result cards
    document.getElementById('lastBatchResult').style.display = 'none';
    document.getElementById('compactionResult').style.display = 'none';

    // Clear terminal
    const logDiv = document.getElementById('terminalOutput');
    logDiv.innerHTML = '';
    startTime = Date.now();

    log('Demo reset complete', 'success');

    updateStats();
}

// Initialize the demo
async function initDemo() {
    try {
        log('Initializing WASM module...', 'info');
        await init();
        log('WASM module loaded successfully', 'success');

        // Init browser info
        initBrowserInfo();

        // Attach event listeners
        document.getElementById('setupBtn').addEventListener('click', setupIndex);
        document.getElementById('batchDeleteBtn').addEventListener('click', batchDelete);
        document.getElementById('deleteAllBtn').addEventListener('click', deleteAllLive);
        document.getElementById('compactBtn').addEventListener('click', compactIndex);
        document.getElementById('resetBtn').addEventListener('click', resetDemo);

        log('Demo ready - click "Setup Index" to begin', 'success');

    } catch (e) {
        log(`Initialization failed: ${e.message}`, 'error');
        console.error(e);
        document.getElementById('wasmStatus').textContent = 'Failed';
        document.getElementById('wasmStatus').classList.remove('supported');
    }
}

// Start the demo
initDemo();
