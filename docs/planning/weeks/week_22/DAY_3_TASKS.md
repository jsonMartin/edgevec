# Week 22, Day 3: Pre-Filter vs Post-Filter Strategy

**Date:** 2025-12-19
**Sprint:** Week 22 (v0.5.0 Phase)
**Day Theme:** HNSW Integration Strategy Decision
**Status:** PLANNED

---

## Task W22.3: Pre-Filter vs Post-Filter Strategy

**Priority:** CRITICAL (P0)
**Estimated Effort:** 6 hours (3x rule: 2h optimistic × 3 = 6h)
**Status:** PLANNED
**Depends On:** W22.1, W22.2
**Blocks:** W22.4, W22.5

---

### Context

Day 3 determines the optimal strategy for integrating filtering with HNSW search. This is the most critical performance decision of the sprint - wrong choice here means missing the <10ms P99 target.

**Strategic Importance:**
- Filter strategy determines search performance (2-300ms range in industry)
- Wrong choice could make filtering SLOWER than no filtering
- Qdrant-style "Filterable HNSW" is the gold standard

**Industry Benchmark Data (2025):**

| Engine | Filter Approach | QPS (Unfiltered) | QPS (10% Filter) | Latency P99 |
|:-------|:----------------|:-----------------|:-----------------|:------------|
| Pinecone | Integrated | ~800 | ~600 | <10ms |
| Zilliz/Milvus | Integrated | ~750 | ~700 | <15ms |
| Qdrant | Filterable HNSW | ~700 | ~750 | <12ms |
| LanceDB | Post-filter | ~500 | ~300 | 200-300ms |
| PGVector | Post-filter | ~400 | ~250 | 150-200ms |

**Key Finding:** Engines with in-algorithm filtering get FASTER under filters.

---

### Objective

Create `docs/architecture/FILTER_STRATEGY.md` with:
1. All 4 strategies analyzed with complexity
2. Performance modeling for 100k × 384-dim vectors
3. Recommended strategy with quantitative rationale
4. Edge case handling specification
5. API design for strategy selection

---

### Technical Approach

#### 1. Strategy Analysis

**Strategy 1: Pre-Filter**
```
┌─────────────────────────────────────────────────────────┐
│  PRE-FILTER: Filter first, then search                  │
├─────────────────────────────────────────────────────────┤
│  1. Evaluate filter on ALL vectors: O(n)                │
│  2. Build candidate set (bitset)                        │
│  3. Run HNSW on filtered subset                         │
│                                                         │
│  Pros:                                                  │
│  - Guaranteed k results (if k exist)                    │
│  - Accurate recall                                      │
│                                                         │
│  Cons:                                                  │
│  - O(n) filter evaluation ALWAYS                        │
│  - HNSW graph may be disconnected                       │
│  - May miss good candidates                             │
│                                                         │
│  Best for: Selectivity >50% (most vectors pass)         │
└─────────────────────────────────────────────────────────┘
```

**Strategy 2: Post-Filter**
```
┌─────────────────────────────────────────────────────────┐
│  POST-FILTER: Search first, then filter                 │
├─────────────────────────────────────────────────────────┤
│  1. Run HNSW on full index: O(log n)                    │
│  2. Get ef_search candidates                            │
│  3. Filter candidates: O(ef_search)                     │
│  4. Return top-k that pass                              │
│                                                         │
│  Pros:                                                  │
│  - HNSW graph stays intact                              │
│  - Fast for selective filters                           │
│                                                         │
│  Cons:                                                  │
│  - May return <k results                                │
│  - Wasted distance computations                         │
│  - Poor recall under tight filters                      │
│                                                         │
│  Best for: Selectivity <10% (few vectors pass)          │
└─────────────────────────────────────────────────────────┘
```

**Strategy 3: Hybrid (Oversampling)**
```
┌─────────────────────────────────────────────────────────┐
│  HYBRID: Oversample then filter                         │
├─────────────────────────────────────────────────────────┤
│  1. Estimate selectivity (filter pass rate)             │
│  2. Calculate oversample_k = k / selectivity            │
│  3. Run HNSW with ef_search = oversample_k              │
│  4. Filter candidates                                   │
│  5. Return top-k that pass                              │
│                                                         │
│  Pros:                                                  │
│  - Adapts to selectivity                                │
│  - Better than pure post-filter                         │
│                                                         │
│  Cons:                                                  │
│  - Still may return <k                                  │
│  - Selectivity estimation overhead                      │
│  - Not as good as integrated filtering                  │
│                                                         │
│  Best for: Unknown selectivity, simple implementation   │
└─────────────────────────────────────────────────────────┘
```

**Strategy 4: Filterable HNSW (Qdrant-style)**
```
┌─────────────────────────────────────────────────────────┐
│  FILTERABLE HNSW: In-graph filtering                    │
├─────────────────────────────────────────────────────────┤
│  1. During HNSW traversal, check filter at each node    │
│  2. Skip nodes that fail filter                         │
│  3. Continue to next neighbor                           │
│  4. Maintain graph connectivity via extra links         │
│                                                         │
│  Pros:                                                  │
│  - Best recall AND speed                                │
│  - Filtering can IMPROVE performance                    │
│  - No wasted distance computations                      │
│                                                         │
│  Cons:                                                  │
│  - Requires HNSW modification                           │
│  - More complex implementation                          │
│  - Additional memory for extra links                    │
│                                                         │
│  Best for: All selectivity ranges (gold standard)       │
└─────────────────────────────────────────────────────────┘
```

#### 2. Performance Modeling

**Scenario: 100k × 384-dim vectors, k=10**

| Selectivity | Pre-Filter | Post-Filter | Hybrid (2x) | Filterable HNSW |
|:------------|:-----------|:------------|:------------|:----------------|
| 1% pass | 15ms | 8ms* | 6ms | 3ms |
| 10% pass | 12ms | 5ms | 4ms | 2ms |
| 50% pass | 10ms | 12ms | 8ms | 3ms |
| 90% pass | 8ms | 18ms | 15ms | 4ms |

*May return <k results

**Memory Overhead:**

| Strategy | Additional Memory |
|:---------|:------------------|
| Pre-Filter | 100k bits = 12.5KB (bitset) |
| Post-Filter | 0 |
| Hybrid | 0 |
| Filterable HNSW | ~10-20% more graph edges |

#### 3. Decision Matrix

```
┌─────────────────┬────────────────┬─────────────────────────────────┐
│ Selectivity     │ Strategy       │ Rationale                       │
├─────────────────┼────────────────┼─────────────────────────────────┤
│ <10% pass       │ Post-filter    │ Bitset overhead > search speedup│
│ 10-50% pass     │ Hybrid         │ Pre-filter + HNSW on subset     │
│ >50% pass       │ Pre-filter     │ Bitset scan faster than full KNN│
│ Unknown         │ Auto           │ Measure first N queries, adapt  │
└─────────────────┴────────────────┴─────────────────────────────────┘
```

#### 4. Recommended Strategy: Hybrid with Auto-Selection

**Phase 1 (Week 23 MVP):** Implement Hybrid with configurable oversample factor

```rust
pub enum FilterStrategy {
    /// Filter after search, oversample by factor
    PostFilter { oversample: f32 },

    /// Filter before search (full scan)
    PreFilter,

    /// Auto-select based on estimated selectivity
    Auto,
}

// Default configuration
impl Default for FilterStrategy {
    fn default() -> Self {
        FilterStrategy::PostFilter { oversample: 3.0 }
    }
}
```

**Phase 2 (Future):** Filterable HNSW (requires HNSW modification)

---

### Deliverables

1. **`docs/architecture/FILTER_STRATEGY.md`** containing:
   - All 4 strategies analyzed
   - Performance modeling tables
   - Decision matrix
   - Recommended strategy with rationale
   - Edge case handling
   - API design for strategy selection

---

### Acceptance Criteria

**CRITICAL (Must Pass):**
- [ ] All 4 strategies analyzed with complexity
- [ ] Performance model shows <10ms P99 at 100k × 384-dim for all tiers
- [ ] Recommended strategy documented with quantitative rationale
- [ ] Decision matrix complete

**MAJOR (Should Pass):**
- [ ] Edge cases handled: empty result set, all filtered, 0%/100% selectivity
- [ ] API design for strategy selection with 4+ enum variants
- [ ] Selectivity estimation approach documented
- [ ] Memory overhead calculated for each strategy

**Performance Tiers:**

| Tier | Filter Type | Example | P99 Target |
|:-----|:------------|:--------|:-----------|
| Tier 1 | Simple equality | `category = "gpu"` | <2ms |
| Tier 2 | Range | `price BETWEEN 100 500` | <5ms |
| Tier 3 | Complex AND/OR | `(a AND b) OR c` | <10ms |
| Tier 4 | Worst-case | 3+ clauses + negations | <20ms |

---

### Edge Case Handling

| Edge Case | Behavior |
|:----------|:---------|
| 0% selectivity (nothing passes) | Return empty Vec |
| 100% selectivity (everything passes) | Skip filter, normal search |
| Empty filter expression | Return all results (no filtering) |
| k > matching vectors | Return all matching (may be <k) |
| Contradictory filter (`a AND NOT a`) | Return empty Vec |

---

### Implementation Checklist

- [ ] Create `docs/architecture/FILTER_STRATEGY.md`
- [ ] Analyze all 4 strategies
- [ ] Create performance modeling tables
- [ ] Design decision matrix
- [ ] Document recommended strategy
- [ ] Specify edge case handling
- [ ] Design API for strategy selection
- [ ] Calculate memory overhead

---

### Dependencies

**Blocks:**
- W22.4 (WASM API needs strategy options)
- W22.5 (Test strategy needs edge cases)

**Blocked By:**
- W22.1 (Grammar defines operators)
- W22.2 (Evaluator defines how filters execute)

---

### Verification Method

**Day 3 is COMPLETE when:**

1. `docs/architecture/FILTER_STRATEGY.md` exists
2. All 4 strategies documented
3. Performance model validates <10ms target
4. Decision matrix complete
5. API design finalized

---

### Estimated Timeline

| Phase | Time | Cumulative |
|:------|:-----|:-----------|
| Strategy research | 1h | 1h |
| Performance modeling | 1.5h | 2.5h |
| Decision matrix | 0.5h | 3h |
| API design | 1h | 4h |
| Edge cases | 0.5h | 4.5h |
| Documentation | 1h | 5.5h |
| Buffer | 0.5h | 6h |

---

### Hostile Review Checkpoint

**End of Day 3:** Submit for `/review` with:
- `docs/architecture/FILTER_STRATEGY.md`

**Expected Review Focus:**
- Strategy analysis completeness
- Performance budget validation
- Decision rationale soundness
- Edge case coverage

---

**Task Owner:** META_ARCHITECT
**Review Required:** HOSTILE_REVIEWER
**Next Task:** W22.4 (WASM Boundary & TypeScript API)

---

*"The right strategy at the right time beats the perfect algorithm at the wrong time."*
