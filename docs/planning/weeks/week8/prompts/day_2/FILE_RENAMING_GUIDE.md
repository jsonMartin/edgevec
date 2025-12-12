# FILE RENAMING GUIDE — Day 2 Test-First Order Fix

**Date:** 2025-12-12
**Status:** CRITICAL — Required for test-first compliance
**Executor:** Manual (requires file system operations)

---

## CRITICAL ISSUE

Current file numbering violates test-first methodology:
```
CURRENT (WRONG):
02_SIMD_HAMMING_IMPL.md ← CODE BEFORE TESTS ❌
03_SIMD_QUANTIZE_IMPL.md ← CODE BEFORE TESTS ❌
05_SIMD_TESTS.md ← TESTS AFTER CODE ❌
```

**This MUST be fixed before executing Day 2.**

---

## REQUIRED FILE RENAMING

### Step 1: Rename Existing Files

Execute these commands in `docs/planning/weeks/week8/prompts/day_2/`:

```bash
cd docs/planning/weeks/week8/prompts/day_2/

# Rename implementation prompts (move to higher numbers)
mv 02_SIMD_HAMMING_IMPL.md 04_SIMD_HAMMING_IMPL.md
mv 03_SIMD_QUANTIZE_IMPL.md 05_SIMD_QUANTIZE_IMPL.md

# Rename review prompts (increment by 1)
mv 06_HOSTILE_REVIEW.md 07_HOSTILE_REVIEW.md
mv 07_NVIDIA_GRADE_HOSTILE_REVIEW.md 08_NVIDIA_GRADE_HOSTILE_REVIEW.md

# Note: 02 and 03 are now NEW files (already created)
# Note: 04_SIMD_BENCHMARKS.md will be merged into 06_SIMD_VALIDATION.md (later)
# Note: 05_SIMD_TESTS.md content split into 02_TEST_SPEC (done) + 06_VALIDATION (later)
```

### Step 2: Verify New Structure

After renaming, directory should contain:

```
docs/planning/weeks/week8/prompts/day_2/
├── 00_MASTER_DISPATCH.md (updated time estimates ✅)
├── 01_SIMD_ARCHITECTURE.md (updated criteria + protocol ✅)
├── 02_SIMD_TEST_SPEC.md (NEW ✅)
├── 03_SIMD_BENCHMARK_SPEC.md (NEW ✅)
├── 04_SIMD_HAMMING_IMPL.md (RENAMED from 02 ⚠️)
├── 05_SIMD_QUANTIZE_IMPL.md (RENAMED from 03 ⚠️)
├── 06_SIMD_VALIDATION.md (TODO - merge from 04+05 old)
├── 07_HOSTILE_REVIEW.md (RENAMED from 06 ⚠️)
├── 08_NVIDIA_GRADE_HOSTILE_REVIEW.md (RENAMED from 07 ⚠️)
├── META_CORRECTION_TEST_FIRST.md (unchanged)
├── PLANNER_DAY2_OPTIMIZATION.md (unchanged)
├── PLANNER_FIX_PROMPT.md (unchanged)
├── PROMPT_MAKER_META_REVIEW.md (unchanged)
└── FILE_RENAMING_GUIDE.md (this file)
```

---

## VERIFICATION COMMANDS

After renaming, run these checks:

```bash
cd docs/planning/weeks/week8/prompts/day_2/

# Check test-first order
echo "=== File Order Check ==="
ls -1 | grep -E "^0[0-8]" | head -9

# Expected output:
# 00_MASTER_DISPATCH.md
# 01_SIMD_ARCHITECTURE.md
# 02_SIMD_TEST_SPEC.md ← TESTS FIRST ✅
# 03_SIMD_BENCHMARK_SPEC.md ← TARGETS FIRST ✅
# 04_SIMD_HAMMING_IMPL.md ← CODE AFTER TESTS ✅
# 05_SIMD_QUANTIZE_IMPL.md
# 06_SIMD_VALIDATION.md
# 07_HOSTILE_REVIEW.md
# 08_NVIDIA_GRADE_HOSTILE_REVIEW.md

# Check new files exist
echo "=== New Files Check ==="
test -f 02_SIMD_TEST_SPEC.md && echo "✅ TEST_SPEC created"
test -f 03_SIMD_BENCHMARK_SPEC.md && echo "✅ BENCHMARK_SPEC created"

# Check old files renamed
echo "=== Renaming Check ==="
test -f 04_SIMD_HAMMING_IMPL.md && echo "✅ HAMMING_IMPL renamed"
test -f 05_SIMD_QUANTIZE_IMPL.md && echo "✅ QUANTIZE_IMPL renamed"
test -f 07_HOSTILE_REVIEW.md && echo "✅ HOSTILE_REVIEW renamed"
test -f 08_NVIDIA_GRADE_HOSTILE_REVIEW.md && echo "✅ NVIDIA_REVIEW renamed"

# Check no old numbered files remain
echo "=== Old File Check (should be empty) ==="
test ! -f 02_SIMD_HAMMING_IMPL.md && echo "✅ Old 02 removed"
test ! -f 03_SIMD_QUANTIZE_IMPL.md && echo "✅ Old 03 removed"
test ! -f 06_HOSTILE_REVIEW.md && echo "✅ Old 06 removed"
test ! -f 07_NVIDIA_GRADE_HOSTILE_REVIEW.md && echo "✅ Old 07 removed"

echo ""
echo "If all checks pass: Test-first order FIXED ✅"
```

---

## UPDATED CROSS-REFERENCES

After renaming, these cross-references in files need updating:

### In 00_MASTER_DISPATCH.md:
- Line 61: `02_SIMD_HAMMING_IMPL.md` → `04_SIMD_HAMMING_IMPL.md`
- Line 68: `03_SIMD_QUANTIZE_IMPL.md` → `05_SIMD_QUANTIZE_IMPL.md`
- Line 94: `06_HOSTILE_REVIEW.md` → `07_HOSTILE_REVIEW.md`

### In 01_SIMD_ARCHITECTURE.md:
- Line 383: `02_SIMD_HAMMING_IMPL.md` → `04_SIMD_HAMMING_IMPL.md`

### In 02_SIMD_TEST_SPEC.md:
- Already references `04_SIMD_HAMMING_IMPL.md` (correct!)

### In 03_SIMD_BENCHMARK_SPEC.md:
- Already references `04_SIMD_HAMMING_IMPL.md` (correct!)

**Note:** Some cross-references were already created correctly in new files!

---

## WHY THIS IS CRITICAL

**Before Fix:**
```
01_ARCHITECTURE → 02_IMPL → 03_IMPL → 05_TESTS
                   ↑ CODE FIRST ❌
```

**After Fix:**
```
01_ARCHITECTURE → 02_TEST_SPEC → 03_BENCHMARK_SPEC → 04_IMPL
                   ↑ TESTS FIRST ✅
```

This enforces:
1. Tests written BEFORE implementation exists
2. Performance targets defined BEFORE optimization
3. Implementation GUIDED BY pre-existing tests

**Violating this order enables hallucinations and untestable code.**

---

## MANUAL STEPS REQUIRED

**Executor:** User or File Manager

1. Navigate to: `docs/planning/weeks/week8/prompts/day_2/`
2. Rename files as specified in Step 1 above
3. Run verification commands
4. (Optional) Update cross-references in files

**Estimated Time:** 5 minutes

---

## AFTER RENAMING

Once files are renamed, update this document:

```markdown
## RENAMING STATUS

- [ ] Files renamed as specified
- [ ] Verification commands run (all ✅)
- [ ] Cross-references updated (optional)
- [ ] Ready for meta-review v2

Updated by: _______________
Date: _______________
```

---

**END OF RENAMING GUIDE**
