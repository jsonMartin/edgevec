/**
 * HNSWLib-Node Adapter for Competitive Benchmarks
 *
 * Task: W14.3
 * Purpose: Benchmark adapter for hnswlib-node library
 */

class HnswlibAdapter {
    constructor() {
        this.name = 'hnswlib-node';
        this.index = null;
        this.HNSWLib = null;
    }

    async initialize(config) {
        try {
            // Dynamic require to handle optional dependency
            this.HNSWLib = require('hnswlib-node');

            this.index = new this.HNSWLib.HierarchicalNSW('l2', config.dimensions);

            // Initialize index with parameters
            // initIndex(maxElements, M, efConstruction, randomSeed)
            const maxElements = config.vectorCount || 100000;
            const M = config.hnsw?.m || 16;
            const efConstruction = config.hnsw?.efConstruction || 200;

            this.index.initIndex(maxElements, M, efConstruction, 100);

            // Set ef search parameter
            if (config.hnsw?.efSearch) {
                this.index.setEf(config.hnsw.efSearch);
            }

            console.log(`[${this.name}] Initialized with dimensions=${config.dimensions}, maxElements=${maxElements}`);
        } catch (error) {
            console.warn(`[${this.name}] Not available: ${error.message}`);
            console.warn(`[${this.name}] Install with: npm install hnswlib-node`);
            this.index = null;
        }
    }

    async insert(vectors) {
        if (!this.index) {
            return vectors.map((_, i) => i);
        }

        const ids = [];
        for (let i = 0; i < vectors.length; i++) {
            const vec = vectors[i] instanceof Float32Array
                ? Array.from(vectors[i])
                : vectors[i];

            this.index.addPoint(vec, i);
            ids.push(i);
        }
        return ids;
    }

    async search(query, k) {
        if (!this.index) {
            return [];
        }

        const vec = query instanceof Float32Array
            ? Array.from(query)
            : query;

        try {
            const result = this.index.searchKnn(vec, k);
            return result.neighbors.map((id, idx) => ({
                id,
                distance: result.distances[idx]
            }));
        } catch (error) {
            console.error(`[${this.name}] Search error:`, error.message);
            return [];
        }
    }

    async getMemoryUsage() {
        return process.memoryUsage().heapUsed;
    }

    async cleanup() {
        this.index = null;
    }
}

module.exports = { HnswlibAdapter };
