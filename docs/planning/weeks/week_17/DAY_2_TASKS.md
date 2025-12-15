# Day 2 Tasks — W17.2: TypeScript Types + Integration Tests

**Date:** Week 17, Day 2
**Task ID:** W17.2
**Agent:** TEST_ENGINEER
**Estimate:** 6h (2h base × 3x)
**Priority:** P0
**Status:** PENDING

---

## Objective

Create comprehensive integration tests for WASM soft delete functionality, covering Node.js environment and edge cases. Ensure TypeScript types are complete and accurate.

---

## Prerequisites

- [ ] W17.1 complete (WASM bindings implemented)
- [x] Node.js test environment configured
- [x] Jest or similar test framework available
- [x] TypeScript compilation working

---

## Implementation Checklist

### 1. Create Test File Structure

```
wasm/
├── tests/
│   ├── soft_delete.test.ts    # NEW
│   ├── compaction.test.ts     # NEW
│   └── integration.test.ts    # Existing
├── package.json
└── tsconfig.json
```

### 2. Soft Delete Tests (`wasm/tests/soft_delete.test.ts`)

```typescript
import { WasmIndex } from '../pkg/edgevec';

describe('Soft Delete', () => {
    let index: WasmIndex;

    beforeEach(() => {
        index = new WasmIndex(128, 16, 200);
    });

    afterEach(() => {
        index.free();
    });

    test('soft_delete marks vector as deleted', () => {
        const vector = new Float32Array(128).fill(1.0);
        const id = index.insert(vector);

        expect(index.isDeleted(id)).toBe(false);
        expect(index.softDelete(id)).toBe(true);
        expect(index.isDeleted(id)).toBe(true);
    });

    test('soft_delete returns false for already deleted', () => {
        const vector = new Float32Array(128).fill(1.0);
        const id = index.insert(vector);

        expect(index.softDelete(id)).toBe(true);
        expect(index.softDelete(id)).toBe(false); // Already deleted
    });

    test('soft_delete throws for non-existent ID', () => {
        expect(() => index.softDelete(BigInt(999))).toThrow();
    });

    test('is_deleted throws for non-existent ID', () => {
        expect(() => index.isDeleted(BigInt(999))).toThrow();
    });

    test('deleted_count increments correctly', () => {
        const v1 = index.insert(new Float32Array(128).fill(0.1));
        const v2 = index.insert(new Float32Array(128).fill(0.2));
        const v3 = index.insert(new Float32Array(128).fill(0.3));

        expect(index.deletedCount()).toBe(0);
        index.softDelete(v1);
        expect(index.deletedCount()).toBe(1);
        index.softDelete(v2);
        expect(index.deletedCount()).toBe(2);
        // v3 not deleted
        expect(index.deletedCount()).toBe(2);
    });

    test('live_count decrements correctly', () => {
        const v1 = index.insert(new Float32Array(128).fill(0.1));
        const v2 = index.insert(new Float32Array(128).fill(0.2));

        expect(index.liveCount()).toBe(2);
        index.softDelete(v1);
        expect(index.liveCount()).toBe(1);
    });

    test('tombstone_ratio calculates correctly', () => {
        for (let i = 0; i < 10; i++) {
            index.insert(new Float32Array(128).fill(i / 10));
        }

        expect(index.tombstoneRatio()).toBeCloseTo(0.0);

        index.softDelete(BigInt(0));
        expect(index.tombstoneRatio()).toBeCloseTo(0.1);

        index.softDelete(BigInt(1));
        index.softDelete(BigInt(2));
        expect(index.tombstoneRatio()).toBeCloseTo(0.3);
    });

    test('search excludes deleted vectors', () => {
        const v1 = index.insert(new Float32Array(128).fill(1.0));
        const v2 = index.insert(new Float32Array(128).fill(0.9));
        const v3 = index.insert(new Float32Array(128).fill(0.8));

        index.softDelete(v1);

        const query = new Float32Array(128).fill(1.0);
        const results = index.search(query, 10);

        // v1 should NOT be in results
        expect(results.find(r => r.vectorId === v1)).toBeUndefined();
        // v2 and v3 should be in results
        expect(results.find(r => r.vectorId === v2)).toBeDefined();
    });
});
```

### 3. Compaction Tests (`wasm/tests/compaction.test.ts`)

```typescript
import { WasmIndex } from '../pkg/edgevec';

describe('Compaction', () => {
    let index: WasmIndex;

    beforeEach(() => {
        index = new WasmIndex(128, 16, 200);
    });

    afterEach(() => {
        index.free();
    });

    test('needs_compaction returns false initially', () => {
        for (let i = 0; i < 10; i++) {
            index.insert(new Float32Array(128).fill(i / 10));
        }
        expect(index.needsCompaction()).toBe(false);
    });

    test('needs_compaction returns true above threshold', () => {
        for (let i = 0; i < 10; i++) {
            index.insert(new Float32Array(128).fill(i / 10));
        }

        // Delete 4 out of 10 (40% > default 30% threshold)
        for (let i = 0; i < 4; i++) {
            index.softDelete(BigInt(i));
        }

        expect(index.needsCompaction()).toBe(true);
    });

    test('compaction_warning returns message above threshold', () => {
        for (let i = 0; i < 10; i++) {
            index.insert(new Float32Array(128).fill(i / 10));
        }

        expect(index.compactionWarning()).toBeNull();

        for (let i = 0; i < 4; i++) {
            index.softDelete(BigInt(i));
        }

        const warning = index.compactionWarning();
        expect(warning).not.toBeNull();
        expect(warning).toContain('Compaction recommended');
    });

    test('compact removes all tombstones', () => {
        for (let i = 0; i < 100; i++) {
            index.insert(new Float32Array(128).fill(i / 100));
        }

        for (let i = 0; i < 30; i++) {
            index.softDelete(BigInt(i));
        }

        expect(index.deletedCount()).toBe(30);
        expect(index.liveCount()).toBe(70);

        const result = index.compact();

        expect(result.tombstones_removed).toBe(30);
        expect(result.new_size).toBe(70);
        expect(index.deletedCount()).toBe(0);
        expect(index.liveCount()).toBe(70);
        expect(result.duration_ms).toBeGreaterThanOrEqual(0);
    });

    test('compact preserves search quality', () => {
        // Insert vectors
        const vectors: Float32Array[] = [];
        for (let i = 0; i < 100; i++) {
            const v = new Float32Array(128).fill(i / 100);
            vectors.push(v);
            index.insert(v);
        }

        // Delete some
        for (let i = 0; i < 30; i++) {
            index.softDelete(BigInt(i));
        }

        // Search before compact
        const query = vectors[50];
        const resultsBefore = index.search(query, 5);

        // Compact
        index.compact();

        // Search after compact
        const resultsAfter = index.search(query, 5);

        // Results should be similar (same top result)
        expect(resultsAfter[0].vectorId).toBe(resultsBefore[0].vectorId);
    });

    test('set_compaction_threshold changes threshold', () => {
        for (let i = 0; i < 10; i++) {
            index.insert(new Float32Array(128).fill(i / 10));
        }

        // Delete 2 out of 10 (20%)
        index.softDelete(BigInt(0));
        index.softDelete(BigInt(1));

        // Default threshold is 30%, so should NOT need compaction
        expect(index.needsCompaction()).toBe(false);

        // Lower threshold to 15%
        index.setCompactionThreshold(0.15);

        // Now 20% > 15%, so SHOULD need compaction
        expect(index.needsCompaction()).toBe(true);
    });
});
```

### 4. Persistence Tests (add to existing)

```typescript
describe('Persistence with Soft Delete', () => {
    test('save and load preserves deleted state', () => {
        const index = new WasmIndex(128, 16, 200);

        const v1 = index.insert(new Float32Array(128).fill(0.1));
        const v2 = index.insert(new Float32Array(128).fill(0.2));

        index.softDelete(v1);

        const data = index.save();
        index.free();

        const loaded = WasmIndex.load(data);

        expect(loaded.isDeleted(v1)).toBe(true);
        expect(loaded.isDeleted(v2)).toBe(false);
        expect(loaded.deletedCount()).toBe(1);

        loaded.free();
    });

    test('save and load preserves deleted_count', () => {
        const index = new WasmIndex(128, 16, 200);

        for (let i = 0; i < 10; i++) {
            index.insert(new Float32Array(128).fill(i / 10));
        }

        for (let i = 0; i < 3; i++) {
            index.softDelete(BigInt(i));
        }

        const data = index.save();
        index.free();

        const loaded = WasmIndex.load(data);

        expect(loaded.deletedCount()).toBe(3);
        expect(loaded.liveCount()).toBe(7);

        loaded.free();
    });
});
```

### 5. Update `package.json`

```json
{
  "scripts": {
    "test": "jest",
    "test:coverage": "jest --coverage"
  },
  "devDependencies": {
    "@types/jest": "^29.0.0",
    "jest": "^29.0.0",
    "ts-jest": "^29.0.0",
    "typescript": "^5.0.0"
  }
}
```

---

## Acceptance Criteria Verification

| AC | Verification | Expected |
|:---|:-------------|:---------|
| AC17.2.1 | `tsc --noEmit` | No errors |
| AC17.2.2 | `npm test soft_delete.test.ts` | PASS |
| AC17.2.3 | `npm test soft_delete.test.ts` | PASS |
| AC17.2.4 | `npm test compaction.test.ts` | PASS |
| AC17.2.5 | `npm test persistence` | PASS |
| AC17.2.6 | `npm test compaction.test.ts` | PASS |
| AC17.2.7 | `npm test` | All PASS |
| AC17.2.8 | `npm test:coverage` | > 90% |

---

## Output

### Artifacts Generated

- [ ] `wasm/tests/soft_delete.test.ts` — Soft delete tests
- [ ] `wasm/tests/compaction.test.ts` — Compaction tests
- [ ] `wasm/package.json` — Updated with test deps

### Status After Completion

```
✅ W17.2 COMPLETE
Next: W17.3 (Example App)
```

---

**Status:** PENDING
**Next:** `/test-prop soft_delete_wasm`
