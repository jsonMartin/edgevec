# Week 34 Day 1: Vue Composables Design

**Date:** 2026-01-20
**Focus:** Design Vue 3 composables API
**Hours:** 2h
**Status:** [x] COMPLETE

---

## Objectives

Design Vue 3 composables that match React hooks functionality while following Vue idioms.

---

## Tasks

### W34.1.1: Design Vue Composables API (2h)

**Goal:** Create design document for Vue composables.

**Subtasks:**

- [ ] **1.1** Study Vue 3 Composition API patterns (30min)
  - Review `ref()`, `reactive()`, `computed()`, `watch()`
  - Understand `onMounted()`, `onUnmounted()` lifecycle
  - Compare to React hooks patterns

- [ ] **1.2** Design `useEdgeVec` composable (30min)
  - Define options interface
  - Define return type with Vue refs
  - Handle async WASM initialization
  - Plan persistence loading flow

- [ ] **1.3** Design `useSearch` composable (30min)
  - Define options interface (support ref inputs)
  - Define return type with Vue refs
  - Plan debounce implementation with `watch()`
  - Handle race conditions

- [ ] **1.4** Create type definitions (30min)
  - Write `pkg/vue/types.ts` draft
  - Ensure compatibility with Vue's `Ref<T>` types
  - Document all interfaces

---

## Design Document

### useEdgeVec Composable

```typescript
import { ref, onMounted, onUnmounted } from 'vue';
import type { Ref } from 'vue';

interface UseEdgeVecOptions {
  dimensions: number;
  persistName?: string;
  efConstruction?: number;
  m?: number;
}

interface UseEdgeVecResult {
  db: Ref<EdgeVecIndex | null>;
  isReady: Ref<boolean>;
  isLoading: Ref<boolean>;
  error: Ref<Error | null>;
  stats: Ref<{ count: number; dimensions: number } | null>;
  reload: () => Promise<void>;
  save: (name?: string) => Promise<void>;
}

function useEdgeVec(options: UseEdgeVecOptions): UseEdgeVecResult;
```

### useSearch Composable

```typescript
import { ref, watch, computed } from 'vue';
import type { Ref, ComputedRef } from 'vue';

interface UseSearchOptions {
  vector: Ref<Float32Array | number[] | null> | Float32Array | number[] | null;
  k?: number;
  filter?: FilterExpression | string;
  enabled?: Ref<boolean> | ComputedRef<boolean> | boolean;
  debounceMs?: number;
  includeVectors?: boolean;
  includeMetadata?: boolean;
}

interface UseSearchResult {
  results: Ref<SearchResult[]>;
  isSearching: Ref<boolean>;
  error: Ref<Error | null>;
  searchTime: Ref<number | null>;
  refetch: () => Promise<void>;
}

function useSearch(
  db: Ref<EdgeVecIndex | null> | EdgeVecIndex | null,
  options: UseSearchOptions
): UseSearchResult;
```

### Key Vue Differences from React

| React | Vue | Notes |
|:------|:----|:------|
| `useState()` | `ref()` | Vue refs are reactive |
| `useEffect()` | `watch()` / `watchEffect()` | Vue has more granular reactivity |
| `useCallback()` | Regular functions | Vue doesn't need memoization |
| `useRef()` | `ref()` (same) | Both use refs for mutable values |
| Dependency array | Automatic tracking | Vue tracks dependencies automatically |

---

## Verification

- [ ] Design document complete
- [ ] Type definitions drafted
- [ ] Vue patterns understood
- [ ] Ready for implementation

---

## Notes

- Vue's reactivity system tracks dependencies automatically (no dep arrays)
- Need to handle both `Ref<T>` and raw values in options
- Use `toValue()` helper to unwrap refs
- Consider `MaybeRef<T>` utility type for flexible inputs

---

## Next

Day 2: Implement useEdgeVec and useSearch composables
