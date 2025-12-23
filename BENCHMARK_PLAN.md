# Vector Database Library Comparison Benchmark

## Executive Summary

This benchmark compares vector search libraries in Node.js for the purpose of selecting the best option for browser/Electron hybrid search architectures.

### Libraries Under Test

| Library | npm Package | Type | Notes |
|---------|-------------|------|-------|
| **EdgeVec** | `edgevec@0.5.4` | WASM HNSW | Browser-first, SQL-like filtering, 747KB |
| **sqlite-vec** | `sqlite-vec@0.1.7-alpha.2` | SQLite Extension | Native SIMD, Alex Garcia's library |
| **PGlite** | `@electric-sql/pglite@0.3.14` | PostgreSQL WASM | pgvector, 3.7MB gzipped |
| **Turso/libSQL** | `@libsql/client@0.15.0` | SQLite Fork | F8/F1BIT vector types |
| **usearch** | `usearch@2.21.4` | Native HNSW | SIMD Hamming, reference baseline |

### Key Questions to Answer

1. **Binary search speed**: Which is fastest for 1-bit vectors?
2. **Float search speed**: Which is fastest for F32/F8 vectors?
3. **Memory efficiency**: RAM usage at 1M vectors
4. **Index build time**: How long to insert and build index
5. **Browser viability**: Which work in browser via WASM?
6. **Recall accuracy**: How accurate is ANN vs exact search?

---

## Test Matrix

### Scales

| Scale | Vectors | Purpose |
|-------|---------|---------|
| 1K | 1,000 | Baseline / sanity check |
| 10K | 10,000 | Small app scenario |
| 100K | 100,000 | Medium app scenario |
| 1M | 1,000,000 | Large-scale stress test |

### Vector Types

| Type | Dimensions | Bytes/Vector | Libraries Supporting |
|------|------------|--------------|---------------------|
| **F32** | 768 | 3,072 | All |
| **INT8/SQ8** | 768 | 768 | sqlite-vec, Turso (F8), EdgeVec (SQ8) |
| **Binary** | 1024 bits | 128 | sqlite-vec (BIT), Turso (F1BIT), usearch (b1), EdgeVec (planned) |

### Metrics

| Metric | Unit | Method |
|--------|------|--------|
| Insert time (total) | ms | `performance.now()` around batch insert |
| Insert throughput | vec/s | vectors / insert_time |
| Index build time | ms | Time for HNSW graph construction (if separate) |
| Query time (cold) | ms | First query after fresh load |
| Query time (warm) | ms | Average of subsequent queries |
| Query time (p50/p95/p99) | ms | Percentiles over 100 queries |
| Memory (heap) | MB | `process.memoryUsage().heapUsed` |
| Memory (RSS) | MB | `process.memoryUsage().rss` |
| Index size (disk) | MB | File size or DB size |
| Recall@10 | % | Overlap with brute force ground truth |

---

## Project Setup

### Directory Structure

```
/Users/json/Projects/edgevec/benchmark/
├── package.json
├── benchmark.ts           # Main orchestrator
├── lib/
│   ├── edgevec.ts        # EdgeVec wrapper
│   ├── sqlite-vec.ts     # sqlite-vec wrapper
│   ├── pglite.ts         # PGlite wrapper
│   ├── turso.ts          # Turso/libSQL wrapper
│   └── usearch.ts        # usearch wrapper (baseline)
├── utils/
│   ├── vectors.ts        # Vector generation
│   ├── metrics.ts        # Timing/memory utils
│   └── recall.ts         # Recall calculation
├── results/
│   └── (benchmark outputs)
└── RESULTS.md            # Generated results
```

### package.json

```json
{
  "name": "vector-benchmark",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "bench": "bun run benchmark.ts",
    "bench:edgevec": "bun run benchmark.ts --lib=edgevec",
    "bench:sqlite-vec": "bun run benchmark.ts --lib=sqlite-vec",
    "bench:pglite": "bun run benchmark.ts --lib=pglite",
    "bench:turso": "bun run benchmark.ts --lib=turso",
    "bench:usearch": "bun run benchmark.ts --lib=usearch",
    "clean": "rm -rf *.db *.index results/"
  },
  "dependencies": {
    "edgevec": "^0.5.4",
    "sqlite-vec": "^0.1.7-alpha.2",
    "@electric-sql/pglite": "^0.3.14",
    "@libsql/client": "^0.15.0",
    "usearch": "^2.21.4",
    "better-sqlite3": "^11.0.0"
  }
}
```

---

## Library Wrappers

### Common Interface

```typescript
// lib/types.ts
export interface VectorLibrary {
  name: string

  // Lifecycle
  init(config: LibraryConfig): Promise<void>
  close(): Promise<void>

  // Operations
  insert(id: number, vector: Float32Array, metadata?: Record<string, unknown>): Promise<void>
  insertBatch(items: { id: number; vector: Float32Array }[]): Promise<void>
  search(query: Float32Array, k: number): Promise<SearchResult[]>

  // Index management
  buildIndex?(): Promise<void>  // For libraries with separate build step

  // Metrics
  getMemoryUsage(): number
  getIndexSize(): number
  getVectorCount(): number
}

export interface LibraryConfig {
  dimensions: number
  vectorType: 'f32' | 'int8' | 'binary'
  metric: 'cosine' | 'l2' | 'hamming'
  // HNSW params (where applicable)
  m?: number
  efConstruction?: number
  efSearch?: number
}

export interface SearchResult {
  id: number
  distance: number
}
```

### EdgeVec Wrapper

```typescript
// lib/edgevec.ts
import { EdgeVec } from 'edgevec'
import type { VectorLibrary, LibraryConfig, SearchResult } from './types'

export class EdgeVecLibrary implements VectorLibrary {
  name = 'EdgeVec'
  private db: EdgeVec | null = null
  private config!: LibraryConfig

  async init(config: LibraryConfig): Promise<void> {
    this.config = config
    this.db = new EdgeVec({
      dimensions: config.dimensions,
      metric: config.metric === 'hamming' ? 'hamming' :
              config.metric === 'l2' ? 'euclidean' : 'cosine',
      m: config.m ?? 16,
      ef_construction: config.efConstruction ?? 100,
    })
    await this.db.init()
  }

  async close(): Promise<void> {
    // EdgeVec cleanup if needed
    this.db = null
  }

  async insert(id: number, vector: Float32Array): Promise<void> {
    if (!this.db) throw new Error('Not initialized')
    await this.db.insert(vector, { id })
  }

  async insertBatch(items: { id: number; vector: Float32Array }[]): Promise<void> {
    if (!this.db) throw new Error('Not initialized')
    for (const item of items) {
      await this.db.insert(item.vector, { id: item.id })
    }
  }

  async search(query: Float32Array, k: number): Promise<SearchResult[]> {
    if (!this.db) throw new Error('Not initialized')
    const results = await this.db.search(query, k)
    return results.map(r => ({
      id: r.metadata?.id as number,
      distance: r.distance
    }))
  }

  getMemoryUsage(): number {
    return process.memoryUsage().heapUsed
  }

  getIndexSize(): number {
    // EdgeVec in-memory, estimate from vector count
    return this.getVectorCount() * (this.config.dimensions * 4 + 50) // f32 + overhead
  }

  getVectorCount(): number {
    return this.db?.count() ?? 0
  }
}
```

### sqlite-vec Wrapper

```typescript
// lib/sqlite-vec.ts
import Database from 'better-sqlite3'
import * as sqliteVec from 'sqlite-vec'
import type { VectorLibrary, LibraryConfig, SearchResult } from './types'

export class SqliteVecLibrary implements VectorLibrary {
  name = 'sqlite-vec'
  private db: Database.Database | null = null
  private config!: LibraryConfig
  private dbPath = './sqlite-vec-bench.db'

  async init(config: LibraryConfig): Promise<void> {
    this.config = config
    this.db = new Database(this.dbPath)
    sqliteVec.load(this.db)

    const vecType = config.vectorType === 'binary'
      ? `bit[${config.dimensions}]`
      : config.vectorType === 'int8'
        ? `int8[${config.dimensions}]`
        : `float[${config.dimensions}]`

    this.db.exec(`
      DROP TABLE IF EXISTS vectors;
      CREATE VIRTUAL TABLE vectors USING vec0(
        id INTEGER PRIMARY KEY,
        embedding ${vecType}
      );
    `)
  }

  async close(): Promise<void> {
    this.db?.close()
    this.db = null
  }

  async insert(id: number, vector: Float32Array): Promise<void> {
    if (!this.db) throw new Error('Not initialized')

    const blob = this.config.vectorType === 'binary'
      ? this.floatToBinaryBlob(vector)
      : this.config.vectorType === 'int8'
        ? this.floatToInt8Blob(vector)
        : Buffer.from(vector.buffer)

    this.db.prepare('INSERT INTO vectors(id, embedding) VALUES (?, ?)').run(id, blob)
  }

  async insertBatch(items: { id: number; vector: Float32Array }[]): Promise<void> {
    if (!this.db) throw new Error('Not initialized')

    const stmt = this.db.prepare('INSERT INTO vectors(id, embedding) VALUES (?, ?)')
    const insertMany = this.db.transaction((items: { id: number; vector: Float32Array }[]) => {
      for (const item of items) {
        const blob = this.config.vectorType === 'binary'
          ? this.floatToBinaryBlob(item.vector)
          : this.config.vectorType === 'int8'
            ? this.floatToInt8Blob(item.vector)
            : Buffer.from(item.vector.buffer)
        stmt.run(item.id, blob)
      }
    })
    insertMany(items)
  }

  async search(query: Float32Array, k: number): Promise<SearchResult[]> {
    if (!this.db) throw new Error('Not initialized')

    const queryBlob = this.config.vectorType === 'binary'
      ? this.floatToBinaryBlob(query)
      : this.config.vectorType === 'int8'
        ? this.floatToInt8Blob(query)
        : Buffer.from(query.buffer)

    const distanceFunc = this.config.metric === 'hamming' ? 'vec_distance_hamming'
      : this.config.metric === 'l2' ? 'vec_distance_L2'
      : 'vec_distance_cosine'

    const rows = this.db.prepare(`
      SELECT id, ${distanceFunc}(embedding, ?) as distance
      FROM vectors
      ORDER BY distance ASC
      LIMIT ?
    `).all(queryBlob, k) as { id: number; distance: number }[]

    return rows
  }

  private floatToBinaryBlob(vec: Float32Array): Buffer {
    const bytes = Math.ceil(vec.length / 8)
    const buf = Buffer.alloc(bytes)
    for (let i = 0; i < vec.length; i++) {
      if (vec[i] > 0) {
        buf[Math.floor(i / 8)] |= (1 << (i % 8))
      }
    }
    return buf
  }

  private floatToInt8Blob(vec: Float32Array): Buffer {
    const buf = Buffer.alloc(vec.length)
    for (let i = 0; i < vec.length; i++) {
      // Scale [-1, 1] to [-127, 127]
      buf[i] = Math.max(-127, Math.min(127, Math.round(vec[i] * 127)))
    }
    return buf
  }

  getMemoryUsage(): number {
    return process.memoryUsage().heapUsed
  }

  getIndexSize(): number {
    const fs = require('fs')
    try {
      return fs.statSync(this.dbPath).size
    } catch {
      return 0
    }
  }

  getVectorCount(): number {
    if (!this.db) return 0
    const row = this.db.prepare('SELECT COUNT(*) as cnt FROM vectors').get() as { cnt: number }
    return row.cnt
  }
}
```

### PGlite Wrapper

```typescript
// lib/pglite.ts
import { PGlite } from '@electric-sql/pglite'
import { vector } from '@electric-sql/pglite/vector'
import type { VectorLibrary, LibraryConfig, SearchResult } from './types'

export class PGliteLibrary implements VectorLibrary {
  name = 'PGlite'
  private db: PGlite | null = null
  private config!: LibraryConfig

  async init(config: LibraryConfig): Promise<void> {
    this.config = config

    this.db = new PGlite({
      dataDir: './pglite-bench',
      extensions: { vector }
    })

    await this.db.exec('CREATE EXTENSION IF NOT EXISTS vector')

    const vecType = config.vectorType === 'binary'
      ? `bit(${config.dimensions})`
      : config.vectorType === 'int8'
        ? `halfvec(${config.dimensions})`  // PGlite doesn't have int8, use halfvec
        : `vector(${config.dimensions})`

    await this.db.exec(`
      DROP TABLE IF EXISTS vectors;
      CREATE TABLE vectors (
        id INTEGER PRIMARY KEY,
        embedding ${vecType}
      );
    `)
  }

  async close(): Promise<void> {
    await this.db?.close()
    this.db = null
  }

  async insert(id: number, vector: Float32Array): Promise<void> {
    if (!this.db) throw new Error('Not initialized')

    const vecStr = this.config.vectorType === 'binary'
      ? this.floatToBitString(vector)
      : `[${Array.from(vector).join(',')}]`

    await this.db.query(
      'INSERT INTO vectors(id, embedding) VALUES ($1, $2)',
      [id, vecStr]
    )
  }

  async insertBatch(items: { id: number; vector: Float32Array }[]): Promise<void> {
    if (!this.db) throw new Error('Not initialized')

    // PGlite batch via COPY or multi-value INSERT
    const values = items.map(item => {
      const vecStr = this.config.vectorType === 'binary'
        ? this.floatToBitString(item.vector)
        : `[${Array.from(item.vector).join(',')}]`
      return `(${item.id}, '${vecStr}')`
    }).join(',')

    await this.db.exec(`INSERT INTO vectors(id, embedding) VALUES ${values}`)
  }

  async search(query: Float32Array, k: number): Promise<SearchResult[]> {
    if (!this.db) throw new Error('Not initialized')

    const queryStr = this.config.vectorType === 'binary'
      ? this.floatToBitString(query)
      : `[${Array.from(query).join(',')}]`

    const distanceOp = this.config.metric === 'hamming' ? '<#>'  // bit_count XOR
      : this.config.metric === 'l2' ? '<->'
      : '<=>'  // cosine

    const result = await this.db.query(`
      SELECT id, embedding ${distanceOp} $1 as distance
      FROM vectors
      ORDER BY distance ASC
      LIMIT $2
    `, [queryStr, k])

    return result.rows.map((r: any) => ({
      id: r.id,
      distance: r.distance
    }))
  }

  private floatToBitString(vec: Float32Array): string {
    let bits = ''
    for (let i = 0; i < vec.length; i++) {
      bits += vec[i] > 0 ? '1' : '0'
    }
    return bits
  }

  getMemoryUsage(): number {
    return process.memoryUsage().heapUsed
  }

  getIndexSize(): number {
    const fs = require('fs')
    try {
      // PGlite stores in dataDir
      const stats = fs.statSync('./pglite-bench')
      return stats.size
    } catch {
      return 0
    }
  }

  getVectorCount(): number {
    // Would need sync query
    return 0
  }
}
```

### Turso/libSQL Wrapper

```typescript
// lib/turso.ts
import { createClient, type Client } from '@libsql/client'
import type { VectorLibrary, LibraryConfig, SearchResult } from './types'

export class TursoLibrary implements VectorLibrary {
  name = 'Turso/libSQL'
  private db: Client | null = null
  private config!: LibraryConfig
  private dbPath = './turso-bench.db'

  async init(config: LibraryConfig): Promise<void> {
    this.config = config
    this.db = createClient({ url: `file:${this.dbPath}` })

    const vecType = config.vectorType === 'binary'
      ? `F1BIT_BLOB(${config.dimensions})`
      : config.vectorType === 'int8'
        ? `F8_BLOB(${config.dimensions})`
        : `F32_BLOB(${config.dimensions})`

    await this.db.execute(`DROP TABLE IF EXISTS vectors`)
    await this.db.execute(`
      CREATE TABLE vectors (
        id INTEGER PRIMARY KEY,
        embedding ${vecType}
      )
    `)
  }

  async close(): Promise<void> {
    this.db?.close()
    this.db = null
  }

  async insert(id: number, vector: Float32Array): Promise<void> {
    if (!this.db) throw new Error('Not initialized')

    const vecSql = `[${Array.from(vector).join(',')}]`
    const vecFunc = this.config.vectorType === 'binary' ? 'vector1bit'
      : this.config.vectorType === 'int8' ? 'vector8'
      : 'vector32'

    await this.db.execute({
      sql: `INSERT INTO vectors(id, embedding) VALUES (?, ${vecFunc}(?))`,
      args: [id, vecSql]
    })
  }

  async insertBatch(items: { id: number; vector: Float32Array }[]): Promise<void> {
    if (!this.db) throw new Error('Not initialized')

    const vecFunc = this.config.vectorType === 'binary' ? 'vector1bit'
      : this.config.vectorType === 'int8' ? 'vector8'
      : 'vector32'

    const values = items.map(item => {
      const vecSql = `[${Array.from(item.vector).join(',')}]`
      return `(${item.id}, ${vecFunc}('${vecSql}'))`
    }).join(',')

    await this.db.execute(`INSERT INTO vectors(id, embedding) VALUES ${values}`)
  }

  async search(query: Float32Array, k: number): Promise<SearchResult[]> {
    if (!this.db) throw new Error('Not initialized')

    const querySql = `[${Array.from(query).join(',')}]`
    const vecFunc = this.config.vectorType === 'binary' ? 'vector1bit'
      : this.config.vectorType === 'int8' ? 'vector8'
      : 'vector32'

    const result = await this.db.execute({
      sql: `
        SELECT id, vector_distance_cos(embedding, ${vecFunc}(?)) as distance
        FROM vectors
        ORDER BY distance ASC
        LIMIT ?
      `,
      args: [querySql, k]
    })

    return result.rows.map(r => ({
      id: r.id as number,
      distance: r.distance as number
    }))
  }

  getMemoryUsage(): number {
    return process.memoryUsage().heapUsed
  }

  getIndexSize(): number {
    const fs = require('fs')
    try {
      return fs.statSync(this.dbPath).size
    } catch {
      return 0
    }
  }

  getVectorCount(): number {
    return 0 // Would need sync
  }
}
```

### usearch Wrapper (Baseline)

```typescript
// lib/usearch.ts
import { Index as UsearchIndex } from 'usearch'
import type { VectorLibrary, LibraryConfig, SearchResult } from './types'

export class UsearchLibrary implements VectorLibrary {
  name = 'usearch'
  private index: UsearchIndex | null = null
  private config!: LibraryConfig

  async init(config: LibraryConfig): Promise<void> {
    this.config = config

    const metric = config.metric === 'hamming' ? 'hamming'
      : config.metric === 'l2' ? 'l2sq'
      : 'cos'

    const quantization = config.vectorType === 'binary' ? 'b1'
      : config.vectorType === 'int8' ? 'i8'
      : 'f32'

    this.index = new UsearchIndex({
      dimensions: config.dimensions,
      metric,
      quantization,
      connectivity: config.m ?? 16,
      expansion_add: config.efConstruction ?? 128,
      expansion_search: config.efSearch ?? 64,
    })
  }

  async close(): Promise<void> {
    this.index = null
  }

  async insert(id: number, vector: Float32Array): Promise<void> {
    if (!this.index) throw new Error('Not initialized')
    this.index.add(BigInt(id), vector)
  }

  async insertBatch(items: { id: number; vector: Float32Array }[]): Promise<void> {
    if (!this.index) throw new Error('Not initialized')
    for (const item of items) {
      this.index.add(BigInt(item.id), item.vector)
    }
  }

  async search(query: Float32Array, k: number): Promise<SearchResult[]> {
    if (!this.index) throw new Error('Not initialized')
    const results = this.index.search(query, k)

    return Array.from(results.keys).map((key, i) => ({
      id: Number(key),
      distance: results.distances[i]
    }))
  }

  getMemoryUsage(): number {
    if (!this.index) return 0
    // Approximate: vectors + HNSW graph overhead
    const vectorBytes = this.config.vectorType === 'binary'
      ? Math.ceil(this.config.dimensions / 8)
      : this.config.vectorType === 'int8'
        ? this.config.dimensions
        : this.config.dimensions * 4
    return this.index.size() * (vectorBytes + 50) // ~50 bytes HNSW overhead
  }

  getIndexSize(): number {
    return this.getMemoryUsage() // In-memory
  }

  getVectorCount(): number {
    return this.index?.size() ?? 0
  }
}
```

---

## Main Benchmark Script

```typescript
// benchmark.ts
import { EdgeVecLibrary } from './lib/edgevec'
import { SqliteVecLibrary } from './lib/sqlite-vec'
import { PGliteLibrary } from './lib/pglite'
import { TursoLibrary } from './lib/turso'
import { UsearchLibrary } from './lib/usearch'
import type { VectorLibrary, LibraryConfig, SearchResult } from './lib/types'
import { writeFileSync, mkdirSync } from 'fs'

// ════════════════════════════════════════════════════════════════════════════
// Configuration
// ════════════════════════════════════════════════════════════════════════════

const SCALES = [1_000, 10_000, 100_000, 1_000_000]
const VECTOR_TYPES = ['f32', 'int8', 'binary'] as const
const DIMENSIONS = { f32: 768, int8: 768, binary: 1024 }
const QUERY_ITERATIONS = 100
const WARMUP_QUERIES = 5
const TOP_K = 10
const BATCH_SIZE = 1000

// ════════════════════════════════════════════════════════════════════════════
// Vector Generation
// ════════════════════════════════════════════════════════════════════════════

function generateRandomFloat(dim: number): Float32Array {
  const vec = new Float32Array(dim)
  for (let i = 0; i < dim; i++) {
    vec[i] = Math.random() * 2 - 1
  }
  // Normalize for cosine
  const norm = Math.sqrt(vec.reduce((sum, v) => sum + v * v, 0))
  for (let i = 0; i < dim; i++) {
    vec[i] /= norm
  }
  return vec
}

function generateVectors(count: number, dim: number): Float32Array[] {
  return Array.from({ length: count }, () => generateRandomFloat(dim))
}

// ════════════════════════════════════════════════════════════════════════════
// Metrics
// ════════════════════════════════════════════════════════════════════════════

interface BenchmarkResult {
  library: string
  scale: number
  vectorType: string
  insertTime: number
  insertThroughput: number
  coldQuery: number
  warmQuery: number
  p50: number
  p95: number
  p99: number
  memoryMB: number
  indexSizeMB: number
  recall10: number
}

function percentile(arr: number[], p: number): number {
  const sorted = [...arr].sort((a, b) => a - b)
  const idx = Math.ceil((p / 100) * sorted.length) - 1
  return sorted[Math.max(0, idx)]
}

function calculateRecall(results: SearchResult[], groundTruth: SearchResult[]): number {
  const resultIds = new Set(results.map(r => r.id))
  const truthIds = groundTruth.map(r => r.id)
  const overlap = truthIds.filter(id => resultIds.has(id)).length
  return overlap / truthIds.length
}

// ════════════════════════════════════════════════════════════════════════════
// Brute Force Ground Truth
// ════════════════════════════════════════════════════════════════════════════

function bruteForceSearch(
  vectors: Float32Array[],
  query: Float32Array,
  k: number,
  metric: 'cosine' | 'l2' | 'hamming'
): SearchResult[] {
  const distances = vectors.map((vec, id) => ({
    id,
    distance: metric === 'cosine' ? cosineDist(vec, query)
      : metric === 'l2' ? l2Dist(vec, query)
      : hammingDist(vec, query)
  }))
  return distances.sort((a, b) => a.distance - b.distance).slice(0, k)
}

function cosineDist(a: Float32Array, b: Float32Array): number {
  let dot = 0, normA = 0, normB = 0
  for (let i = 0; i < a.length; i++) {
    dot += a[i] * b[i]
    normA += a[i] * a[i]
    normB += b[i] * b[i]
  }
  return 1 - dot / (Math.sqrt(normA) * Math.sqrt(normB))
}

function l2Dist(a: Float32Array, b: Float32Array): number {
  let sum = 0
  for (let i = 0; i < a.length; i++) {
    const diff = a[i] - b[i]
    sum += diff * diff
  }
  return Math.sqrt(sum)
}

function hammingDist(a: Float32Array, b: Float32Array): number {
  let dist = 0
  for (let i = 0; i < a.length; i++) {
    if ((a[i] > 0) !== (b[i] > 0)) dist++
  }
  return dist
}

// ════════════════════════════════════════════════════════════════════════════
// Benchmark Runner
// ════════════════════════════════════════════════════════════════════════════

async function benchmarkLibrary(
  library: VectorLibrary,
  scale: number,
  vectorType: typeof VECTOR_TYPES[number]
): Promise<BenchmarkResult> {
  const dim = DIMENSIONS[vectorType]
  const metric = vectorType === 'binary' ? 'hamming' : 'cosine'

  console.log(`\n  ${library.name} @ ${scale.toLocaleString()} vectors (${vectorType})`)

  // Initialize
  const config: LibraryConfig = {
    dimensions: dim,
    vectorType: vectorType === 'binary' ? 'binary' : vectorType === 'int8' ? 'int8' : 'f32',
    metric: metric as any,
    m: 16,
    efConstruction: 100,
    efSearch: 50,
  }

  await library.init(config)

  // Generate vectors
  console.log(`    Generating ${scale.toLocaleString()} vectors...`)
  const vectors = generateVectors(scale, dim)

  // Insert
  console.log(`    Inserting...`)
  const insertStart = performance.now()

  for (let i = 0; i < vectors.length; i += BATCH_SIZE) {
    const batch = vectors.slice(i, i + BATCH_SIZE).map((vec, j) => ({
      id: i + j,
      vector: vec
    }))
    await library.insertBatch(batch)

    if ((i + BATCH_SIZE) % 50000 === 0) {
      const elapsed = (performance.now() - insertStart) / 1000
      const rate = (i + BATCH_SIZE) / elapsed
      console.log(`      ${(i + BATCH_SIZE).toLocaleString()} (${rate.toFixed(0)} vec/s)`)
    }
  }

  const insertTime = performance.now() - insertStart
  console.log(`    Insert: ${(insertTime / 1000).toFixed(2)}s (${(scale / (insertTime / 1000)).toFixed(0)} vec/s)`)

  // Build index if needed
  if (library.buildIndex) {
    console.log(`    Building index...`)
    await library.buildIndex()
  }

  // Generate query vectors
  const queryVectors = generateVectors(QUERY_ITERATIONS + WARMUP_QUERIES, dim)

  // Warmup queries
  console.log(`    Warming up...`)
  for (let i = 0; i < WARMUP_QUERIES; i++) {
    await library.search(queryVectors[i], TOP_K)
  }

  // Timed queries
  console.log(`    Running ${QUERY_ITERATIONS} queries...`)
  const queryTimes: number[] = []
  const allResults: SearchResult[][] = []

  for (let i = 0; i < QUERY_ITERATIONS; i++) {
    const query = queryVectors[WARMUP_QUERIES + i]
    const start = performance.now()
    const results = await library.search(query, TOP_K)
    queryTimes.push(performance.now() - start)
    allResults.push(results)
  }

  // Calculate recall against brute force (sample 10 queries)
  console.log(`    Calculating recall...`)
  let totalRecall = 0
  for (let i = 0; i < 10; i++) {
    const query = queryVectors[WARMUP_QUERIES + i]
    const groundTruth = bruteForceSearch(vectors, query, TOP_K, metric as any)
    totalRecall += calculateRecall(allResults[i], groundTruth)
  }
  const avgRecall = totalRecall / 10

  // Memory and size
  const memoryMB = library.getMemoryUsage() / (1024 * 1024)
  const indexSizeMB = library.getIndexSize() / (1024 * 1024)

  await library.close()

  const result: BenchmarkResult = {
    library: library.name,
    scale,
    vectorType,
    insertTime,
    insertThroughput: scale / (insertTime / 1000),
    coldQuery: queryTimes[0],
    warmQuery: queryTimes.slice(1).reduce((a, b) => a + b, 0) / (queryTimes.length - 1),
    p50: percentile(queryTimes.slice(1), 50),
    p95: percentile(queryTimes.slice(1), 95),
    p99: percentile(queryTimes.slice(1), 99),
    memoryMB,
    indexSizeMB,
    recall10: avgRecall * 100,
  }

  console.log(`    Warm: ${result.warmQuery.toFixed(2)}ms | p50: ${result.p50.toFixed(2)}ms | Recall@10: ${result.recall10.toFixed(1)}%`)

  return result
}

// ════════════════════════════════════════════════════════════════════════════
// Main
// ════════════════════════════════════════════════════════════════════════════

async function main() {
  const args = process.argv.slice(2)
  const libFilter = args.find(a => a.startsWith('--lib='))?.split('=')[1]
  const scaleFilter = args.find(a => a.startsWith('--scale='))?.split('=')[1]

  console.log('═'.repeat(70))
  console.log('VECTOR DATABASE LIBRARY COMPARISON BENCHMARK')
  console.log('═'.repeat(70))

  // Create library instances
  const libraries: VectorLibrary[] = [
    new EdgeVecLibrary(),
    new SqliteVecLibrary(),
    new PGliteLibrary(),
    new TursoLibrary(),
    new UsearchLibrary(),
  ].filter(lib => !libFilter || lib.name.toLowerCase().includes(libFilter.toLowerCase()))

  const scales = scaleFilter
    ? [parseInt(scaleFilter)]
    : SCALES

  const results: BenchmarkResult[] = []

  for (const scale of scales) {
    console.log(`\n${'─'.repeat(70)}`)
    console.log(`SCALE: ${scale.toLocaleString()} vectors`)
    console.log('─'.repeat(70))

    for (const vectorType of VECTOR_TYPES) {
      console.log(`\n  Vector Type: ${vectorType.toUpperCase()}`)

      for (const library of libraries) {
        try {
          const result = await benchmarkLibrary(library, scale, vectorType)
          results.push(result)
        } catch (error) {
          console.error(`    ERROR: ${error}`)
        }
      }
    }
  }

  // Output results
  console.log('\n' + '═'.repeat(70))
  console.log('RESULTS SUMMARY')
  console.log('═'.repeat(70))

  // Group by scale and vector type
  for (const scale of scales) {
    for (const vectorType of VECTOR_TYPES) {
      const scaleResults = results.filter(r => r.scale === scale && r.vectorType === vectorType)
      if (scaleResults.length === 0) continue

      console.log(`\n${scale.toLocaleString()} vectors - ${vectorType.toUpperCase()}:`)
      console.log('| Library | Insert (vec/s) | Query (ms) | p95 (ms) | Recall@10 | Memory (MB) |')
      console.log('|---------|---------------|------------|----------|-----------|-------------|')

      for (const r of scaleResults.sort((a, b) => a.warmQuery - b.warmQuery)) {
        console.log(`| ${r.library.padEnd(7)} | ${r.insertThroughput.toFixed(0).padStart(13)} | ${r.warmQuery.toFixed(2).padStart(10)} | ${r.p95.toFixed(2).padStart(8)} | ${r.recall10.toFixed(1).padStart(8)}% | ${r.memoryMB.toFixed(1).padStart(11)} |`)
      }
    }
  }

  // Save results
  mkdirSync('./results', { recursive: true })
  writeFileSync('./results/benchmark.json', JSON.stringify(results, null, 2))
  console.log('\n\nResults saved to ./results/benchmark.json')
}

main().catch(console.error)
```

---

## Running the Benchmark

```bash
# Install dependencies
cd /Users/json/Projects/edgevec/benchmark
bun install

# Run full benchmark
bun run bench

# Run specific library
bun run bench:edgevec
bun run bench:sqlite-vec
bun run bench:usearch

# Run specific scale
bun run benchmark.ts --scale=10000

# Run specific library at specific scale
bun run benchmark.ts --lib=edgevec --scale=100000
```

---

## Expected Results (Estimates)

Based on previous benchmarks and library documentation:

### F32 Search @ 1M vectors

| Library | Est. Query Time | Notes |
|---------|----------------|-------|
| usearch | 0.3-0.5ms | HNSW + SIMD |
| EdgeVec | 1-3ms | WASM HNSW |
| sqlite-vec | 150-200ms | Brute force |
| PGlite | 2,000-3,000ms | WASM overhead |
| Turso | 400-500ms | F32 brute force |

### Binary Search @ 1M vectors

| Library | Est. Query Time | Notes |
|---------|----------------|-------|
| usearch | 0.3-0.5ms | HNSW + SIMD Hamming |
| sqlite-vec | 40-50ms | SIMD Hamming brute force |
| EdgeVec | N/A (planned) | Binary not yet supported |
| PGlite | 150-200ms | Bit type |
| Turso | 300-400ms | F1BIT, no SIMD |

### Memory @ 1M vectors (768-dim F32)

| Library | Est. RAM | Notes |
|---------|----------|-------|
| usearch | 120-150 MB | HNSW in-memory |
| EdgeVec | 150-200 MB | WASM + HNSW |
| sqlite-vec | 50 MB | On-disk |
| PGlite | 3-4 GB | Full Postgres in WASM |
| Turso | 50 MB | On-disk |

---

## Next Steps

1. Create benchmark directory and initialize project
2. Implement library wrappers
3. Run benchmarks at each scale
4. Analyze results and create RESULTS.md
5. Determine optimal hybrid architecture for browser + Turso sync
