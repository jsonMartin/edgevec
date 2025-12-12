// wasm/EdgeVecClient.ts

import init, { EdgeVec as WasmEdgeVec, EdgeVecConfig as WasmConfig } from '../pkg/edgevec.js';

export interface SearchResult {
  id: number;
  distance: number;
}

export interface EdgeVecClientConfig {
  dimensions: number;
  metric?: 'l2' | 'cosine' | 'dot';
  quantization?: 'none' | 'sq8';
}

/**
 * High-level TypeScript wrapper for EdgeVec WASM bindings.
 *
 * Provides auto-initialization, Promise-based API, and TypeScript ergonomics.
 *
 * @example
 * ```typescript
 * const client = await EdgeVecClient.create({ dimensions: 128 });
 *
 * const vector = new Float32Array(128).fill(0.1);
 * const id = client.insert(vector); // Synchronous
 *
 * const results = client.search(vector, 10); // Synchronous
 * console.log(results);
 *
 * await client.save('my-db');
 * ```
 */
export class EdgeVecClient {
  private static initPromise: Promise<void> | null = null;
  private inner: WasmEdgeVec;
  private config: EdgeVecClientConfig;
  private vectorCount: number;

  private constructor(inner: WasmEdgeVec, config: EdgeVecClientConfig, vectorCount: number = 0) {
    this.inner = inner;
    this.config = config;
    this.vectorCount = vectorCount;
  }

  /**
   * Create a new EdgeVec instance with auto WASM initialization.
   *
   * @param config - Configuration options
   * @returns Promise resolving to initialized EdgeVecClient
   * @throws Error if dimensions <= 0 or metric is invalid
   */
  static async create(config: EdgeVecClientConfig): Promise<EdgeVecClient> {
    // C4 Fix: Input validation before WASM calls
    if (!config.dimensions || config.dimensions <= 0) {
      throw new Error(`Dimensions must be positive, got ${config.dimensions}`);
    }
    if (!Number.isInteger(config.dimensions)) {
      throw new Error(`Dimensions must be an integer, got ${config.dimensions}`);
    }
    if (config.metric && !['l2', 'cosine', 'dot'].includes(config.metric)) {
      throw new Error(`Invalid metric: ${config.metric}. Must be 'l2', 'cosine', or 'dot'`);
    }
    // M5 Fix: Explicitly reject quantization parameter
    if (config.quantization) {
      throw new Error('Quantization not supported in current WASM API. Remove the quantization parameter from config.');
    }

    await EdgeVecClient.ensureInitialized();

    const wasmConfig = new WasmConfig(config.dimensions);
    // Apply optional config
    if (config.metric) {
      wasmConfig.metric = config.metric;
    }

    const inner = new WasmEdgeVec(wasmConfig);
    return new EdgeVecClient(inner, config);
  }

  /**
   * Load an existing database from IndexedDB.
   *
   * @param name - Database name
   * @param config - Configuration (must match saved database)
   * @returns Promise resolving to loaded EdgeVecClient
   * @throws Error if name is empty or config is invalid
   *
   * **Important:** The `length` property will be 0 after load until vectors are inserted.
   * This is a known limitation - WASM API doesn't expose vector count from loaded databases.
   */
  static async load(name: string, config: EdgeVecClientConfig): Promise<EdgeVecClient> {
    // M1 Fix: Validate inputs
    if (!name || name.trim().length === 0) {
      throw new Error('Database name cannot be empty');
    }
    if (!config.dimensions || config.dimensions <= 0) {
      throw new Error(`Dimensions must be positive, got ${config.dimensions}`);
    }
    // M10 Fix: Add integer validation (consistent with create())
    if (!Number.isInteger(config.dimensions)) {
      throw new Error(`Dimensions must be an integer, got ${config.dimensions}`);
    }
    if (config.metric && !['l2', 'cosine', 'dot'].includes(config.metric)) {
      throw new Error(`Invalid metric: ${config.metric}. Must be 'l2', 'cosine', or 'dot'`);
    }

    await EdgeVecClient.ensureInitialized();

    const inner = await WasmEdgeVec.load(name);
    // C1: Vector count not restored - WASM API limitation
    // Count remains 0 after load, only tracks inserts made after load
    return new EdgeVecClient(inner, config, 0);
  }

  private static async ensureInitialized(): Promise<void> {
    if (!EdgeVecClient.initPromise) {
      EdgeVecClient.initPromise = init().then(() => undefined);
    }
    await EdgeVecClient.initPromise;
  }

  /**
   * Insert a vector into the index.
   *
   * @param vector - Float32Array of dimension matching config
   * @returns Assigned vector ID
   * @throws Error if vector dimension doesn't match config
   */
  insert(vector: Float32Array): number {
    this.validateDimension(vector);
    const id = this.inner.insert(vector);
    this.vectorCount++;
    return id;
  }

  /**
   * Search for k nearest neighbors.
   *
   * @param query - Query vector (Float32Array)
   * @param k - Number of neighbors to return
   * @returns Search results sorted by distance
   * @throws Error if query dimension doesn't match config or k <= 0
   */
  search(query: Float32Array, k: number): SearchResult[] {
    this.validateDimension(query);
    if (k <= 0) {
      throw new Error(`k must be positive, got ${k}`);
    }
    if (!Number.isInteger(k)) {
      throw new Error(`k must be an integer, got ${k}`);
    }
    const results = this.inner.search(query, k);
    // M4 Fix: Type guard for WASM results
    return Array.from(results).map((r) => {
      if (!this.isWasmSearchResult(r)) {
        throw new Error('Invalid search result from WASM');
      }
      // Transform WASM results to typed interface
      // WASM API returns { id, score } but we normalize to { id, distance }
      return {
        id: r.id,
        distance: r.score
      };
    });
  }

  private isWasmSearchResult(value: unknown): value is { id: number; score: number } {
    return (
      typeof value === 'object' &&
      value !== null &&
      'id' in value &&
      'score' in value &&
      typeof (value as { id: unknown }).id === 'number' &&
      typeof (value as { score: unknown }).score === 'number'
    );
  }

  /**
   * Save database to IndexedDB.
   *
   * @param name - Database name for storage
   * @returns Promise resolving when save is complete
   * @throws Error if name is empty
   */
  async save(name: string): Promise<void> {
    // M3 Fix: Validate database name
    if (!name || name.trim().length === 0) {
      throw new Error('Database name cannot be empty');
    }
    await this.inner.save(name);
  }

  /**
   * Get the number of vectors inserted in this session.
   *
   * **Important Limitation:** This only tracks vectors inserted via `insert()` after
   * the client was created or loaded. For databases loaded from IndexedDB, this will
   * be 0 until new vectors are inserted.
   *
   * **Reason:** The WASM API doesn't expose the total vector count from loaded databases.
   */
  get length(): number {
    return this.vectorCount;
  }

  /**
   * Get the configured dimensions.
   */
  get dimensions(): number {
    return this.config.dimensions;
  }

  private validateDimension(vector: Float32Array): void {
    if (vector.length !== this.config.dimensions) {
      throw new Error(
        `Dimension mismatch: expected ${this.config.dimensions}, got ${vector.length}`
      );
    }
  }
}
