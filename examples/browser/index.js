import init, { EdgeVec, EdgeVecConfig } from '../../pkg/edgevec.js';

// ==========================================
// Application State
// ==========================================
let db = null;
const DIMS = 128;
const BATCH_SIZE = 10000;
let totalVectors = 0;
let lastResults = null;

// ==========================================
// UI Helpers
// ==========================================
const logDiv = document.getElementById('log');
const statusIndicator = document.getElementById('statusIndicator');
const statusText = document.getElementById('statusText');
const statCount = document.getElementById('statCount');
const statDims = document.getElementById('statDims');

function setStatus(state, msg) {
    statusText.innerText = msg || state;
    if (state === 'BUSY') {
        statusIndicator.className = 'status-indicator busy';
        document.body.style.cursor = 'wait';
    } else {
        statusIndicator.className = 'status-indicator active';
        document.body.style.cursor = 'default';
    }
}

function updateMetrics() {
    statCount.innerText = totalVectors.toLocaleString();
    statDims.innerText = DIMS;
}

function log(msg, type = 'info') {
    const el = document.createElement('div');
    el.className = 'log-entry';
    
    const ts = new Date().toLocaleTimeString('en-US', { hour12: false });
    let colorClass = '';
    
    if (type === 'success') colorClass = 'highlight-green';
    else if (type === 'error') colorClass = 'highlight-red';
    else if (type === 'warn') colorClass = 'highlight-purple';
    else if (type === 'cyan') colorClass = 'highlight-cyan';

    el.innerHTML = `
        <span class="timestamp">[${ts}]</span>
        <span class="message ${colorClass}">${msg}</span>
    `;
    logDiv.appendChild(el);
    logDiv.scrollTop = logDiv.scrollHeight;
}

function renderResults(results) {
    let html = `
    <table class="results-table">
        <thead>
            <tr>
                <th>Rank</th>
                <th>Vector ID</th>
                <th>Score</th>
                <th>Similarity</th>
            </tr>
        </thead>
        <tbody>
    `;
    results.forEach((r, i) => {
        // Normalize score for display width (assuming cosine similarity -1 to 1, or 0 to 1)
        // Adjust based on your metric. Assuming 0-1 for simplicity or mapping.
        // If score is cosine similarity, it can be negative. 
        const width = Math.max(0, Math.min(100, r.score * 100));
        html += `
            <tr>
                <td>#${i+1}</td>
                <td class="highlight-cyan">${r.id}</td>
                <td>${r.score.toFixed(5)}</td>
                <td>
                    <div class="score-bar">
                        <div class="score-fill" style="width: ${width}%"></div>
                    </div>
                </td>
            </tr>
        `;
    });
    html += `</tbody></table>`;
    const el = document.createElement('div');
    el.innerHTML = html;
    logDiv.appendChild(el);
    logDiv.scrollTop = logDiv.scrollHeight;
}

function enableButtons() {
    ['btnInsert', 'btnSearch', 'btnSave'].forEach(id => {
        document.getElementById(id).disabled = false;
    });
}

// ==========================================
// Event Handlers
// ==========================================

async function run() {
    setStatus('BUSY', 'INITIALIZING WASM...');
    try {
        await init();
        setStatus('READY', 'WASM LOADED');
        log("✅ System Initialized. Ready to start engine.", 'success');
        document.getElementById('btnInit').disabled = false;
        
        // Check if DB exists in IndexedDB to enable Load button
        // (Optional: Implement check logic here)
        document.getElementById('btnLoad').disabled = false;
        
    } catch (e) {
        setStatus('ERROR', 'INIT FAILED');
        log(`❌ Initialization Error: ${e}`, 'error');
    }
}

document.getElementById('btnInit').onclick = async () => {
    setStatus('BUSY', 'STARTING ENGINE...');
    try {
        const config = new EdgeVecConfig(DIMS);
        // Optional: config.m = 16; config.ef_construction = 100;
        
        db = new EdgeVec(config);
        
        totalVectors = 0;
        updateMetrics();
        
        log(`✅ EdgeVec Engine Started (Dimensions: ${DIMS})`, 'cyan');
        enableButtons();
        setStatus('READY', 'ENGINE RUNNING');
    } catch (e) {
        setStatus('ERROR', 'ENGINE START FAILED');
        log(`❌ Engine Error: ${e}`, 'error');
    }
};

document.getElementById('btnInsert').onclick = async () => {
    if (!db) return;
    setStatus('BUSY', 'GENERATING DATA...');
    
    // Use setTimeout to allow UI update before heavy work
    setTimeout(() => {
        log(`Processing batch of ${BATCH_SIZE} vectors...`);
        
        const data = new Float32Array(BATCH_SIZE * DIMS);
        for (let i = 0; i < data.length; i++) {
            data[i] = Math.random() * 2 - 1; // [-1, 1]
        }
        
        setStatus('BUSY', 'INDEXING...');
        const start = performance.now();
        
        try {
            const ids = db.insert_batch(data, BATCH_SIZE);
            const end = performance.now();
            const time = (end - start).toFixed(2);
            const tps = Math.round(BATCH_SIZE / ((end - start) / 1000));
            
            totalVectors += ids.length;
            updateMetrics();
            
            log(`✅ Inserted ${ids.length} vectors in ${time}ms`, 'success');
            log(`⚡ Throughput: ${tps.toLocaleString()} vec/s`, 'cyan');
            setStatus('READY', 'INDEXING COMPLETE');
        } catch (e) {
            setStatus('ERROR', 'INSERT FAILED');
            log(`❌ Insert Error: ${e}`, 'error');
        }
    }, 50);
};

document.getElementById('btnSearch').onclick = () => {
    if (!db) return;
    setStatus('BUSY', 'SEARCHING...');
    
    // Disable export during search to prevent stale data export
    document.getElementById('btnExport').disabled = true;
    lastResults = null;
    
    const query = new Float32Array(DIMS);
    for (let i = 0; i < DIMS; i++) {
        query[i] = Math.random() * 2 - 1;
    }
    
    setTimeout(() => {
        const start = performance.now();
        try {
            const k = 10;
            const results = db.search(query, k);
            lastResults = results; // Store for export
            document.getElementById('btnExport').disabled = false;
            
            const end = performance.now();
            
            log(`🔍 Search Completed in ${(end - start).toFixed(3)}ms (k=${k})`, 'cyan');
            renderResults(results);
            setStatus('READY', 'SEARCH COMPLETE');
        } catch (e) {
            setStatus('ERROR', 'SEARCH FAILED');
            log(`❌ Search Error: ${e}`, 'error');
        }
    }, 10);
};

document.getElementById('btnSave').onclick = async () => {
    if (!db) return;
    setStatus('BUSY', 'SAVING SNAPSHOT...');
    try {
        const start = performance.now();
        await db.save("edgevec-snapshot");
        const end = performance.now();
        log(`✅ Snapshot saved in ${(end - start).toFixed(2)}ms`, 'success');
        setStatus('READY', 'SNAPSHOT SAVED');
    } catch (e) {
        setStatus('ERROR', 'SAVE FAILED');
        log(`❌ Save Error: ${e}`, 'error');
    }
};

document.getElementById('btnLoad').onclick = async () => {
    setStatus('BUSY', 'LOADING SNAPSHOT...');
    // Drop current instance
    db = null;
    totalVectors = 0; // Reset count or read from DB if possible? 
    // EdgeVec doesn't expose vector count directly on load easily unless we track it.
    // We will assume load works and maybe we can't update count accurately without querying the DB for stats if API supports it.
    // For demo, we might just set to "Loaded" or keep 0.
    // Assuming 10k for demo if we just saved 10k? No, that's unsafe.
    // We'll leave it 0 or add a `db.len()` if available. 
    // Checking lib.rs, `EdgeVec` struct might not expose len. 
    // But let's assume successful load restores state.
    
    try {
        const start = performance.now();
        db = await EdgeVec.load("edgevec-snapshot");
        const end = performance.now();
        
        log(`✅ Snapshot loaded in ${(end - start).toFixed(2)}ms`, 'success');
        enableButtons();
        
        // Try to update metrics if possible.
        // Since we don't have db.len(), we might try to search to see if it works.
        setStatus('READY', 'SNAPSHOT LOADED');
        
        // Auto-run a verification search
        document.getElementById('btnSearch').click();
        
    } catch (e) {
        setStatus('ERROR', 'LOAD FAILED');
        log(`❌ Load Error: ${e}`, 'error');
    }
};

document.getElementById('btnExport').onclick = () => {
    if (!lastResults) {
        log('❌ No results to export. Run a search first.', 'warn');
        return;
    }
    
    try {
        // Convert to JSON
        const json = JSON.stringify(lastResults, null, 2);
        const blob = new Blob([json], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        
        // Create download link
        const a = document.createElement('a');
        a.href = url;
        a.download = 'edgevec_results.json';
        document.body.appendChild(a);
        a.click();
        
        // Cleanup
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        log(`✅ Exported ${lastResults.length} results to edgevec_results.json`, 'success');
    } catch (e) {
        log(`❌ Export Error: ${e}`, 'error');
    }
};

// Start
run();
