#!/bin/bash

PASS=0
FAIL=0

echo "=== WEEK 8 DAY 2 COMPREHENSIVE FIX VERIFICATION ==="
echo ""

# Function to check and report
check_test() {
  local name=$1
  local cmd=$2
  local expected=$3
  
  result=$(eval "$cmd")
  if [ -n "$expected" ]; then
    if [ "$result" = "$expected" ] || echo "$result" | grep -q "$expected"; then
      echo "✅ $name"
      ((PASS++))
    else
      echo "❌ $name (got: $result, expected: $expected)"
      ((FAIL++))
    fi
  else
    if [ $? -eq 0 ]; then
      echo "✅ $name"
      ((PASS++))
    else
      echo "❌ $name"
      ((FAIL++))
    fi
  fi
}

echo "=== CRITICAL ISSUES ==="

# C1: Binary criteria in 01
check_test "[C1] Binary criteria in 01_SIMD_ARCHITECTURE.md" \
  "grep -c 'Verify:\|Expected:' 01_SIMD_ARCHITECTURE.md" \
  "^[0-9][0-9]"

# C2: Time reconciliation in 00
check_test "[C2] Time reconciliation in 00_MASTER_DISPATCH.md" \
  "grep -q 'Reconciliation:' 00_MASTER_DISPATCH.md && echo 'present'"

# C3-C5: No old references in 00
check_test "[C3-C5] No old file references in 00_MASTER_DISPATCH.md" \
  "! grep -q '02_SIMD_HAMMING_IMPL\|03_SIMD_QUANTIZE_IMPL\|06_HOSTILE_REVIEW' 00_MASTER_DISPATCH.md && echo 'clean'"

# C6: Files in correct sequence
check_test "[C6] Correct file sequence" \
  "ls -1 | grep -E '^0[0-8]' | wc -l" \
  "^8$"

# C7: Binary criteria in 02
check_test "[C7] Binary criteria in 02_SIMD_TEST_SPEC.md" \
  "grep -c 'Expected:\|Forbidden:' 02_SIMD_TEST_SPEC.md" \
  "^[0-9]"

# C8: CLAMP in 04
check_test "[C8] ANTI-HALLUCINATION CLAMP in 04_SIMD_HAMMING_IMPL.md" \
  "grep -q 'ANTI-HALLUCINATION CLAMPS' 04_SIMD_HAMMING_IMPL.md && echo 'present'"

# C9: CLAMP in 05
check_test "[C9] ANTI-HALLUCINATION CLAMP in 05_SIMD_QUANTIZE_IMPL.md" \
  "grep -q 'ANTI-HALLUCINATION CLAMPS' 05_SIMD_QUANTIZE_IMPL.md && echo 'present'"

# C10: File 06 exists
check_test "[C10] File 06_SIMD_VALIDATION.md exists" \
  "test -f 06_SIMD_VALIDATION.md && echo 'exists'"

# C11: Binary criteria in 06
check_test "[C11] Binary criteria in 06_SIMD_VALIDATION.md" \
  "grep -c 'Verify:\|Expected:' 06_SIMD_VALIDATION.md" \
  "^[0-9][0-9]"

echo ""
echo "=== MAJOR ISSUES ==="

# M1: Test matrix in 02
check_test "[M1] CROSS-PLATFORM TEST MATRIX in 02" \
  "grep -q 'CROSS-PLATFORM TEST MATRIX' 02_SIMD_TEST_SPEC.md && echo 'present'"

# M2-M5: Statistical validation in 03
check_test "[M2-M5] STATISTICAL VALIDATION REQUIREMENTS in 03" \
  "grep -q 'STATISTICAL VALIDATION REQUIREMENTS' 03_SIMD_BENCHMARK_SPEC.md && echo 'present'"

# M2-M5: Statistical content
check_test "[M2-M5] Statistical content (Mean/Median/Std Dev)" \
  "grep -c 'Mean:\|Median:\|Std Dev:' 03_SIMD_BENCHMARK_SPEC.md" \
  "^[0-9]"

# M6-M8: CLAMP sections in all files
echo -n "[M6-M8] ANTI-HALLUCINATION CLAMPs in all files: "
clamp_count=$(grep -c "ANTI-HALLUCINATION CLAMPS" 04_SIMD_HAMMING_IMPL.md 05_SIMD_QUANTIZE_IMPL.md 06_SIMD_VALIDATION.md 2>/dev/null | awk -F: '{s+=$2} END {print s}')
if [ "$clamp_count" -ge 3 ]; then
  echo "✅ ($clamp_count sections found)"
  ((PASS++))
else
  echo "❌ (only $clamp_count sections found, need 3)"
  ((FAIL++))
fi

# M9-M11: Dependency verification
echo -n "[M9-M11] DEPENDENCY VERIFICATION sections: "
dep_count=$(grep -c "DEPENDENCY VERIFICATION" 04_SIMD_HAMMING_IMPL.md 05_SIMD_QUANTIZE_IMPL.md 2>/dev/null | awk -F: '{s+=$2} END {print s}')
if [ "$dep_count" -eq 2 ]; then
  echo "✅ (found in both files)"
  ((PASS++))
else
  echo "❌ (found in $dep_count files, need 2)"
  ((FAIL++))
fi

echo ""
echo "=== SUMMARY ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
  echo "✅ ALL FIXES VERIFIED - PRODUCTION READY"
  exit 0
else
  echo "❌ SOME FIXES INCOMPLETE - REMEDIATION NEEDED"
  exit 1
fi
