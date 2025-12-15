// wasm/__tests__/__mocks__/wasm.mock.ts
// Mock WASM module for Node.js testing

/**
 * Mock WasmCompactionResult for testing
 */
export interface MockCompactionResult {
  tombstones_removed: number;
  new_size: number;
  duration_ms: number;
}

/**
 * Mock WASM EdgeVec class for Node.js testing.
 * This allows unit testing the TypeScript wrapper without actual WASM.
 */
export class MockWasmEdgeVec {
  private vectors: Map<number, { data: Float32Array; deleted: boolean }> = new Map();
  private nextId = 0;
  private _compactionThreshold = 0.3;

  insert(vector: Float32Array): number {
    const id = this.nextId++;
    this.vectors.set(id, { data: vector, deleted: false });
    return id;
  }

  search(query: Float32Array, k: number): Array<{ id: number; score: number }> {
    const results: Array<{ id: number; score: number; distance: number }> = [];

    for (const [id, entry] of this.vectors) {
      if (!entry.deleted) {
        // Simple L2 distance calculation
        let sum = 0;
        for (let i = 0; i < query.length; i++) {
          const diff = query[i] - entry.data[i];
          sum += diff * diff;
        }
        results.push({ id, score: Math.sqrt(sum), distance: Math.sqrt(sum) });
      }
    }

    results.sort((a, b) => a.score - b.score);
    return results.slice(0, k);
  }

  softDelete(id: number): boolean {
    const entry = this.vectors.get(id);
    if (!entry) {
      throw new Error(`Vector ${id} not found`);
    }
    if (entry.deleted) {
      return false;
    }
    entry.deleted = true;
    return true;
  }

  isDeleted(id: number): boolean {
    const entry = this.vectors.get(id);
    if (!entry) {
      throw new Error(`Vector ${id} not found`);
    }
    return entry.deleted;
  }

  deletedCount(): number {
    let count = 0;
    for (const entry of this.vectors.values()) {
      if (entry.deleted) count++;
    }
    return count;
  }

  liveCount(): number {
    let count = 0;
    for (const entry of this.vectors.values()) {
      if (!entry.deleted) count++;
    }
    return count;
  }

  tombstoneRatio(): number {
    const total = this.vectors.size;
    if (total === 0) return 0;
    return this.deletedCount() / total;
  }

  needsCompaction(): boolean {
    return this.tombstoneRatio() > this._compactionThreshold;
  }

  compactionThreshold(): number {
    return this._compactionThreshold;
  }

  setCompactionThreshold(ratio: number): void {
    this._compactionThreshold = Math.max(0.01, Math.min(0.99, ratio));
  }

  compactionWarning(): string | null {
    if (this.needsCompaction()) {
      const ratio = (this.tombstoneRatio() * 100).toFixed(1);
      return `Compaction recommended: ${ratio}% tombstones exceed threshold`;
    }
    return null;
  }

  compact(): MockCompactionResult {
    const startTime = Date.now();
    const tombstonesRemoved = this.deletedCount();

    // Remove deleted entries
    for (const [id, entry] of this.vectors) {
      if (entry.deleted) {
        this.vectors.delete(id);
      }
    }

    return {
      tombstones_removed: tombstonesRemoved,
      new_size: this.vectors.size,
      duration_ms: Date.now() - startTime
    };
  }

  async save(_name: string): Promise<void> {
    // Mock save - no-op in tests
  }

  static async load(_name: string): Promise<MockWasmEdgeVec> {
    return new MockWasmEdgeVec();
  }
}

/**
 * Mock EdgeVecConfig class
 */
export class MockWasmConfig {
  dimensions: number;
  metric: string = 'l2';

  constructor(dimensions: number) {
    this.dimensions = dimensions;
  }
}

/**
 * Mock init function - no-op in tests
 */
export async function mockInit(): Promise<void> {
  // No-op for testing
}
