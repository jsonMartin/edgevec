// wasm/__tests__/soft_delete.test.ts
// W17.2 — Soft Delete Integration Tests

import { EdgeVecClient } from '../EdgeVecClient';

describe('Soft Delete API', () => {
  describe('softDelete', () => {
    it('should mark vector as deleted', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      const vector = new Float32Array([1, 2, 3, 4]);
      const id = client.insert(vector);

      expect(client.isDeleted(id)).toBe(false);
      const wasDeleted = client.softDelete(id);
      expect(wasDeleted).toBe(true);
      expect(client.isDeleted(id)).toBe(true);
    });

    it('should return false for already deleted vector (idempotent)', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      const id = client.insert(new Float32Array([1, 2, 3, 4]));

      expect(client.softDelete(id)).toBe(true);
      expect(client.softDelete(id)).toBe(false); // Already deleted
    });

    it('should throw for non-existent ID', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      expect(() => client.softDelete(999)).toThrow();
    });
  });

  describe('isDeleted', () => {
    it('should return false for live vector', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      const id = client.insert(new Float32Array([1, 2, 3, 4]));
      expect(client.isDeleted(id)).toBe(false);
    });

    it('should return true for deleted vector', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      const id = client.insert(new Float32Array([1, 2, 3, 4]));
      client.softDelete(id);
      expect(client.isDeleted(id)).toBe(true);
    });

    it('should throw for non-existent ID', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      expect(() => client.isDeleted(999)).toThrow();
    });
  });

  describe('deletedCount', () => {
    it('should start at 0', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      expect(client.deletedCount).toBe(0);
    });

    it('should increment on each delete', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      const id1 = client.insert(new Float32Array([1, 0, 0, 0]));
      const id2 = client.insert(new Float32Array([0, 1, 0, 0]));
      const id3 = client.insert(new Float32Array([0, 0, 1, 0]));

      expect(client.deletedCount).toBe(0);

      client.softDelete(id1);
      expect(client.deletedCount).toBe(1);

      client.softDelete(id2);
      expect(client.deletedCount).toBe(2);

      // id3 not deleted
      expect(client.deletedCount).toBe(2);
    });

    it('should not increment for duplicate deletes', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      const id = client.insert(new Float32Array([1, 2, 3, 4]));

      client.softDelete(id);
      expect(client.deletedCount).toBe(1);

      client.softDelete(id); // Duplicate
      expect(client.deletedCount).toBe(1);
    });
  });

  describe('liveCount', () => {
    it('should return total vectors when none deleted', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      client.insert(new Float32Array([1, 0, 0, 0]));
      client.insert(new Float32Array([0, 1, 0, 0]));

      expect(client.liveCount).toBe(2);
    });

    it('should decrement on delete', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      const id1 = client.insert(new Float32Array([1, 0, 0, 0]));
      client.insert(new Float32Array([0, 1, 0, 0]));

      expect(client.liveCount).toBe(2);

      client.softDelete(id1);
      expect(client.liveCount).toBe(1);
    });
  });

  describe('tombstoneRatio', () => {
    it('should be 0.0 when no deletions', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      expect(client.tombstoneRatio).toBeCloseTo(0.0);
    });

    it('should calculate ratio correctly', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 3 out of 10 (30%)
      client.softDelete(0);
      client.softDelete(1);
      client.softDelete(2);

      expect(client.tombstoneRatio).toBeCloseTo(0.3);
    });

    it('should be 1.0 when all deleted', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      const ids: number[] = [];
      for (let i = 0; i < 5; i++) {
        ids.push(client.insert(new Float32Array([i / 5, 0, 0, 0])));
      }

      for (const id of ids) {
        client.softDelete(id);
      }

      expect(client.tombstoneRatio).toBeCloseTo(1.0);
    });
  });

  describe('search excludes deleted vectors', () => {
    it('should not return deleted vectors in search results', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      // Insert test vectors
      const id1 = client.insert(new Float32Array([1, 0, 0, 0]));  // Exact match to query
      const id2 = client.insert(new Float32Array([0.9, 0.1, 0, 0]));  // Close
      const id3 = client.insert(new Float32Array([0.8, 0.2, 0, 0]));  // Further

      // Delete the exact match
      client.softDelete(id1);

      // Search for the deleted vector's pattern
      const query = new Float32Array([1, 0, 0, 0]);
      const results = client.search(query, 10);

      // id1 should NOT be in results
      const ids = results.map(r => r.id);
      expect(ids).not.toContain(id1);

      // id2 and id3 should be in results
      expect(ids).toContain(id2);
      expect(ids).toContain(id3);
    });

    it('should return empty results when all vectors deleted', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      const id1 = client.insert(new Float32Array([1, 0, 0, 0]));
      const id2 = client.insert(new Float32Array([0, 1, 0, 0]));

      client.softDelete(id1);
      client.softDelete(id2);

      const results = client.search(new Float32Array([1, 0, 0, 0]), 10);
      expect(results).toHaveLength(0);
    });
  });

  describe('getSoftDeleteStats', () => {
    it('should return all stats in one object', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      // Insert 10 vectors
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 3
      client.softDelete(0);
      client.softDelete(1);
      client.softDelete(2);

      const stats = client.getSoftDeleteStats();

      expect(stats.deletedCount).toBe(3);
      expect(stats.liveCount).toBe(7);
      expect(stats.tombstoneRatio).toBeCloseTo(0.3);
      expect(stats.needsCompaction).toBe(false); // Exactly at threshold
    });
  });
});
