// wasm/__tests__/compaction.unit.test.ts
// W17.2 — Compaction Unit Tests (Mocked WASM)

import { MockWasmEdgeVec } from './__mocks__/wasm.mock';

describe('Compaction API (Unit Tests)', () => {
  describe('needsCompaction', () => {
    it('should return false initially', () => {
      const wasm = new MockWasmEdgeVec();
      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }
      expect(wasm.needsCompaction()).toBe(false);
    });

    it('should return true when ratio exceeds threshold', () => {
      const wasm = new MockWasmEdgeVec();
      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 4 out of 10 (40% > default 30% threshold)
      wasm.softDelete(0);
      wasm.softDelete(1);
      wasm.softDelete(2);
      wasm.softDelete(3);

      expect(wasm.needsCompaction()).toBe(true);
    });

    it('should return false when ratio equals threshold', () => {
      const wasm = new MockWasmEdgeVec();
      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 3 out of 10 (exactly 30%)
      wasm.softDelete(0);
      wasm.softDelete(1);
      wasm.softDelete(2);

      // Default threshold is 0.3, so 0.3 <= 0.3 should be false
      expect(wasm.needsCompaction()).toBe(false);
    });
  });

  describe('compactionThreshold', () => {
    it('should have default threshold of 0.3', () => {
      const wasm = new MockWasmEdgeVec();
      expect(wasm.compactionThreshold()).toBeCloseTo(0.3);
    });

    it('should allow setting threshold', () => {
      const wasm = new MockWasmEdgeVec();
      wasm.setCompactionThreshold(0.5);
      expect(wasm.compactionThreshold()).toBeCloseTo(0.5);
    });

    it('should clamp threshold to valid range', () => {
      const wasm = new MockWasmEdgeVec();

      // Setting very low should clamp to minimum
      wasm.setCompactionThreshold(0.001);
      expect(wasm.compactionThreshold()).toBeGreaterThanOrEqual(0.01);

      // Setting very high should clamp to maximum
      wasm.setCompactionThreshold(0.999);
      expect(wasm.compactionThreshold()).toBeLessThanOrEqual(0.99);
    });

    it('should affect needsCompaction behavior', () => {
      const wasm = new MockWasmEdgeVec();
      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 2 out of 10 (20%)
      wasm.softDelete(0);
      wasm.softDelete(1);

      // Default threshold is 30%, so should NOT need compaction
      expect(wasm.needsCompaction()).toBe(false);

      // Lower threshold to 15%
      wasm.setCompactionThreshold(0.15);

      // Now 20% > 15%, so SHOULD need compaction
      expect(wasm.needsCompaction()).toBe(true);
    });
  });

  describe('compactionWarning', () => {
    it('should return null when compaction not needed', () => {
      const wasm = new MockWasmEdgeVec();
      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      expect(wasm.compactionWarning()).toBeNull();
    });

    it('should return warning message when compaction needed', () => {
      const wasm = new MockWasmEdgeVec();
      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 4 out of 10 (40%)
      wasm.softDelete(0);
      wasm.softDelete(1);
      wasm.softDelete(2);
      wasm.softDelete(3);

      const warning = wasm.compactionWarning();
      expect(warning).not.toBeNull();
      expect(warning).toContain('Compaction recommended');
    });
  });

  describe('compact', () => {
    it('should remove all tombstones', () => {
      const wasm = new MockWasmEdgeVec();

      // Insert 20 vectors
      for (let i = 0; i < 20; i++) {
        wasm.insert(new Float32Array([i / 20, 0, 0, 0]));
      }

      // Delete 10 vectors
      for (let i = 0; i < 10; i++) {
        wasm.softDelete(i);
      }

      expect(wasm.deletedCount()).toBe(10);
      expect(wasm.liveCount()).toBe(10);

      const result = wasm.compact();

      expect(result.tombstones_removed).toBe(10);
      expect(result.new_size).toBe(10);
      expect(wasm.deletedCount()).toBe(0);
      expect(wasm.liveCount()).toBe(10);
    });

    it('should return valid CompactionResult', () => {
      const wasm = new MockWasmEdgeVec();

      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete some
      wasm.softDelete(0);
      wasm.softDelete(1);
      wasm.softDelete(2);

      const result = wasm.compact();

      expect(result.tombstones_removed).toBe(3);
      expect(result.new_size).toBe(7);
      expect(result.duration_ms).toBeGreaterThanOrEqual(0);
    });

    it('should reset needsCompaction to false', () => {
      const wasm = new MockWasmEdgeVec();

      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 5 out of 10 (50%)
      for (let i = 0; i < 5; i++) {
        wasm.softDelete(i);
      }

      expect(wasm.needsCompaction()).toBe(true);

      wasm.compact();

      expect(wasm.needsCompaction()).toBe(false);
      expect(wasm.tombstoneRatio()).toBeCloseTo(0.0);
    });

    it('should work with empty index', () => {
      const wasm = new MockWasmEdgeVec();

      const result = wasm.compact();

      expect(result.tombstones_removed).toBe(0);
      expect(result.new_size).toBe(0);
    });

    it('should work when no tombstones', () => {
      const wasm = new MockWasmEdgeVec();

      for (let i = 0; i < 5; i++) {
        wasm.insert(new Float32Array([i / 5, 0, 0, 0]));
      }

      const result = wasm.compact();

      expect(result.tombstones_removed).toBe(0);
      expect(result.new_size).toBe(5);
    });
  });

  describe('compaction with large datasets', () => {
    it('should handle 100 vectors with 30% deletion', () => {
      const wasm = new MockWasmEdgeVec();

      // Insert 100 vectors
      for (let i = 0; i < 100; i++) {
        wasm.insert(new Float32Array([
          Math.sin(i * 0.1),
          Math.cos(i * 0.1),
          Math.sin(i * 0.2),
          Math.cos(i * 0.2)
        ]));
      }

      // Delete 30 vectors (30%)
      for (let i = 0; i < 30; i++) {
        wasm.softDelete(i);
      }

      expect(wasm.deletedCount()).toBe(30);
      expect(wasm.liveCount()).toBe(70);

      const result = wasm.compact();

      expect(result.tombstones_removed).toBe(30);
      expect(result.new_size).toBe(70);
      expect(wasm.deletedCount()).toBe(0);
      expect(wasm.liveCount()).toBe(70);
    });
  });

  describe('edge cases', () => {
    it('should handle delete-all-then-compact', () => {
      const wasm = new MockWasmEdgeVec();

      const ids: number[] = [];
      for (let i = 0; i < 5; i++) {
        ids.push(wasm.insert(new Float32Array([i / 5, 0, 0, 0])));
      }

      // Delete all
      for (const id of ids) {
        wasm.softDelete(id);
      }

      expect(wasm.tombstoneRatio()).toBeCloseTo(1.0);

      const result = wasm.compact();

      expect(result.tombstones_removed).toBe(5);
      expect(result.new_size).toBe(0);
      expect(wasm.deletedCount()).toBe(0);
      expect(wasm.liveCount()).toBe(0);
    });

    it('should handle multiple compact calls', () => {
      const wasm = new MockWasmEdgeVec();

      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      wasm.softDelete(0);
      wasm.softDelete(1);

      // First compact
      const result1 = wasm.compact();
      expect(result1.tombstones_removed).toBe(2);

      // Second compact (no tombstones)
      const result2 = wasm.compact();
      expect(result2.tombstones_removed).toBe(0);
      expect(result2.new_size).toBe(8);
    });
  });
});
