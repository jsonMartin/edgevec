# PROMPT: SIMD Architecture Design

**Target Agent:** META_ARCHITECT
**Command:** `/architect-design simd_quantization`
**Priority:** P0 (BLOCKING — All implementation depends on this)
**Estimated Time:** 1 hour
**Output:** `docs/architecture/SIMD_DESIGN.md`

---

## MISSION

Design the SIMD acceleration architecture for EdgeVec's binary quantization module. Define the module structure, dispatch strategy, safety model, and platform support matrix.

**Core Challenge:** Achieve <50 CPU cycles for 768-bit Hamming distance while maintaining:
- Portability (x86_64, ARM64, WASM)
- Safety (documented unsafe boundaries)
- Compatibility (no breaking API changes)

---

## CONTEXT FILES TO LOAD

```bash
# REQUIRED - Read these first
cat src/quantization/binary.rs           # Current implementation
cat src/quantization/mod.rs              # Module structure
cat docs/architecture/DATA_LAYOUT.md     # Memory alignment (64-byte)

# REFERENCE
cat docs/reviews/2025-12-12_W8D36_*_APPROVED.md  # Day 36 approval
```

---

## DESIGN DECISIONS REQUIRED

### Decision 1: Module Structure

**Option A — Flat Structure:**
```
src/quantization/
├── binary.rs      # Public API + portable
├── simd.rs        # All SIMD code (AVX2, NEON, etc.)
├── scalar.rs      # Existing SQ8
└── mod.rs
```
- Pros: Simple, single file for all SIMD
- Cons: File may grow large, harder to test per-platform

**Option B — Hierarchical Structure:**
```
src/quantization/
├── binary.rs      # Public API
├── simd/
│   ├── mod.rs     # Dispatch logic
│   ├── avx2.rs    # x86_64 AVX2
│   ├── neon.rs    # ARM64 NEON
│   ├── wasm.rs    # WebAssembly SIMD
│   └── portable.rs # std::simd fallback
├── scalar.rs
└── mod.rs
```
- Pros: Clean separation, easy per-platform testing
- Cons: More files, more complex module tree

**Option C — Feature-Gated Inline:**
```
src/quantization/
├── binary.rs      # All code inline with #[cfg] gates
├── scalar.rs
└── mod.rs
```
- Pros: Everything in one place
- Cons: Hard to read, massive file

**YOUR TASK:** Choose one and justify.

---

### Decision 2: Dispatch Strategy

**Option A — Compile-Time Only:**
```rust
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub fn hamming_distance(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    unsafe { avx2::hamming(a, b) }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
pub fn hamming_distance(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    portable::hamming(a, b)
}
```
- Pros: Zero runtime overhead, compiler optimizes fully
- Cons: Binary specific to compile target, no runtime adaptation

**Option B — Runtime Detection:**
```rust
pub fn hamming_distance(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        return unsafe { avx2::hamming(a, b) };
    }
    portable::hamming(a, b)
}
```
- Pros: Single binary works everywhere, adapts to CPU
- Cons: Branch on every call (mitigated by branch prediction)

**Option C — Cached Runtime Detection:**
```rust
use std::sync::OnceLock;

type HammingFn = fn(&[u8; 96], &[u8; 96]) -> u32;
static HAMMING_IMPL: OnceLock<HammingFn> = OnceLock::new();

pub fn hamming_distance(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    let f = HAMMING_IMPL.get_or_init(|| {
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") {
            return avx2::hamming as HammingFn;
        }
        portable::hamming
    });
    f(a, b)
}
```
- Pros: Detection once, then direct call
- Cons: Function pointer indirection (usually negligible)

**YOUR TASK:** Choose one and justify.

---

### Decision 3: Safety Model

All SIMD intrinsics are `unsafe`. Define the safety boundaries:

```rust
/// SAFETY CONTRACT for SIMD functions:
///
/// 1. CPU Feature Requirement
///    - Caller MUST verify CPU supports required features
///    - Use is_x86_feature_detected!() or equivalent
///    - Compile-time: #[target_feature(enable = "avx2")]
///
/// 2. Alignment Requirement
///    - Input arrays MUST be 32-byte aligned (AVX2) or 64-byte (AVX-512)
///    - QuantizedVector guarantees 64-byte alignment via #[repr(align(64))]
///
/// 3. Size Requirement
///    - Input arrays MUST be exactly 96 bytes
///    - Enforced by type system: &[u8; 96]
///
/// 4. No Aliasing
///    - Input references must not alias
///    - Enforced by Rust borrow checker
```

**YOUR TASK:** Define the complete safety model with verification strategy.

---

### Decision 4: Platform Priority

Given 8-hour constraint, which platforms to support?

| Platform | Instruction Set | Market Share | Priority |
|:---------|:----------------|:-------------|:---------|
| x86_64 Desktop | AVX2 | ~70% | ? |
| x86_64 Server | AVX-512 | ~20% servers | ? |
| ARM64 (Apple, AWS) | NEON | ~15% | ? |
| WebAssembly | WASM SIMD | 100% browsers | ? |
| Fallback | Portable | 100% | P0 (required) |

**YOUR TASK:** Assign priorities (P0/P1/P2/Defer) with justification.

---

### Decision 5: API Integration

How does SIMD integrate with existing `QuantizedVector`?

**Option A — Transparent Replacement:**
```rust
impl QuantizedVector {
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        simd::hamming(&self.data, &other.data)  // Dispatch internally
    }
}
```
- User code unchanged
- SIMD is implementation detail

**Option B — Explicit SIMD Method:**
```rust
impl QuantizedVector {
    pub fn hamming_distance(&self, other: &Self) -> u32 { /* portable */ }
    pub fn hamming_distance_simd(&self, other: &Self) -> u32 { /* simd */ }
}
```
- User chooses explicitly
- More control, more API surface

**Option C — Feature Flag:**
```rust
#[cfg(feature = "simd")]
impl QuantizedVector {
    pub fn hamming_distance(&self, other: &Self) -> u32 { /* simd */ }
}
#[cfg(not(feature = "simd"))]
impl QuantizedVector {
    pub fn hamming_distance(&self, other: &Self) -> u32 { /* portable */ }
}
```
- Compile-time choice
- Smaller binary if SIMD disabled

**YOUR TASK:** Choose integration strategy.

---

## OUTPUT FORMAT

Create `docs/architecture/SIMD_DESIGN.md`:

```markdown
# EdgeVec SIMD Architecture

**Version:** 1.0.0
**Author:** META_ARCHITECT
**Date:** 2025-12-12
**Status:** [PROPOSED]

---

## 1. Executive Summary

[2-3 sentences on chosen approach]

---

## 2. Module Structure

### Chosen: [Option A/B/C]

[Justification]

### File Layout
```
[Actual file tree]
```

---

## 3. Dispatch Strategy

### Chosen: [Option A/B/C]

[Justification with performance analysis]

### Code Pattern
```rust
[Example dispatch code]
```

---

## 4. Platform Support

| Platform | Priority | Status | Notes |
|:---------|:---------|:-------|:------|
| ... | ... | ... | ... |

### Rationale
[Why these priorities]

---

## 5. Safety Model

### Invariants
1. [Invariant 1]
2. [Invariant 2]
...

### Unsafe Boundaries
```rust
// Example of properly documented unsafe
```

### Verification Strategy
- [ ] [How to verify invariant 1]
- [ ] [How to verify invariant 2]

---

## 6. API Design

### Public API (Unchanged)
```rust
impl QuantizedVector {
    pub fn hamming_distance(&self, other: &Self) -> u32;
}
```

### Internal API
```rust
// New internal functions
```

---

## 7. Performance Projections

| Implementation | Expected Cycles | Verification |
|:---------------|:----------------|:-------------|
| Portable | ~300 | Baseline |
| AVX2 | ~40 | Benchmark |
| ... | ... | ... |

---

## 8. Testing Strategy

- [ ] Unit tests per platform
- [ ] Property tests (SIMD == portable)
- [ ] Benchmark regression tests
- [ ] Miri for unsafe verification

---

## 9. Risks

| Risk | Mitigation |
|:-----|:-----------|
| ... | ... |

---

## 10. Approval

| Role | Status |
|:-----|:-------|
| META_ARCHITECT | PROPOSED |
| HOSTILE_REVIEWER | PENDING |
```

---

## CONSTRAINTS

- **MUST NOT** change `QuantizedVector` public API
- **MUST NOT** change struct layout (64-byte aligned, 96 bytes)
- **MUST** provide portable fallback
- **MUST** document all unsafe blocks
- **SHOULD** prioritize AVX2 (widest support)
- **MAY** defer ARM NEON to future if time-constrained

---

## ACCEPTANCE CRITERIA (BINARY)

**Critical (Must Pass for Approval):**
- [ ] Module structure decision documented in Section 2 with exact option (A/B/C) + justification ≥3 sentences
  - Verify: `grep -E "### Chosen: Option [ABC]" docs/architecture/SIMD_DESIGN.md`
  - Expected: Exactly one match showing Option A, B, or C
- [ ] Dispatch strategy documented in Section 3 with exact option (A/B/C) + cycle overhead calculation
  - Verify: `grep -E "(Compile-Time Only|Runtime Detection|Cached Runtime)" docs/architecture/SIMD_DESIGN.md | head -1`
  - Expected: Match indicating which strategy chosen (e.g., "### Chosen: Runtime Detection")
- [ ] Safety model includes ≥4 invariants (enumerated in Section 5)
  - Verify: `grep -A 30 "### Invariants" docs/architecture/SIMD_DESIGN.md | grep -c "^[0-9]\+\."`
  - Expected: Count ≥ 4
- [ ] Safety model includes verification bash commands for each invariant (Section 5)
  - Verify: `grep -A 50 "Verification Strategy" docs/architecture/SIMD_DESIGN.md | grep -c "\`\`\`bash"`
  - Expected: Count ≥ 1 (bash code blocks present)
- [ ] Platform priorities assigned (P0/P1/P2/Defer) for all 5 platforms (Section 4 table)
  - Verify: `grep -c "| .* | .* | P[012] \\| Defer |" docs/architecture/SIMD_DESIGN.md`
  - Expected: Count ≥ 5 (5 platforms with priorities)
- [ ] API integration documented in Section 6 with exact option (A/B/C) + migration impact analysis
  - Verify: `grep -E "(Transparent Replacement|Explicit SIMD Method|Feature Flag)" docs/architecture/SIMD_DESIGN.md | head -1`
  - Expected: Match indicating which API strategy chosen
- [ ] Performance projections calculated from operation counts, not estimated (Section 7 with arithmetic)
  - Verify: `grep -E "[0-9]+ cycles" docs/architecture/SIMD_DESIGN.md | head -1`
  - Expected: Specific cycle counts present (not ranges or "~")
- [ ] File `docs/architecture/SIMD_DESIGN.md` created with all 10 sections complete
  - Verify: `test -f docs/architecture/SIMD_DESIGN.md && grep -c "^## " docs/architecture/SIMD_DESIGN.md`
  - Expected: File exists AND count ≥ 10

**Verification Script (Run All Checks):**
```bash
cd docs/architecture

# Check file exists
test -f SIMD_DESIGN.md || { echo "❌ FAIL: File missing"; exit 1; }

# Check module structure decision explicit
grep -E "### Chosen: Option [ABC]" SIMD_DESIGN.md || { echo "❌ FAIL: No explicit module structure choice"; exit 1; }

# Check dispatch strategy explicit
grep -E "### Chosen: (Compile-Time Only|Runtime Detection|Cached Runtime)" SIMD_DESIGN.md || { echo "❌ FAIL: No explicit dispatch strategy choice"; exit 1; }

# Check safety invariants count (≥4)
COUNT=$(grep -A 30 "### Invariants" SIMD_DESIGN.md | grep -c "^[0-9]\+\." || echo 0)
[ "$COUNT" -ge 4 ] || { echo "❌ FAIL: Only $COUNT invariants (need ≥4)"; exit 1; }

# Check verification strategy has bash commands
grep -A 50 "Verification Strategy" SIMD_DESIGN.md | grep -q "\`\`\`bash" || { echo "❌ FAIL: No bash verification commands"; exit 1; }

# Check platform priorities (≥5)
PLATFORMS=$(grep -c "| .* | .* | P[012]" SIMD_DESIGN.md || echo 0)
[ "$PLATFORMS" -ge 5 ] || { echo "❌ FAIL: Only $PLATFORMS platforms prioritized (need ≥5)"; exit 1; }

# Check API integration explicit
grep -E "(Transparent Replacement|Explicit SIMD Method|Feature Flag)" SIMD_DESIGN.md | grep -q "Chosen:" || { echo "❌ FAIL: No explicit API integration choice"; exit 1; }

# Check performance has specific cycle counts
grep -E "[0-9]+ cycles" SIMD_DESIGN.md | head -1 || { echo "❌ FAIL: No specific cycle projections"; exit 1; }

# Check all sections present (≥10)
SECTIONS=$(grep -c "^## " SIMD_DESIGN.md || echo 0)
[ "$SECTIONS" -ge 10 ] || { echo "❌ FAIL: Only $SECTIONS sections (need ≥10)"; exit 1; }

echo "✅ ALL ACCEPTANCE CRITERIA PASSED"
```

---

## FAILURE PROTOCOL

### Detection

If architecture design cannot be completed:
- Symptom: Unable to decide between options after analysis
- Evidence: Stuck on decision for >30 minutes

### Categorization

1. **Type A: All options seem equally viable/flawed**
   - Action: Create comparison matrix with weighted pros/cons
   - Time limit: 30 minutes
   - Decision method: Choose option with highest score or fewest critical cons
   - Example:
     ```
     | Option | Pros Score | Cons Score | Net Score |
     |:-------|:-----------|:-----------|:----------|
     | A | +5 | -2 | +3 |
     | B | +7 | -5 | +2 |
     | C | +4 | -1 | +3 |
     Choose: A or C (tie) → Pick A (listed first)
     ```

2. **Type B: Performance projections show <50 cycles impossible**
   - Action: Recalculate with actual operation counts from Intel manuals
   - Time limit: 1 hour
   - Tools:
     ```bash
     # Use LLVM-MCA for cycle analysis
     llvm-mca -march=x86-64 -mcpu=haswell <assembly>
     ```
   - Escalation: If recalculation still shows >75 cycles, escalate to PLANNER
   - Options:
     - A. Relax target to <75 cycles (document justification)
     - B. Defer SIMD to Week 9 for more research
     - C. Ship without SIMD (portable only)

3. **Type C: Safety model has unresolvable unsafe**
   - Action: Escalate to PLANNER immediately
   - Issue: "Cannot prove safety invariant X"
   - Options:
     - A. Use only safe portable implementation (no SIMD)
     - B. Document unsafe with "UNPROVEN" tag, extra hostile review
     - C. Defer SIMD until safety provable

### Escalation Triggers

Escalate to PLANNER if:
- [ ] >2 hours without complete SIMD_DESIGN.md
- [ ] Fundamental architectural conflict discovered (e.g., safety vs performance)
- [ ] All design options violate constraints
- [ ] Required hardware features unavailable on target platforms

### Alternative Paths

If architecture design unfeasible:
- Option A: Defer SIMD to Week 9, ship with portable implementation only
- Option B: Simplify to single-platform SIMD (x86_64 AVX2 only)
- Option C: Use external SIMD library (evaluate licensing, dependencies)

Document decision in: `docs/planning/weeks/week8/W8D37_ARCHITECTURE_BLOCKER.md`

---

## HANDOFF

```
META_ARCHITECT → RUST_ENGINEER

Output: docs/architecture/SIMD_DESIGN.md
Status: APPROVED

Next: 02_SIMD_HAMMING_IMPL.md
```

---

**END OF PROMPT**
