/* tslint:disable */
/* eslint-disable */

export class BatchInsertConfig {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Creates a new `BatchInsertConfig` with default settings.
   *
   * Default: `validate_dimensions = true`
   */
  constructor();
  /**
   * Returns whether dimension validation is enabled.
   */
  validateDimensions: boolean;
}

export class BatchInsertResult {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Returns a copy of the IDs of successfully inserted vectors.
   */
  readonly ids: BigUint64Array;
  /**
   * Returns the total number of vectors attempted (input array length).
   */
  readonly total: number;
  /**
   * Returns the number of vectors successfully inserted.
   */
  readonly inserted: number;
}

export class EdgeVec {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Loads the database from IndexedDB.
   *
   * # Arguments
   *
   * * `name` - The name of the database file in IndexedDB.
   *
   * # Returns
   *
   * A Promise that resolves to the loaded EdgeVec instance.
   *
   * # Errors
   *
   * Returns an error if loading fails, deserialization fails, or data is corrupted.
   */
  static load(name: string): Promise<EdgeVec>;
  /**
   * Saves the database to IndexedDB.
   *
   * # Arguments
   *
   * * `name` - The name of the database file in IndexedDB.
   *
   * # Returns
   *
   * A Promise that resolves when saving is complete.
   *
   * # Errors
   *
   * Returns an error if serialization fails or if the backend write fails.
   */
  save(name: string): Promise<void>;
  /**
   * Check if a vector is deleted (tombstoned).
   *
   * # Arguments
   *
   * * `vector_id` - The ID of the vector to check.
   *
   * # Returns
   *
   * * `true` if the vector is deleted
   * * `false` if the vector is live
   *
   * # Errors
   *
   * Returns an error if the vector ID doesn't exist.
   */
  isDeleted(vector_id: number): boolean;
  /**
   * Get the count of live (non-deleted) vectors.
   *
   * # Returns
   *
   * The number of vectors that are currently searchable.
   */
  liveCount(): number;
  /**
   * Creates an iterator to save the database in chunks.
   *
   * # Arguments
   *
   * * `chunk_size` - Maximum size of each chunk in bytes (default: 10MB).
   *
   * # Returns
   *
   * A `PersistenceIterator` that yields `Uint8Array` chunks.
   *
   * # Safety
   *
   * The returned iterator holds a reference to this `EdgeVec` instance.
   * You MUST ensure `EdgeVec` is not garbage collected or freed while using the iterator.
   */
  save_stream(chunk_size?: number | null): PersistenceIterator;
  /**
   * Soft delete a vector by marking it as a tombstone.
   *
   * The vector remains in the index but is excluded from search results.
   * Space is reclaimed via `compact()` when tombstone ratio exceeds threshold.
   *
   * # Arguments
   *
   * * `vector_id` - The ID of the vector to delete (returned from `insert`).
   *
   * # Returns
   *
   * * `true` if the vector was deleted
   * * `false` if the vector was already deleted (idempotent)
   *
   * # Errors
   *
   * Returns an error if the vector ID doesn't exist.
   *
   * # Example (JavaScript)
   *
   * ```javascript
   * const id = index.insert(new Float32Array(128).fill(1.0));
   * const wasDeleted = index.softDelete(id);
   * console.log(`Deleted: ${wasDeleted}`); // true
   * console.log(`Is deleted: ${index.isDeleted(id)}`); // true
   * ```
   */
  softDelete(vector_id: number): boolean;
  /**
   * Get the count of deleted (tombstoned) vectors.
   *
   * # Returns
   *
   * The number of vectors that have been soft-deleted but not yet compacted.
   */
  deletedCount(): number;
  /**
   * Inserts multiple vectors using the new batch API (W12.3).
   *
   * This method follows the API design from `WASM_BATCH_API.md`:
   * - Input: Array of Float32Array (each array is one vector)
   * - Output: BatchInsertResult with inserted count, total, and IDs
   * - Error codes: EMPTY_BATCH, DIMENSION_MISMATCH, DUPLICATE_ID, etc.
   *
   * # Arguments
   *
   * * `vectors` - JS Array of Float32Array vectors to insert (1 to 100,000)
   * * `config` - Optional BatchInsertConfig (default: validateDimensions = true)
   *
   * # Returns
   *
   * `BatchInsertResult` containing:
   * - `inserted`: Number of vectors successfully inserted
   * - `total`: Total vectors attempted (input array length)
   * - `ids`: Array of IDs for inserted vectors
   *
   * # Performance Note
   *
   * Batch insert optimizes **JavaScript↔WASM boundary overhead**, not HNSW graph
   * construction. At smaller batch sizes (100-1K vectors), expect 1.2-1.5x speedup
   * vs sequential insertion due to reduced FFI calls. At larger scales (5K+), both
   * methods converge as HNSW graph construction becomes the dominant cost.
   *
   * The batch API still provides value at all scales through:
   * - Simpler API (single call vs loop)
   * - Atomic operation semantics
   * - Progress callback support (via `insertBatchWithProgress`)
   *
   * # Errors
   *
   * Returns a JS error object with `code` property:
   * - `EMPTY_BATCH`: Input array is empty
   * - `DIMENSION_MISMATCH`: Vector dimensions don't match index
   * - `DUPLICATE_ID`: Vector ID already exists
   * - `INVALID_VECTOR`: Vector contains NaN or Infinity
   * - `CAPACITY_EXCEEDED`: Batch exceeds max capacity
   * - `INTERNAL_ERROR`: Internal HNSW error
   */
  insertBatch(vectors: Array<any>, config?: BatchInsertConfig | null): BatchInsertResult;
  /**
   * Get the ratio of deleted to total vectors.
   *
   * # Returns
   *
   * A value between 0.0 and 1.0 representing the tombstone ratio.
   * 0.0 means no deletions, 1.0 means all vectors deleted.
   */
  tombstoneRatio(): number;
  /**
   * Check if compaction is recommended.
   *
   * Returns `true` when `tombstoneRatio()` exceeds the compaction threshold
   * (default: 30%). Use `compact()` to reclaim space from deleted vectors.
   *
   * # Returns
   *
   * * `true` if compaction is recommended
   * * `false` if tombstone ratio is below threshold
   */
  needsCompaction(): boolean;
  /**
   * Inserts a batch of vectors into the index (flat array format).
   *
   * **Note:** This is the legacy API. For the new API, use `insertBatch` which
   * accepts an Array of Float32Array.
   *
   * # Arguments
   *
   * * `vectors` - Flat Float32Array containing `count * dimensions` elements.
   * * `count` - Number of vectors in the batch.
   *
   * # Returns
   *
   * A Uint32Array containing the assigned Vector IDs.
   *
   * # Errors
   *
   * Returns error if dimensions mismatch, vector contains NaNs, or ID overflows.
   */
  insertBatchFlat(vectors: Float32Array, count: number): Uint32Array;
  /**
   * Get a warning message if compaction is recommended.
   *
   * # Returns
   *
   * * A warning string if `needsCompaction()` is true
   * * `null` if compaction is not needed
   *
   * # Example (JavaScript)
   *
   * ```javascript
   * const warning = index.compactionWarning();
   * if (warning) {
   *     console.warn(warning);
   *     index.compact();
   * }
   * ```
   */
  compactionWarning(): string | undefined;
  /**
   * Get the current compaction threshold.
   *
   * # Returns
   *
   * The threshold ratio (0.0 to 1.0) above which `needsCompaction()` returns true.
   * Default is 0.3 (30%).
   */
  compactionThreshold(): number;
  /**
   * Set the compaction threshold.
   *
   * # Arguments
   *
   * * `ratio` - The new threshold (clamped to 0.01 - 0.99).
   */
  setCompactionThreshold(ratio: number): void;
  /**
   * Batch insert with progress callback (W14.1).
   *
   * Inserts multiple vectors while reporting progress to a JavaScript callback.
   * The callback is invoked at the **start (0%)** and **end (100%)** of the batch
   * insertion. Intermediate progress during insertion is not currently reported.
   *
   * # Arguments
   *
   * * `vectors` - JS Array of Float32Array vectors to insert
   * * `on_progress` - JS function called with (inserted: number, total: number)
   *
   * # Returns
   *
   * `BatchInsertResult` containing inserted count, total, and IDs.
   *
   * # Performance Note
   *
   * See [`Self::insert_batch_v2`] for performance characteristics. Batch insert optimizes
   * JS↔WASM boundary overhead (1.2-1.5x at small scales), but converges with
   * sequential insertion at larger scales as HNSW graph construction dominates.
   *
   * # Callback Behavior
   *
   * - The callback is called exactly **twice**: once with `(0, total)` before
   *   insertion begins, and once with `(total, total)` after completion.
   * - **Errors in the callback are intentionally ignored** — the batch insert
   *   will succeed even if the progress callback throws an exception. This
   *   ensures that UI errors don't break data operations.
   *
   * # Example (JavaScript)
   *
   * ```javascript
   * const result = index.insertBatchWithProgress(vectors, (done, total) => {
   *     console.log(`Progress: ${Math.round(done/total*100)}%`);
   * });
   * console.log(`Inserted ${result.inserted} vectors`);
   * ```
   *
   * # Errors
   *
   * Returns a JS error object with `code` property on failure.
   * Note: Callback exceptions do NOT cause this function to return an error.
   */
  insertBatchWithProgress(vectors: Array<any>, on_progress: Function): BatchInsertResult;
  /**
   * Creates a new EdgeVec database.
   *
   * # Errors
   *
   * Returns an error if the configuration is invalid (e.g., unknown metric).
   */
  constructor(config: EdgeVecConfig);
  /**
   * Inserts a vector into the index.
   *
   * # Arguments
   *
   * * `vector` - A Float32Array containing the vector data.
   *
   * # Returns
   *
   * The assigned Vector ID (u32).
   *
   * # Errors
   *
   * Returns error if dimensions mismatch, vector contains NaNs, or ID overflows.
   */
  insert(vector: Float32Array): number;
  /**
   * Searches for nearest neighbors.
   *
   * # Arguments
   *
   * * `query` - The query vector.
   * * `k` - The number of neighbors to return.
   *
   * # Returns
   *
   * An array of objects: `[{ id: u32, score: f32 }, ...]`.
   *
   * # Errors
   *
   * Returns error if dimensions mismatch or vector contains NaNs.
   */
  search(query: Float32Array, k: number): any;
  /**
   * Compact the index by rebuilding without tombstones.
   *
   * This operation:
   * 1. Creates a new index with only live vectors
   * 2. Re-inserts vectors preserving IDs
   * 3. Replaces the current index
   *
   * **WARNING:** This is a blocking operation. For indices with >10k vectors,
   * consider running during idle time or warning the user about potential delays.
   *
   * # Returns
   *
   * A `CompactionResult` object containing:
   * * `tombstonesRemoved` - Number of deleted vectors removed
   * * `newSize` - Size of the index after compaction
   * * `durationMs` - Time taken in milliseconds
   *
   * # Errors
   *
   * Returns an error if compaction fails (e.g., memory allocation error).
   *
   * # Example (JavaScript)
   *
   * ```javascript
   * if (index.needsCompaction()) {
   *     const result = index.compact();
   *     console.log(`Removed ${result.tombstonesRemoved} tombstones`);
   *     console.log(`New size: ${result.newSize}`);
   *     console.log(`Took ${result.durationMs}ms`);
   * }
   * ```
   */
  compact(): WasmCompactionResult;
}

export class EdgeVecConfig {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create a new configuration with required dimensions.
   */
  constructor(dimensions: number);
  /**
   * Vector dimensionality.
   */
  dimensions: number;
  /**
   * Set distance metric ("l2", "cosine", "dot").
   */
  set metric(value: string);
  /**
   * Set ef_search parameter.
   */
  set ef_search(value: number);
  /**
   * Set ef_construction parameter.
   */
  set ef_construction(value: number);
  /**
   * Set M parameter (max connections per node in layers > 0).
   */
  set m(value: number);
  /**
   * Set M0 parameter (max connections per node in layer 0).
   */
  set m0(value: number);
}

export class PersistenceIterator {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Returns the next chunk of data.
   *
   * # Returns
   *
   * * `Some(Uint8Array)` - The next chunk of data.
   * * `None` - If iteration is complete.
   *
   * # Panics
   *
   * Panics if the parent `EdgeVec` instance has been freed.
   */
  next_chunk(): Uint8Array | undefined;
}

export class WasmCompactionResult {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Number of tombstones (deleted vectors) removed during compaction.
   */
  readonly tombstones_removed: number;
  /**
   * New index size after compaction (live vectors only).
   */
  readonly new_size: number;
  /**
   * Time taken for the compaction operation in milliseconds.
   */
  readonly duration_ms: number;
}

/**
 * Initialize logging hooks.
 */
export function init_logging(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_batchinsertconfig_free: (a: number, b: number) => void;
  readonly __wbg_batchinsertresult_free: (a: number, b: number) => void;
  readonly __wbg_edgevec_free: (a: number, b: number) => void;
  readonly __wbg_edgevecconfig_free: (a: number, b: number) => void;
  readonly __wbg_get_edgevecconfig_dimensions: (a: number) => number;
  readonly __wbg_get_wasmcompactionresult_duration_ms: (a: number) => number;
  readonly __wbg_get_wasmcompactionresult_new_size: (a: number) => number;
  readonly __wbg_get_wasmcompactionresult_tombstones_removed: (a: number) => number;
  readonly __wbg_persistenceiterator_free: (a: number, b: number) => void;
  readonly __wbg_set_edgevecconfig_dimensions: (a: number, b: number) => void;
  readonly __wbg_wasmcompactionresult_free: (a: number, b: number) => void;
  readonly batchinsertconfig_new: () => number;
  readonly batchinsertconfig_set_validateDimensions: (a: number, b: number) => void;
  readonly batchinsertconfig_validateDimensions: (a: number) => number;
  readonly batchinsertresult_ids: (a: number, b: number) => void;
  readonly batchinsertresult_inserted: (a: number) => number;
  readonly batchinsertresult_total: (a: number) => number;
  readonly edgevec_compact: (a: number, b: number) => void;
  readonly edgevec_compactionThreshold: (a: number) => number;
  readonly edgevec_compactionWarning: (a: number, b: number) => void;
  readonly edgevec_deletedCount: (a: number) => number;
  readonly edgevec_insert: (a: number, b: number, c: number) => void;
  readonly edgevec_insertBatch: (a: number, b: number, c: number, d: number) => void;
  readonly edgevec_insertBatchFlat: (a: number, b: number, c: number, d: number) => void;
  readonly edgevec_insertBatchWithProgress: (a: number, b: number, c: number, d: number) => void;
  readonly edgevec_isDeleted: (a: number, b: number, c: number) => void;
  readonly edgevec_liveCount: (a: number) => number;
  readonly edgevec_load: (a: number, b: number) => number;
  readonly edgevec_needsCompaction: (a: number) => number;
  readonly edgevec_new: (a: number, b: number) => void;
  readonly edgevec_save: (a: number, b: number, c: number) => number;
  readonly edgevec_save_stream: (a: number, b: number) => number;
  readonly edgevec_search: (a: number, b: number, c: number, d: number) => void;
  readonly edgevec_setCompactionThreshold: (a: number, b: number) => void;
  readonly edgevec_softDelete: (a: number, b: number, c: number) => void;
  readonly edgevec_tombstoneRatio: (a: number) => number;
  readonly edgevecconfig_new: (a: number) => number;
  readonly edgevecconfig_set_ef_construction: (a: number, b: number) => void;
  readonly edgevecconfig_set_ef_search: (a: number, b: number) => void;
  readonly edgevecconfig_set_m: (a: number, b: number) => void;
  readonly edgevecconfig_set_m0: (a: number, b: number) => void;
  readonly edgevecconfig_set_metric: (a: number, b: number, c: number) => void;
  readonly persistenceiterator_next_chunk: (a: number) => number;
  readonly init_logging: () => void;
  readonly __wasm_bindgen_func_elem_301: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_294: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_496: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_export: (a: number) => void;
  readonly __wbindgen_export2: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export3: (a: number, b: number) => number;
  readonly __wbindgen_export4: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
