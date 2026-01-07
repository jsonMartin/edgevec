/**
 * EdgeVec Vue 3 Composables
 *
 * Reactive composables for Vue 3 applications using EdgeVec.
 * Requires Vue 3.3+ as a peer dependency.
 *
 * @module edgevec/vue
 * @version 0.8.0
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { ref, computed } from 'vue';
 * import { useEdgeVec, useSearch } from 'edgevec/vue';
 *
 * const { db, isReady, isLoading, error, stats } = useEdgeVec({
 *   dimensions: 384,
 *   persistName: 'my-index',
 * });
 *
 * const queryVector = ref<number[] | null>(null);
 *
 * const { results, isSearching, searchTime } = useSearch(db, {
 *   vector: queryVector,
 *   k: 10,
 *   enabled: computed(() => isReady.value && queryVector.value !== null),
 *   debounceMs: 300,
 * });
 *
 * async function handleSearch(embedding: number[]) {
 *   queryVector.value = embedding;
 * }
 * </script>
 *
 * <template>
 *   <div v-if="isLoading">Loading EdgeVec...</div>
 *   <div v-else-if="error">Error: {{ error.message }}</div>
 *   <template v-else-if="isReady">
 *     <p>Loaded {{ stats?.count }} vectors ({{ stats?.dimensions }}d)</p>
 *     <div v-if="isSearching">Searching...</div>
 *     <ul v-else>
 *       <li v-for="result in results" :key="result.id">
 *         Score: {{ result.score.toFixed(4) }}
 *       </li>
 *     </ul>
 *     <p v-if="searchTime !== null">Search took {{ searchTime.toFixed(2) }}ms</p>
 *   </template>
 * </template>
 * ```
 */

// Composables
export { useEdgeVec } from './useEdgeVec.js';
export { useSearch } from './useSearch.js';

// Types
export type {
  UseEdgeVecOptions,
  UseEdgeVecResult,
  UseSearchOptions,
  UseSearchResult,
  MaybeRef,
  MaybeRefOrGetter,
} from './types.js';
