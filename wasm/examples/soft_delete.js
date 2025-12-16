/**
 * EdgeVec Soft Delete Demo - JavaScript Module
 * Version: 0.3.0 | RFC-001 Compliant
 *
 * Provides a reusable class for soft delete demonstrations and testing.
 * Import this module for programmatic access to soft delete functionality.
 * Uses dynamic imports for file:// protocol compatibility.
 *
 * @example
 * import { SoftDeleteDemo } from './soft_delete.js';
 *
 * const demo = new SoftDeleteDemo(128);
 * await demo.initialize();
 *
 * demo.insert(1000);
 * demo.deleteRandom(0.3);
 * console.log(demo.getStats());
 *
 * if (demo.needsCompaction()) {
 *   const result = demo.compact();
 *   console.log(`Removed ${result.tombstones_removed} tombstones`);
 * }
 */

// WASM module paths - works when serving from PROJECT ROOT
// IMPORTANT: Start server from edgevec/ root, NOT from wasm/examples/
// Example: cd edgevec && python -m http.server 8080
//          Then open: http://localhost:8080/wasm/examples/soft_delete.html
const WASM_PATHS = [
    '../../pkg/edgevec.js',      // From /wasm/examples/ → /pkg/
    '/pkg/edgevec.js',           // Absolute from root
    '../pkg/edgevec.js',         // From /wasm/ → /pkg/
    './pkg/edgevec.js'           // Fallback
];

// Module-level WASM reference (loaded dynamically)
let wasmModule = null;

/**
 * Soft Delete Demo Class
 *
 * Demonstrates EdgeVec v0.3.0 soft delete functionality:
 * - O(1) tombstone-based deletion
 * - Automatic compaction threshold detection
 * - Index compaction for space reclamation
 */
export class SoftDeleteDemo {
    /**
     * Create a new SoftDeleteDemo instance
     * @param {number} dimension - Vector dimension (default: 128, must be positive integer)
     * @param {object} options - Optional configuration
     * @param {number} options.m - HNSW M parameter (default: 16)
     * @param {number} options.efConstruction - HNSW ef_construction (default: 200)
     * @param {number} options.compactionThreshold - Threshold for compaction (default: 0.3)
     * @throws {Error} If dimension is not a positive integer
     */
    constructor(dimension = 128, options = {}) {
        // MAJ-2 FIX: Validate dimension parameter
        if (!Number.isInteger(dimension) || dimension < 1 || dimension > 65536) {
            throw new Error(`Invalid dimension: ${dimension}. Must be a positive integer between 1 and 65536.`);
        }
        this.dimension = dimension;
        this.m = options.m || 16;
        this.efConstruction = options.efConstruction || 200;
        this.compactionThreshold = options.compactionThreshold || 0.3;
        this.index = null;
        this.insertedIds = [];
        this.initialized = false;
        this.listeners = {
            insert: [],
            delete: [],
            compact: [],
            search: [],
            error: []
        };
    }

    /**
     * Initialize the WASM module and create the index
     * @returns {Promise<SoftDeleteDemo>} This instance for chaining
     */
    async initialize() {
        if (this.initialized) {
            return this;
        }

        // Load WASM module dynamically if not already loaded
        if (!wasmModule) {
            for (const path of WASM_PATHS) {
                try {
                    console.log(`[SoftDeleteDemo] Trying WASM path: ${path}`);
                    wasmModule = await import(path);
                    console.log(`[SoftDeleteDemo] Loaded WASM from: ${path}`);
                    break;
                } catch (e) {
                    console.warn(`[SoftDeleteDemo] Failed to load from ${path}: ${e.message}`);
                }
            }

            if (!wasmModule) {
                throw new Error('Could not load WASM module from any path');
            }

            // Initialize the WASM module
            await wasmModule.default();
        }

        const config = new wasmModule.EdgeVecConfig(this.dimension);
        this.index = new wasmModule.EdgeVec(config);
        this.initialized = true;

        this._emit('insert', { type: 'init', dimension: this.dimension });
        return this;
    }

    /**
     * Ensure the demo is initialized
     * @private
     */
    _ensureInitialized() {
        if (!this.initialized || !this.index) {
            throw new Error('SoftDeleteDemo not initialized. Call initialize() first.');
        }
    }

    /**
     * Emit an event to listeners
     * @private
     */
    _emit(event, data) {
        this.listeners[event]?.forEach(fn => fn(data));
    }

    /**
     * Add event listener
     * @param {string} event - Event name (insert, delete, compact, search, error)
     * @param {Function} callback - Callback function
     */
    on(event, callback) {
        if (this.listeners[event]) {
            this.listeners[event].push(callback);
        }
        return this;
    }

    /**
     * Remove event listener
     * @param {string} event - Event name
     * @param {Function} callback - Callback to remove
     */
    off(event, callback) {
        if (this.listeners[event]) {
            this.listeners[event] = this.listeners[event].filter(fn => fn !== callback);
        }
        return this;
    }

    /**
     * Insert random vectors into the index
     * @param {number} count - Number of vectors to insert
     * @returns {object} Insert result with ids and timing
     */
    insert(count) {
        this._ensureInitialized();

        const ids = [];
        const start = performance.now();

        for (let i = 0; i < count; i++) {
            const vector = new Float32Array(this.dimension);
            for (let j = 0; j < this.dimension; j++) {
                vector[j] = Math.random() * 2 - 1; // Range [-1, 1]
            }
            const id = this.index.insert(vector);
            ids.push(id);
            this.insertedIds.push(id);
        }

        const elapsed = performance.now() - start;
        const result = {
            count,
            ids,
            elapsed,
            perVector: elapsed / count,
            throughput: count / (elapsed / 1000)
        };

        this._emit('insert', result);
        return result;
    }

    /**
     * Insert a specific vector
     * @param {Float32Array} vector - The vector to insert
     * @returns {number} The assigned vector ID
     */
    insertVector(vector) {
        this._ensureInitialized();

        if (vector.length !== this.dimension) {
            throw new Error(`Dimension mismatch: expected ${this.dimension}, got ${vector.length}`);
        }

        const id = this.index.insert(vector);
        this.insertedIds.push(id);
        this._emit('insert', { count: 1, ids: [id] });
        return id;
    }

    /**
     * Soft delete a specific vector by ID
     * @param {number} id - The vector ID to delete
     * @returns {boolean} True if deleted, false if already deleted
     */
    delete(id) {
        this._ensureInitialized();

        const result = this.index.softDelete(id);
        if (result) {
            this._emit('delete', { ids: [id], count: 1 });
        }
        return result;
    }

    /**
     * Delete a random percentage of live vectors
     * @param {number} ratio - Ratio to delete (0.0 to 1.0)
     * @returns {object} Delete result with count and timing
     */
    deleteRandom(ratio) {
        this._ensureInitialized();

        if (ratio < 0 || ratio > 1) {
            throw new Error('Ratio must be between 0.0 and 1.0');
        }

        const liveIds = this.insertedIds.filter(id => {
            try {
                return !this.index.isDeleted(id);
            } catch (e) {
                // CRIT-3 FIX: Log errors instead of silently swallowing
                console.warn(`[SoftDeleteDemo] Could not check delete status for vector ${id}:`, e.message || e);
                return false;
            }
        });

        const toDelete = Math.floor(liveIds.length * ratio);
        if (toDelete === 0) {
            return { count: 0, ids: [], elapsed: 0 };
        }

        // Fisher-Yates shuffle for random selection
        const shuffled = [...liveIds];
        for (let i = shuffled.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
        }

        const targets = shuffled.slice(0, toDelete);
        const deleted = [];
        const start = performance.now();

        for (const id of targets) {
            try {
                if (this.index.softDelete(id)) {
                    deleted.push(id);
                }
            } catch (e) {
                this._emit('error', { operation: 'delete', id, error: e });
            }
        }

        const elapsed = performance.now() - start;
        const result = {
            count: deleted.length,
            ids: deleted,
            elapsed,
            ratio: deleted.length / liveIds.length
        };

        this._emit('delete', result);
        return result;
    }

    /**
     * Check if a vector is deleted
     * @param {number} id - The vector ID
     * @returns {boolean} True if deleted
     */
    isDeleted(id) {
        this._ensureInitialized();
        return this.index.isDeleted(id);
    }

    /**
     * Search for k nearest neighbors
     * @param {Float32Array|null} query - Query vector (null for random)
     * @param {number} k - Number of neighbors (default: 10)
     * @returns {object} Search results with timing
     */
    search(query = null, k = 10) {
        this._ensureInitialized();

        if (!query) {
            query = new Float32Array(this.dimension);
            for (let i = 0; i < this.dimension; i++) {
                query[i] = Math.random() * 2 - 1;
            }
        }

        if (query.length !== this.dimension) {
            throw new Error(`Query dimension mismatch: expected ${this.dimension}, got ${query.length}`);
        }

        const start = performance.now();
        const results = this.index.search(query, k);
        const elapsed = performance.now() - start;

        const searchResult = {
            results: Array.from(results).map(r => ({
                id: r.id,
                distance: r.score
            })),
            elapsed,
            k,
            returned: results.length
        };

        this._emit('search', searchResult);
        return searchResult;
    }

    /**
     * Check if compaction is recommended
     * @returns {boolean} True if compaction is recommended
     */
    needsCompaction() {
        this._ensureInitialized();
        return this.index.needsCompaction();
    }

    /**
     * Get compaction warning message if applicable
     * @returns {string|null} Warning message or null
     */
    getCompactionWarning() {
        this._ensureInitialized();
        return this.index.compactionWarning();
    }

    /**
     * Run compaction to reclaim space from tombstones
     * @returns {object} Compaction result
     */
    compact() {
        this._ensureInitialized();

        const start = performance.now();
        const result = this.index.compact();
        const elapsed = performance.now() - start;

        // Update insertedIds to remove compacted entries
        this.insertedIds = this.insertedIds.filter(id => {
            try {
                return !this.index.isDeleted(id);
            } catch (e) {
                // CRIT-3 FIX: Log - ID likely no longer exists after compaction
                console.debug(`[SoftDeleteDemo] Vector ${id} no longer exists (expected after compaction)`);
                return false;
            }
        });

        const compactResult = {
            tombstonesRemoved: result.tombstones_removed,
            newSize: result.new_size,
            durationMs: result.duration_ms,
            totalElapsed: elapsed
        };

        this._emit('compact', compactResult);
        return compactResult;
    }

    /**
     * Get current index statistics
     * @returns {object} Statistics object
     */
    getStats() {
        this._ensureInitialized();

        const live = this.index.liveCount();
        const deleted = this.index.deletedCount();
        const total = live + deleted;

        return {
            total,
            live,
            deleted,
            tombstoneRatio: this.index.tombstoneRatio(),
            needsCompaction: this.index.needsCompaction(),
            compactionWarning: this.index.compactionWarning(),
            dimension: this.dimension,
            memoryEstimateKB: Math.round(total * this.dimension * 4 / 1024)
        };
    }

    /**
     * Get the compaction threshold
     * @returns {number} Current threshold (0.0 to 1.0)
     */
    getCompactionThreshold() {
        this._ensureInitialized();
        return this.index.compactionThreshold();
    }

    /**
     * Set the compaction threshold
     * @param {number} threshold - New threshold (0.01 to 0.99)
     */
    setCompactionThreshold(threshold) {
        this._ensureInitialized();
        this.index.setCompactionThreshold(threshold);
    }

    /**
     * Reset the index to empty state
     */
    reset() {
        this._ensureInitialized();

        this.insertedIds = [];
        const config = new wasmModule.EdgeVecConfig(this.dimension);
        this.index = new wasmModule.EdgeVec(config);

        this._emit('insert', { type: 'reset' });
    }

    /**
     * Free WASM resources
     */
    dispose() {
        if (this.index) {
            this.index.free();
            this.index = null;
        }
        this.initialized = false;
        this.insertedIds = [];
    }

    /**
     * Run a full benchmark cycle
     * @param {number} vectorCount - Number of vectors to test
     * @param {number} deleteRatio - Ratio to delete
     * @returns {object} Benchmark results
     */
    async benchmark(vectorCount = 1000, deleteRatio = 0.3) {
        this._ensureInitialized();

        const results = {
            vectorCount,
            deleteRatio,
            dimension: this.dimension,
            phases: {}
        };

        // Phase 1: Insert
        const insertResult = this.insert(vectorCount);
        results.phases.insert = {
            elapsed: insertResult.elapsed,
            throughput: insertResult.throughput
        };

        // Phase 2: Search (before delete)
        const searchBefore = this.search(null, 10);
        results.phases.searchBeforeDelete = {
            elapsed: searchBefore.elapsed,
            resultsCount: searchBefore.returned
        };

        // Phase 3: Delete
        const deleteResult = this.deleteRandom(deleteRatio);
        results.phases.delete = {
            deleted: deleteResult.count,
            elapsed: deleteResult.elapsed
        };

        // Phase 4: Search (after delete)
        const searchAfter = this.search(null, 10);
        results.phases.searchAfterDelete = {
            elapsed: searchAfter.elapsed,
            resultsCount: searchAfter.returned
        };

        // Phase 5: Compaction
        if (this.needsCompaction()) {
            const compactResult = this.compact();
            results.phases.compact = {
                tombstonesRemoved: compactResult.tombstonesRemoved,
                newSize: compactResult.newSize,
                elapsed: compactResult.durationMs
            };
        }

        // Phase 6: Search (after compaction)
        const searchFinal = this.search(null, 10);
        results.phases.searchAfterCompact = {
            elapsed: searchFinal.elapsed,
            resultsCount: searchFinal.returned
        };

        results.finalStats = this.getStats();

        return results;
    }
}

// Export for browser usage
if (typeof window !== 'undefined') {
    window.SoftDeleteDemo = SoftDeleteDemo;
}
