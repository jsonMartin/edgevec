// wasm/types.ts

/**
 * Vector identifier returned from insert operations.
 */
export type VectorId = number;

/**
 * Distance metrics supported by EdgeVec.
 */
export type DistanceMetric = 'l2' | 'cosine' | 'dot';

/**
 * Quantization modes for memory optimization.
 */
export type QuantizationMode = 'none' | 'sq8';

/**
 * Search result containing vector ID and distance.
 */
export interface SearchResult {
  /** Unique identifier of the matched vector */
  id: VectorId;
  /** Distance from query (lower is more similar for L2/cosine) */
  distance: number;
}

/**
 * Configuration options for EdgeVecClient.
 */
export interface EdgeVecClientConfig {
  /** Vector dimensions (must match all inserted vectors) */
  dimensions: number;
  /** Distance metric for similarity calculation */
  metric?: DistanceMetric;
  /** Quantization mode for memory optimization */
  quantization?: QuantizationMode;
}

/**
 * Statistics about the EdgeVec instance.
 */
export interface EdgeVecStats {
  /** Number of vectors in the index */
  vectorCount: number;
  /** Configured dimensions */
  dimensions: number;
  /** Memory usage in bytes (approximate) */
  memoryBytes: number;
}

/**
 * Result of a compaction operation (v0.3.0).
 *
 * Returned by `compact()` to provide metrics about the operation.
 */
export interface CompactionResult {
  /** Number of tombstones (deleted vectors) removed during compaction */
  tombstonesRemoved: number;
  /** New index size after compaction (live vectors only) */
  newSize: number;
  /** Time taken for the compaction operation in milliseconds */
  durationMs: number;
}

/**
 * Soft delete statistics for the index.
 */
export interface SoftDeleteStats {
  /** Number of vectors that have been soft-deleted */
  deletedCount: number;
  /** Number of vectors that are currently searchable */
  liveCount: number;
  /** Ratio of deleted to total vectors (0.0 to 1.0) */
  tombstoneRatio: number;
  /** Whether compaction is recommended (ratio > threshold) */
  needsCompaction: boolean;
}
