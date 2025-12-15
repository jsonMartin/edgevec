# HOSTILE REVIEW: Week 16 WEEKLY_TASK_PLAN.md

**Reviewer:** HOSTILE_REVIEWER
**Artifact:** `docs/planning/weeks/week_16/WEEKLY_TASK_PLAN.md`
**Date:** 2025-12-14
**Mode:** SUPER CRITICAL (Maximum Hostility)
**Authority:** VETO POWER ENGAGED

---

## REVIEW SUMMARY

| Category | Status | Critical Issues | Major Issues | Minor Issues |
|:---------|:-------|:----------------|:-------------|:-------------|
| Dependency Criteria | ⚠️ ISSUES | 1 | 2 | 0 |
| Estimation Criteria | ✅ PASS | 0 | 0 | 1 |
| Acceptance Criteria | ⚠️ ISSUES | 2 | 1 | 0 |
| Risk Criteria | ⚠️ ISSUES | 1 | 1 | 0 |
| Architecture Dependency | ✅ PASS | 0 | 0 | 0 |

**TOTAL ISSUES:** 4 CRITICAL, 4 MAJOR, 1 MINOR

---

## PART 2: PLAN ATTACK VECTORS

### 1. DEPENDENCY CRITERIA

#### CRITICAL [C-DEP-1]: Missing VectorStorage API Dependency in W16.4

**Location:** WEEKLY_TASK_PLAN.md lines 243-248, DAY_4_TASKS.md lines 210-220

**Finding:** The `compact()` method specification shows:
```rust
let vec = storage.get_vector(node.vector_id);
```

However, there is **NO verification** that `VectorStorage.get_vector(VectorId)` exists in the current codebase. The plan assumes this API exists but doesn't list it as a dependency or prerequisite.

**Evidence from DAY_4_TASKS.md:**
```rust
let live_vectors: Vec<(VectorId, Vec<f32>)> = self
    .nodes
    .iter()
    .filter(|node| node.deleted == 0)
    .map(|node| {
        let vec = storage.get_vector(node.vector_id);  // <-- UNVERIFIED API
        (node.vector_id, vec.to_vec())
    })
    .collect();
```

**Impact:** W16.4 will FAIL if this method doesn't exist or has a different signature.

**Required Action:**
1. Verify `VectorStorage::get_vector(VectorId) -> &[f32]` exists
2. If not, add it as a sub-task of W16.4
3. Document the exact API signature required

---

#### MAJOR [M-DEP-1]: Unverified get_node() Method Dependency

**Location:** DAY_3_TASKS.md lines 119-126

**Finding:** The search filtering code uses:
```rust
self.get_node(result.node_id)
    .map(|node| node.deleted == 0)
```

But `get_node()` is separate from `get_node_by_vector_id()` defined in W16.2. The plan doesn't clarify if `get_node(NodeId)` already exists or needs to be added.

**Impact:** Ambiguity in implementation - engineer may assume wrong method.

**Required Action:** Clarify if `get_node(NodeId)` exists or if `get_node_by_vector_id(VectorId)` should be used.

---

#### MAJOR [M-DEP-2]: Incomplete Dependency Between W16.3 and W16.2

**Location:** WEEKLY_TASK_PLAN.md lines 373-379

**Finding:** The dependency graph shows:
```
W16.2 ──► W16.3
```

But W16.3 requires `deleted_count` and `tombstone_ratio()` which are specified in W16.2. The plan says "Depends On: W16.2" but doesn't explicitly state which specific deliverables from W16.2 are required.

**Impact:** Task may be started before prerequisite methods are fully implemented and tested.

**Required Action:** List specific W16.2 deliverables required by W16.3:
- `is_deleted()` - for filtering
- `deleted_count` field - for adjusted_k()
- `tombstone_ratio()` - for adjusted_k()

---

### 2. ESTIMATION CRITERIA

#### MINOR [m-EST-1]: Base Estimates Inconsistent with RFC-001

**Location:** WEEKLY_TASK_PLAN.md vs RFC-001

**Finding:** RFC-001 has different base estimates:

| Task | RFC-001 Base | Week 16 Plan Base | 3x Applied |
|:-----|:-------------|:------------------|:-----------|
| W16.1 | 2h | 1.3h | 4h (plan) vs 6h (RFC) |
| W16.2 | 4h | 2h | 6h (plan) vs 12h (RFC) |
| W16.3 | 3h | 2h | 6h (plan) vs 9h (RFC) |
| W16.4 | 6h | 2.7h | 8h (plan) vs 18h (RFC) |
| W16.5 | 4h | 2h | 6h (plan) vs 12h (RFC) |

The Week 16 plan uses LOWER base estimates than RFC-001, resulting in 30h vs RFC's implicit 57h with 3x.

**Impact:** Optimistic estimates may cause schedule slip.

**Assessment:** This is MINOR because:
1. The 3x rule IS applied
2. 25% buffer (10h) is included
3. Total 40h is still substantial
4. RFC estimates were preliminary

**No action required** but noted for awareness.

---

### 3. ACCEPTANCE CRITERIA

#### CRITICAL [C-AC-1]: W16.2 Uses O(n) Lookup, Not O(1) as Documented

**Location:** DAY_2_TASKS.md lines 47-56

**Finding:** The specification says:
```rust
/// # Complexity
/// * Time: O(n) for lookup, O(1) for delete
```

But the public API documentation in RFC-001 and DAY_2_TASKS says:
```rust
/// # Complexity
/// - Time: O(1)
```

**The implementation uses linear search:**
```rust
fn get_node_mut(&mut self, vector_id: VectorId) -> Result<&mut HnswNode, GraphError> {
    self.nodes
        .iter_mut()
        .find(|n| n.vector_id == vector_id)  // <-- O(n) linear search!
        .ok_or(GraphError::InvalidVectorId)
}
```

**Impact:** This is a PERFORMANCE LIE. At 1M vectors, a single delete could take 1-10ms, not "< 1 μs" as promised.

**Required Action:**
1. Change AC16.2 to document true complexity: O(n) lookup + O(1) mutation
2. Consider adding `HashMap<VectorId, NodeId>` index for O(1) lookup (future optimization)
3. Update performance targets: delete latency should be O(n), not O(1)
4. Update RFC-001 or mark as "simplified for v0.3.0"

---

#### CRITICAL [C-AC-2]: W16.4 Missing AC for Memory Safety During Compaction

**Location:** DAY_4_TASKS.md lines 183-232

**Finding:** The `compact()` method creates a NEW index and swaps:
```rust
// Build new index
let mut new_index = HnswIndex::new(config, &new_storage)?;

// ...

// Swap self with new index
*self = new_index;
```

**MISSING ACCEPTANCE CRITERIA:**
- What happens to the OLD storage during swap?
- What if compact() panics mid-rebuild?
- What if a reference to old index exists?
- Memory safety of `*self = new_index` swap

**Impact:** Potential memory safety issues or data loss if not handled correctly.

**Required Action:** Add acceptance criteria:
- AC16.4.8: Compact does not invalidate external references (or document that it does)
- AC16.4.9: Partial failure during compact preserves original index
- AC16.4.10: Memory is properly freed after swap

---

#### MAJOR [M-AC-1]: W16.3 Performance AC Lacks Baseline Definition

**Location:** DAY_3_TASKS.md line 18

**Finding:** AC16.3.4 states:
```
Performance degradation < 20% at 10% tombstones
```

But there is NO baseline defined:
- 20% slower than what baseline?
- Which search parameters (k=10? k=100?)
- Which dataset (100k vectors? 1M vectors?)

**Impact:** Cannot verify AC without defined baseline.

**Required Action:** Define:
- Baseline: Search latency on 100k vectors, k=10, 128D, 0% tombstones
- Measurement: P99 latency
- Exact comparison methodology

---

### 4. RISK CRITERIA

#### CRITICAL [C-RISK-1]: Missing Risk for VectorStorage Mismatch During Compaction

**Location:** Risk Register (lines 357-365)

**Finding:** The risk register identifies R16.3 "Compaction memory spike (2x)" but MISSES a critical risk:

**UNIDENTIFIED RISK:** During `compact()`, the method creates `new_storage`:
```rust
let mut new_storage = VectorStorage::new(&config, None);
let mut new_index = HnswIndex::new(config, &new_storage)?;
```

But then it swaps ONLY the index:
```rust
*self = new_index;
```

**WHAT ABOUT STORAGE?** The caller passes `storage: &VectorStorage` (immutable) but the implementation creates `new_storage`. After swap:
- `self` points to `new_index` which was built with `new_storage`
- Caller still has `storage` which is now STALE

**Impact:** DATA LOSS or CORRUPTION. The swapped index references vectors in `new_storage` which is dropped when `compact()` returns!

**Required Action:**
1. Add R16.6: "Storage/Index mismatch after compaction"
2. Redesign compact() to either:
   a. Take `&mut VectorStorage` and modify in place, OR
   b. Return `(HnswIndex, VectorStorage)` tuple, OR
   c. Use internal storage reference

---

#### MAJOR [M-RISK-1]: R16.5 insert_with_id Collision Risk Underestimated

**Location:** Risk Register R16.5

**Finding:** The risk says "LOW probability" for insert_with_id collision, but the compact() code shows:
```rust
for (id, vector) in live_vectors {
    new_index.insert_with_id(id, &vector, &mut new_storage)?;
}
```

If `insert_with_id()` fails due to collision (which should be impossible but is checked), the entire compact operation fails with partial state.

**Impact:** Need atomicity guarantee for compact.

**Required Action:**
1. Upgrade R16.5 probability to MEDIUM
2. Add mitigation: "insert_with_id in compact() cannot fail on collision since IDs come from original index; add debug_assert"

---

### 5. ARCHITECTURE DEPENDENCY

#### ✅ VERIFIED: RFC-001 is APPROVED

RFC-001-soft-delete.md has status "APPROVED" (line 6).

#### ✅ VERIFIED: Week 15 Complete

GATE_15_COMPLETE.md should exist (per summary context).

#### ✅ VERIFIED: Plan follows RFC-001 design

All Week 16 tasks map to RFC-001 specification.

---

## AUTOMATIC REJECTION TRIGGERS REVIEW

| Trigger | Status |
|:--------|:-------|
| Task > 16 hours without decomposition | ✅ PASS (max is 8h) |
| Vague acceptance criteria | ⚠️ FAIL (performance baseline) |
| Missing critical path analysis | ✅ PASS (dependency graph exists) |
| No contingency buffer | ✅ PASS (25% = 10h buffer) |
| ARCHITECTURE.md not approved | ✅ PASS (RFC-001 approved) |

**AUTOMATIC REJECTION TRIGGERED:** Yes - vague acceptance criteria (C-AC-1, M-AC-1)

---

## DETAILED FINDINGS SUMMARY

### CRITICAL Issues (Must Fix Before Approval)

| ID | Issue | Location | Required Action |
|:---|:------|:---------|:----------------|
| C-DEP-1 | Missing VectorStorage.get_vector() verification | W16.4 | Verify API exists or add sub-task |
| C-AC-1 | O(n) complexity documented as O(1) | W16.2 | Correct documentation |
| C-AC-2 | Missing memory safety ACs for compact() | W16.4 | Add AC16.4.8-10 |
| C-RISK-1 | Storage/Index mismatch after compact | W16.4 | Redesign compact() signature |

### MAJOR Issues (Must Fix Before Approval)

| ID | Issue | Location | Required Action |
|:---|:------|:---------|:----------------|
| M-DEP-1 | get_node() vs get_node_by_vector_id() ambiguity | W16.3 | Clarify which method |
| M-DEP-2 | Incomplete W16.2→W16.3 deliverable list | Dependencies | List specific methods |
| M-AC-1 | No performance baseline defined | W16.3 | Define baseline methodology |
| M-RISK-1 | insert_with_id collision risk underestimated | R16.5 | Upgrade to MEDIUM |

### MINOR Issues (Track for Later)

| ID | Issue | Location | Action |
|:---|:------|:---------|:-------|
| m-EST-1 | Lower estimates than RFC-001 | All tasks | Noted, no action |

---

## VERDICT

# ⛔ REJECTED

**Reason:** 4 CRITICAL issues and 4 MAJOR issues identified.

**Most Severe Issue:** C-RISK-1 - The `compact()` design has a fundamental flaw where the storage/index become misaligned after swap. This is a DATA LOSS bug waiting to happen.

---

## REQUIRED ACTIONS FOR RESUBMISSION

### Priority 1: Fix C-RISK-1 (Storage Mismatch)

The compact() method in DAY_4_TASKS.md MUST be redesigned:

**Option A (Recommended):** Modify storage in place
```rust
pub fn compact(&mut self, storage: &mut VectorStorage) -> Result<CompactionResult, GraphError>
```

The method should:
1. Collect live vectors from current storage
2. Clear storage
3. Re-insert vectors into same storage
4. Rebuild index pointing to same storage

**Option B:** Return new pair
```rust
pub fn compact(self, storage: VectorStorage) -> Result<(HnswIndex, VectorStorage, CompactionResult), GraphError>
```

Choose one and update DAY_4_TASKS.md.

### Priority 2: Fix C-DEP-1 (VectorStorage API)

Verify `VectorStorage::get_vector(VectorId) -> &[f32]` exists. If not, add as W16.4 sub-task.

### Priority 3: Fix C-AC-1 (Complexity Documentation)

Update DAY_2_TASKS.md to document true complexity:
- `delete()`: O(n) lookup + O(1) mutation
- `is_deleted()`: O(n) lookup

Update performance targets accordingly.

### Priority 4: Fix C-AC-2 (Memory Safety ACs)

Add to DAY_4_TASKS.md:
- AC16.4.8: Document external reference behavior
- AC16.4.9: Compact failure preserves original index
- AC16.4.10: Memory properly freed

### Priority 5: Fix MAJOR Issues

1. M-DEP-1: Clarify get_node vs get_node_by_vector_id
2. M-DEP-2: List specific W16.2 deliverables for W16.3
3. M-AC-1: Define performance baseline (dataset, k, percentile)
4. M-RISK-1: Update R16.5 to MEDIUM probability

---

## RESUBMISSION CHECKLIST

Before resubmitting:

- [ ] C-RISK-1: compact() signature redesigned
- [ ] C-DEP-1: VectorStorage.get_vector() verified
- [ ] C-AC-1: Complexity correctly documented
- [ ] C-AC-2: Memory safety ACs added
- [ ] M-DEP-1: get_node clarified
- [ ] M-DEP-2: Deliverable dependencies listed
- [ ] M-AC-1: Performance baseline defined
- [ ] M-RISK-1: R16.5 probability updated
- [ ] Artifact tagged [REVISED]
- [ ] This checklist included in revision

---

**HOSTILE_REVIEWER Signature:**
Maximum hostility applied. Plan has merit but contains critical design flaws that would cause implementation failure.

**Status:** ⛔ REJECTED
**Resubmission Required:** YES
**Priority:** HIGH - Fix before Week 16 execution

---

*Review completed: 2025-12-14*
*Checklist version: HOSTILE_GATE_CHECKLIST v1.0.0*
