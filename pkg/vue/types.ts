/**
 * EdgeVec Vue Composables - Type Definitions
 *
 * @module edgevec/vue
 * @version 0.8.0
 */

import type { Ref, ComputedRef } from 'vue';
import type { EdgeVecIndex, SearchResult } from '../edgevec-wrapper.js';
import type { FilterExpression } from '../filter.js';

// =============================================================================
// Vue Utility Types
// =============================================================================

/**
 * A value that may be a Vue ref or a raw value.
 * Used for flexible composable inputs.
 */
export type MaybeRef<T> = T | Ref<T>;

/**
 * A value that may be a Vue ref, computed ref, or raw value.
 */
export type MaybeRefOrGetter<T> = T | Ref<T> | ComputedRef<T> | (() => T);

// =============================================================================
// useEdgeVec Types
// =============================================================================

/**
 * Options for useEdgeVec composable.
 */
export interface UseEdgeVecOptions {
  /** Vector dimensions (required) */
  dimensions: number;

  /** IndexedDB store name for persistence (optional) */
  persistName?: string;

  /** HNSW ef_construction parameter (default: 200) */
  efConstruction?: number;

  /** HNSW M parameter (default: 16) */
  m?: number;
}

/**
 * Result of useEdgeVec composable.
 * All reactive values are returned as Vue refs.
 */
export interface UseEdgeVecResult {
  /** The EdgeVec index instance (null until ready) */
  db: Ref<EdgeVecIndex | null>;

  /** True when WASM is loaded and index is ready */
  isReady: Ref<boolean>;

  /** True during WASM initialization */
  isLoading: Ref<boolean>;

  /** Error if initialization failed */
  error: Ref<Error | null>;

  /** Index statistics (null until ready) */
  stats: Ref<{
    count: number;
    dimensions: number;
  } | null>;

  /** Force reload the index */
  reload: () => Promise<void>;

  /** Save index to IndexedDB */
  save: (name?: string) => Promise<void>;
}

// =============================================================================
// useSearch Types
// =============================================================================

/**
 * Options for useSearch composable.
 * Supports both raw values and Vue refs for reactive inputs.
 */
export interface UseSearchOptions {
  /** Query vector (null disables search). Can be ref or raw value. */
  vector: MaybeRef<Float32Array | number[] | null>;

  /** Number of results (default: 10) */
  k?: MaybeRef<number>;

  /** Filter expression or string. Can be ref or raw value. */
  filter?: MaybeRef<FilterExpression | string | undefined>;

  /** Enable/disable search (default: true). Can be ref or computed. */
  enabled?: MaybeRefOrGetter<boolean>;

  /** Debounce delay in ms (default: 0) */
  debounceMs?: number;

  /** Include vector data in results (default: false) */
  includeVectors?: boolean;

  /** Include metadata in results (default: false) */
  includeMetadata?: boolean;
}

/**
 * Result of useSearch composable.
 * All reactive values are returned as Vue refs.
 */
export interface UseSearchResult {
  /** Search results (empty array until search completes) */
  results: Ref<SearchResult[]>;

  /** True during search execution */
  isSearching: Ref<boolean>;

  /** Error if search failed */
  error: Ref<Error | null>;

  /** Search execution time in ms (null until search completes) */
  searchTime: Ref<number | null>;

  /** Manually trigger search */
  refetch: () => Promise<void>;
}
