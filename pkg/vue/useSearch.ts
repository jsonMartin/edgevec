/**
 * useSearch - Vue 3 composable for reactive vector search
 *
 * Performs searches that automatically update when the query vector or filter changes.
 * Supports debouncing, filtering, and handles race conditions.
 *
 * @module edgevec/vue
 * @version 0.8.0
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { ref, computed } from 'vue';
 * import { useEdgeVec, useSearch } from 'edgevec/vue';
 * import { eq } from 'edgevec';
 *
 * const { db, isReady } = useEdgeVec({ dimensions: 384 });
 * const queryVector = ref<number[] | null>(null);
 *
 * const { results, isSearching, searchTime } = useSearch(db, {
 *   vector: queryVector,
 *   k: 10,
 *   filter: eq('category', 'docs'),
 *   enabled: computed(() => isReady.value && queryVector.value !== null),
 *   debounceMs: 300,
 * });
 * </script>
 *
 * <template>
 *   <ul>
 *     <li v-for="r in results" :key="r.id">
 *       Score: {{ r.score.toFixed(4) }}
 *     </li>
 *   </ul>
 * </template>
 * ```
 */

import { ref, watch, toValue, onUnmounted } from 'vue';
import type { Ref } from 'vue';
import type { EdgeVecIndex, SearchResult } from '../edgevec-wrapper.js';
import type { UseSearchOptions, UseSearchResult, MaybeRef } from './types.js';

// Stable empty array reference to prevent unnecessary reactivity triggers
const EMPTY_RESULTS: SearchResult[] = [];

/**
 * Vue 3 composable for reactive vector search.
 *
 * @param db - EdgeVec index instance (from useEdgeVec, can be ref or raw value)
 * @param options - Search options
 * @returns Composable result with reactive search results and status
 */
export function useSearch(
  db: MaybeRef<EdgeVecIndex | null>,
  options: UseSearchOptions
): UseSearchResult {
  const {
    vector,
    k = 10,
    filter,
    enabled = true,
    debounceMs = 0,
    includeVectors = false,
    includeMetadata = false,
  } = options;

  // Reactive state
  const results = ref<SearchResult[]>(EMPTY_RESULTS) as Ref<SearchResult[]>;
  const isSearching = ref(false);
  const error = ref<Error | null>(null);
  const searchTime = ref<number | null>(null);

  // Track the current search to handle race conditions
  let searchId = 0;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let isMounted = true;

  async function executeSearch(): Promise<void> {
    const dbValue = toValue(db);
    const vectorValue = toValue(vector);
    const enabledValue = toValue(enabled);

    if (!dbValue || !vectorValue || !enabledValue) {
      results.value = EMPTY_RESULTS;
      isSearching.value = false;
      searchTime.value = null;
      return;
    }

    const currentSearchId = ++searchId;
    isSearching.value = true;
    error.value = null;

    try {
      const startTime = performance.now();

      // Convert number[] to Float32Array if needed
      const queryVector = vectorValue instanceof Float32Array
        ? vectorValue
        : new Float32Array(vectorValue);

      // Get current filter value
      const filterValue = toValue(filter);
      const kValue = toValue(k);

      // Execute search
      const searchResults = await dbValue.search(queryVector, kValue, {
        filter: filterValue,
        includeVectors,
        includeMetadata,
      });

      const endTime = performance.now();

      // Only update if this is still the latest search and component is mounted
      if (currentSearchId === searchId && isMounted) {
        results.value = searchResults;
        searchTime.value = endTime - startTime;
        isSearching.value = false;
      }
    } catch (err) {
      if (currentSearchId === searchId && isMounted) {
        error.value = err instanceof Error ? err : new Error(String(err));
        results.value = EMPTY_RESULTS;
        isSearching.value = false;
      }
    }
  }

  async function refetch(): Promise<void> {
    await executeSearch();
  }

  // Watch all reactive dependencies
  // Use array for efficient comparison (avoids object creation overhead)
  watch(
    () => [
      toValue(db),
      toValue(vector),
      toValue(k),
      toValue(filter),
      toValue(enabled),
    ],
    () => {
      // Clear any existing debounce timer
      if (debounceTimer) {
        clearTimeout(debounceTimer);
        debounceTimer = null;
      }

      const enabledValue = toValue(enabled);
      const dbValue = toValue(db);
      const vectorValue = toValue(vector);

      // Don't search if disabled or no db/vector
      if (!enabledValue || !dbValue || !vectorValue) {
        results.value = EMPTY_RESULTS;
        isSearching.value = false;
        searchTime.value = null;
        return;
      }

      // Apply debounce if specified
      if (debounceMs > 0) {
        isSearching.value = true; // Show searching state during debounce
        debounceTimer = setTimeout(() => {
          executeSearch();
        }, debounceMs);
      } else {
        executeSearch();
      }
    },
    { immediate: true }
  );

  // Cleanup on unmount
  onUnmounted(() => {
    isMounted = false;
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
  });

  return {
    results,
    isSearching,
    error,
    searchTime,
    refetch,
  };
}
