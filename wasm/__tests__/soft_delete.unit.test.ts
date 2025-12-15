// wasm/__tests__/soft_delete.unit.test.ts
// W17.2 — Soft Delete Unit Tests (Mocked WASM)

import {
  MockWasmEdgeVec,
  MockWasmConfig,
  mockInit
} from './__mocks__/wasm.mock';

describe('Soft Delete API (Unit Tests)', () => {
  describe('softDelete', () => {
    it('should mark vector as deleted', () => {
      const wasm = new MockWasmEdgeVec();
      const vector = new Float32Array([1, 2, 3, 4]);
      const id = wasm.insert(vector);

      expect(wasm.isDeleted(id)).toBe(false);
      const wasDeleted = wasm.softDelete(id);
      expect(wasDeleted).toBe(true);
      expect(wasm.isDeleted(id)).toBe(true);
    });

    it('should return false for already deleted vector (idempotent)', () => {
      const wasm = new MockWasmEdgeVec();
      const id = wasm.insert(new Float32Array([1, 2, 3, 4]));

      expect(wasm.softDelete(id)).toBe(true);
      expect(wasm.softDelete(id)).toBe(false); // Already deleted
    });

    it('should throw for non-existent ID', () => {
      const wasm = new MockWasmEdgeVec();
      expect(() => wasm.softDelete(999)).toThrow();
    });
  });

  describe('isDeleted', () => {
    it('should return false for live vector', () => {
      const wasm = new MockWasmEdgeVec();
      const id = wasm.insert(new Float32Array([1, 2, 3, 4]));
      expect(wasm.isDeleted(id)).toBe(false);
    });

    it('should return true for deleted vector', () => {
      const wasm = new MockWasmEdgeVec();
      const id = wasm.insert(new Float32Array([1, 2, 3, 4]));
      wasm.softDelete(id);
      expect(wasm.isDeleted(id)).toBe(true);
    });

    it('should throw for non-existent ID', () => {
      const wasm = new MockWasmEdgeVec();
      expect(() => wasm.isDeleted(999)).toThrow();
    });
  });

  describe('deletedCount', () => {
    it('should start at 0', () => {
      const wasm = new MockWasmEdgeVec();
      expect(wasm.deletedCount()).toBe(0);
    });

    it('should increment on each delete', () => {
      const wasm = new MockWasmEdgeVec();
      const id1 = wasm.insert(new Float32Array([1, 0, 0, 0]));
      const id2 = wasm.insert(new Float32Array([0, 1, 0, 0]));
      wasm.insert(new Float32Array([0, 0, 1, 0])); // id3

      expect(wasm.deletedCount()).toBe(0);

      wasm.softDelete(id1);
      expect(wasm.deletedCount()).toBe(1);

      wasm.softDelete(id2);
      expect(wasm.deletedCount()).toBe(2);

      // id3 not deleted
      expect(wasm.deletedCount()).toBe(2);
    });

    it('should not increment for duplicate deletes', () => {
      const wasm = new MockWasmEdgeVec();
      const id = wasm.insert(new Float32Array([1, 2, 3, 4]));

      wasm.softDelete(id);
      expect(wasm.deletedCount()).toBe(1);

      wasm.softDelete(id); // Duplicate
      expect(wasm.deletedCount()).toBe(1);
    });
  });

  describe('liveCount', () => {
    it('should return total vectors when none deleted', () => {
      const wasm = new MockWasmEdgeVec();
      wasm.insert(new Float32Array([1, 0, 0, 0]));
      wasm.insert(new Float32Array([0, 1, 0, 0]));

      expect(wasm.liveCount()).toBe(2);
    });

    it('should decrement on delete', () => {
      const wasm = new MockWasmEdgeVec();
      const id1 = wasm.insert(new Float32Array([1, 0, 0, 0]));
      wasm.insert(new Float32Array([0, 1, 0, 0]));

      expect(wasm.liveCount()).toBe(2);

      wasm.softDelete(id1);
      expect(wasm.liveCount()).toBe(1);
    });
  });

  describe('tombstoneRatio', () => {
    it('should be 0.0 when no deletions', () => {
      const wasm = new MockWasmEdgeVec();
      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      expect(wasm.tombstoneRatio()).toBeCloseTo(0.0);
    });

    it('should calculate ratio correctly', () => {
      const wasm = new MockWasmEdgeVec();
      for (let i = 0; i < 10; i++) {
        wasm.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 3 out of 10 (30%)
      wasm.softDelete(0);
      wasm.softDelete(1);
      wasm.softDelete(2);

      expect(wasm.tombstoneRatio()).toBeCloseTo(0.3);
    });

    it('should be 1.0 when all deleted', () => {
      const wasm = new MockWasmEdgeVec();
      const ids: number[] = [];
      for (let i = 0; i < 5; i++) {
        ids.push(wasm.insert(new Float32Array([i / 5, 0, 0, 0])));
      }

      for (const id of ids) {
        wasm.softDelete(id);
      }

      expect(wasm.tombstoneRatio()).toBeCloseTo(1.0);
    });
  });

  describe('search excludes deleted vectors', () => {
    it('should not return deleted vectors in search results', () => {
      const wasm = new MockWasmEdgeVec();

      // Insert test vectors
      const id1 = wasm.insert(new Float32Array([1, 0, 0, 0]));  // Exact match to query
      const id2 = wasm.insert(new Float32Array([0.9, 0.1, 0, 0]));  // Close
      const id3 = wasm.insert(new Float32Array([0.8, 0.2, 0, 0]));  // Further

      // Delete the exact match
      wasm.softDelete(id1);

      // Search for the deleted vector's pattern
      const query = new Float32Array([1, 0, 0, 0]);
      const results = wasm.search(query, 10);

      // id1 should NOT be in results
      const ids = results.map(r => r.id);
      expect(ids).not.toContain(id1);

      // id2 and id3 should be in results
      expect(ids).toContain(id2);
      expect(ids).toContain(id3);
    });

    it('should return empty results when all vectors deleted', () => {
      const wasm = new MockWasmEdgeVec();

      const id1 = wasm.insert(new Float32Array([1, 0, 0, 0]));
      const id2 = wasm.insert(new Float32Array([0, 1, 0, 0]));

      wasm.softDelete(id1);
      wasm.softDelete(id2);

      const results = wasm.search(new Float32Array([1, 0, 0, 0]), 10);
      expect(results).toHaveLength(0);
    });
  });
});
