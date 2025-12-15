// wasm/__tests__/compaction.test.ts
// W17.2 — Compaction Integration Tests

import { EdgeVecClient } from '../EdgeVecClient';

describe('Compaction API', () => {
  describe('needsCompaction', () => {
    it('should return false initially', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }
      expect(client.needsCompaction).toBe(false);
    });

    it('should return true when ratio exceeds threshold', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 4 out of 10 (40% > default 30% threshold)
      client.softDelete(0);
      client.softDelete(1);
      client.softDelete(2);
      client.softDelete(3);

      expect(client.needsCompaction).toBe(true);
    });

    it('should return false when ratio equals threshold', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 3 out of 10 (exactly 30%)
      client.softDelete(0);
      client.softDelete(1);
      client.softDelete(2);

      // Default threshold is 0.3, so 0.3 <= 0.3 should be false
      expect(client.needsCompaction).toBe(false);
    });
  });

  describe('compactionThreshold', () => {
    it('should have default threshold of 0.3', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      expect(client.compactionThreshold).toBeCloseTo(0.3);
    });

    it('should allow setting threshold', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      client.compactionThreshold = 0.5;
      expect(client.compactionThreshold).toBeCloseTo(0.5);
    });

    it('should clamp threshold to valid range', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      // Setting very low should clamp to minimum
      client.compactionThreshold = 0.001;
      expect(client.compactionThreshold).toBeGreaterThanOrEqual(0.01);

      // Setting very high should clamp to maximum
      client.compactionThreshold = 0.999;
      expect(client.compactionThreshold).toBeLessThanOrEqual(0.99);
    });

    it('should affect needsCompaction behavior', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 2 out of 10 (20%)
      client.softDelete(0);
      client.softDelete(1);

      // Default threshold is 30%, so should NOT need compaction
      expect(client.needsCompaction).toBe(false);

      // Lower threshold to 15%
      client.compactionThreshold = 0.15;

      // Now 20% > 15%, so SHOULD need compaction
      expect(client.needsCompaction).toBe(true);
    });
  });

  describe('compactionWarning', () => {
    it('should return null when compaction not needed', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      expect(client.compactionWarning).toBeNull();
    });

    it('should return warning message when compaction needed', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });
      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 4 out of 10 (40%)
      client.softDelete(0);
      client.softDelete(1);
      client.softDelete(2);
      client.softDelete(3);

      const warning = client.compactionWarning;
      expect(warning).not.toBeNull();
      expect(warning).toContain('Compaction recommended');
    });
  });

  describe('compact', () => {
    it('should remove all tombstones', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      // Insert 20 vectors
      for (let i = 0; i < 20; i++) {
        client.insert(new Float32Array([i / 20, 0, 0, 0]));
      }

      // Delete 10 vectors
      for (let i = 0; i < 10; i++) {
        client.softDelete(i);
      }

      expect(client.deletedCount).toBe(10);
      expect(client.liveCount).toBe(10);

      const result = client.compact();

      expect(result.tombstonesRemoved).toBe(10);
      expect(result.newSize).toBe(10);
      expect(client.deletedCount).toBe(0);
      expect(client.liveCount).toBe(10);
    });

    it('should return valid CompactionResult', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete some
      client.softDelete(0);
      client.softDelete(1);
      client.softDelete(2);

      const result = client.compact();

      expect(result.tombstonesRemoved).toBe(3);
      expect(result.newSize).toBe(7);
      expect(result.durationMs).toBeGreaterThanOrEqual(0);
    });

    it('should reset needsCompaction to false', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      // Delete 5 out of 10 (50%)
      for (let i = 0; i < 5; i++) {
        client.softDelete(i);
      }

      expect(client.needsCompaction).toBe(true);

      client.compact();

      expect(client.needsCompaction).toBe(false);
      expect(client.tombstoneRatio).toBeCloseTo(0.0);
    });

    it('should preserve search quality after compaction', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      // Insert vectors with distinct patterns
      client.insert(new Float32Array([1, 0, 0, 0]));     // id 0 - will delete
      client.insert(new Float32Array([0, 1, 0, 0]));     // id 1 - will delete
      client.insert(new Float32Array([0.5, 0.5, 0, 0])); // id 2 - keep
      client.insert(new Float32Array([0.9, 0.1, 0, 0])); // id 3 - keep, closest to query
      client.insert(new Float32Array([0.1, 0.9, 0, 0])); // id 4 - keep

      // Delete some
      client.softDelete(0);
      client.softDelete(1);

      // Search before compact
      const query = new Float32Array([1, 0, 0, 0]);
      const resultsBefore = client.search(query, 3);

      // Compact
      client.compact();

      // Search after compact
      const resultsAfter = client.search(query, 3);

      // Results should be similar - same top result (id 3 = [0.9, 0.1, 0, 0])
      expect(resultsAfter[0].id).toBe(resultsBefore[0].id);
      expect(resultsAfter.length).toBe(resultsBefore.length);
    });

    it('should work with empty index', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      const result = client.compact();

      expect(result.tombstonesRemoved).toBe(0);
      expect(result.newSize).toBe(0);
    });

    it('should work when no tombstones', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      for (let i = 0; i < 5; i++) {
        client.insert(new Float32Array([i / 5, 0, 0, 0]));
      }

      const result = client.compact();

      expect(result.tombstonesRemoved).toBe(0);
      expect(result.newSize).toBe(5);
    });
  });

  describe('compaction with large datasets', () => {
    it('should handle 100 vectors with 30% deletion', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      // Insert 100 vectors
      for (let i = 0; i < 100; i++) {
        client.insert(new Float32Array([
          Math.sin(i * 0.1),
          Math.cos(i * 0.1),
          Math.sin(i * 0.2),
          Math.cos(i * 0.2)
        ]));
      }

      // Delete 30 vectors (30%)
      for (let i = 0; i < 30; i++) {
        client.softDelete(i);
      }

      expect(client.deletedCount).toBe(30);
      expect(client.liveCount).toBe(70);

      const result = client.compact();

      expect(result.tombstonesRemoved).toBe(30);
      expect(result.newSize).toBe(70);
      expect(client.deletedCount).toBe(0);
      expect(client.liveCount).toBe(70);
    });
  });

  describe('edge cases', () => {
    it('should handle delete-all-then-compact', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      const ids: number[] = [];
      for (let i = 0; i < 5; i++) {
        ids.push(client.insert(new Float32Array([i / 5, 0, 0, 0])));
      }

      // Delete all
      for (const id of ids) {
        client.softDelete(id);
      }

      expect(client.tombstoneRatio).toBeCloseTo(1.0);

      const result = client.compact();

      expect(result.tombstonesRemoved).toBe(5);
      expect(result.newSize).toBe(0);
      expect(client.deletedCount).toBe(0);
      expect(client.liveCount).toBe(0);
    });

    it('should handle multiple compact calls', async () => {
      const client = await EdgeVecClient.create({ dimensions: 4 });

      for (let i = 0; i < 10; i++) {
        client.insert(new Float32Array([i / 10, 0, 0, 0]));
      }

      client.softDelete(0);
      client.softDelete(1);

      // First compact
      const result1 = client.compact();
      expect(result1.tombstonesRemoved).toBe(2);

      // Second compact (no tombstones)
      const result2 = client.compact();
      expect(result2.tombstonesRemoved).toBe(0);
      expect(result2.newSize).toBe(8);
    });
  });
});
