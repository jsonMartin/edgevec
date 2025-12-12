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
