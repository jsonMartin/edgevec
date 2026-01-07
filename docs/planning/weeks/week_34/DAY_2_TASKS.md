# Week 34 Day 2: Vue Implementation

**Date:** 2026-01-21
**Focus:** Implement Vue composables
**Hours:** 2h
**Status:** [x] COMPLETE

---

## Objectives

Implement the core Vue composables based on Day 1 design.

---

## Tasks

### W34.1.2: Implement useEdgeVec and useSearch (2h)

**Goal:** Working Vue composables with TypeScript.

**Subtasks:**

- [ ] **2.1** Create `pkg/vue/` directory structure (10min)
  ```
  pkg/vue/
  ├── index.ts
  ├── types.ts
  ├── useEdgeVec.ts
  └── useSearch.ts
  ```

- [ ] **2.2** Implement `useEdgeVec` composable (45min)
  - Create reactive refs for all state
  - Implement async initialization in `onMounted()`
  - Handle persistence loading
  - Implement `reload()` and `save()` functions
  - Cleanup in `onUnmounted()`

- [ ] **2.3** Implement `useSearch` composable (45min)
  - Create reactive refs for results, searching, error, time
  - Use `watch()` for reactive search triggers
  - Implement debounce with `setTimeout`
  - Handle race conditions with search ID counter
  - Support both ref and raw value inputs

- [ ] **2.4** Verify TypeScript compilation (20min)
  - Run `npx tsc --noEmit`
  - Fix any type errors
  - Ensure strict mode passes

---

## Implementation Notes

### useEdgeVec Pattern

```typescript
import { ref, onMounted, onUnmounted } from 'vue';
import type { Ref } from 'vue';
import { EdgeVecIndex, IndexConfig } from '../edgevec-wrapper.js';
import type { UseEdgeVecOptions, UseEdgeVecResult } from './types.js';

export function useEdgeVec(options: UseEdgeVecOptions): UseEdgeVecResult {
  const { dimensions, persistName, efConstruction = 200, m = 16 } = options;

  const db = ref<EdgeVecIndex | null>(null);
  const isReady = ref(false);
  const isLoading = ref(true);
  const error = ref<Error | null>(null);
  const stats = ref<{ count: number; dimensions: number } | null>(null);

  let mounted = true;

  const initialize = async () => {
    // ... similar to React but with Vue refs
  };

  onMounted(() => {
    mounted = true;
    initialize();
  });

  onUnmounted(() => {
    mounted = false;
  });

  return { db, isReady, isLoading, error, stats, reload, save };
}
```

### useSearch Pattern

```typescript
import { ref, watch, toValue } from 'vue';
import type { Ref, MaybeRef } from 'vue';

export function useSearch(
  db: MaybeRef<EdgeVecIndex | null>,
  options: UseSearchOptions
): UseSearchResult {
  const results = ref<SearchResult[]>([]);
  const isSearching = ref(false);
  // ...

  // Watch for changes and trigger search
  watch(
    () => [toValue(db), toValue(options.vector), toValue(options.enabled)],
    async () => {
      // Execute search with debounce
    },
    { immediate: true }
  );

  return { results, isSearching, error, searchTime, refetch };
}
```

---

## Verification

- [ ] `pkg/vue/useEdgeVec.ts` created
- [ ] `pkg/vue/useSearch.ts` created
- [ ] TypeScript compiles without errors
- [ ] Basic functionality works

---

## Dependencies

- Vue 3.3+ (composition API)
- `@vue/reactivity` types

---

## Next

Day 3: Complete types, exports, and documentation
