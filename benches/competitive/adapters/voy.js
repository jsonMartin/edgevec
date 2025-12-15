/**
 * Voy Adapter for Competitive Benchmarks
 *
 * Task: W14.3
 * Purpose: Benchmark adapter for voy-search library
 *
 * Note: Voy uses a class-based API with the Voy class that takes embeddings at construction.
 */

const path = require('path');
const { pathToFileURL } = require('url');

class VoyAdapter {
    constructor() {
        this.name = 'voy';
        this.voyModule = null;
        this.index = null;
        this.dimensions = 128;
    }

    async initialize(config) {
        try {
            // voy-search is ESM-only, need to import directly from file
            const voyPath = path.join(__dirname, '..', 'node_modules', 'voy-search', 'voy_search.js');
            const voyUrl = pathToFileURL(voyPath).href;
            this.voyModule = await import(voyUrl);

            this.dimensions = config.dimensions;
            this.index = null;

            console.log(`[${this.name}] Initialized with dimensions=${config.dimensions}`);
        } catch (error) {
            console.warn(`[${this.name}] Not available: ${error.message}`);
            console.warn(`[${this.name}] Install with: npm install voy-search`);
            this.voyModule = null;
        }
    }

    async insert(vectors) {
        if (!this.voyModule) {
            return vectors.map((_, i) => i);
        }

        // Voy uses the Voy class with embeddings array
        // Format: { embeddings: [{ id: string, title: string, url: string, embeddings: number[] }] }
        const embeddings = vectors.map((vec, i) => ({
            id: String(i),
            title: `vec_${i}`,
            url: `#${i}`,
            embeddings: vec instanceof Float32Array ? Array.from(vec) : vec,
        }));

        try {
            // Create index using Voy class
            this.index = new this.voyModule.Voy({ embeddings });
            return vectors.map((_, i) => i);
        } catch (error) {
            console.error(`[${this.name}] Insert error:`, error.message);
            return [];
        }
    }

    async search(query, k) {
        if (!this.voyModule || !this.index) {
            return [];
        }

        const vec = query instanceof Float32Array
            ? Array.from(query)
            : query;

        try {
            // Voy search takes the query vector and k
            const results = this.index.search(vec, k);

            return results.neighbors.map(n => ({
                id: parseInt(n.id),
                distance: 0  // Voy doesn't return distance in this version
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

module.exports = { VoyAdapter };
