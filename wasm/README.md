# EdgeVec TypeScript Wrapper

High-level TypeScript wrapper for EdgeVec WASM bindings, providing auto-initialization, Promise-based API, and TypeScript ergonomics.

## Installation

```bash
npm install @edgevec/client
```

## Quick Start

```typescript
import { EdgeVecClient } from '@edgevec/client';

// Create a new index
const client = await EdgeVecClient.create({ dimensions: 128 });

// Insert vectors (synchronous)
const vector = new Float32Array(128).fill(0.1);
const id = client.insert(vector);

// Search (synchronous)
const results = client.search(vector, 10);
console.log(results);

// Save to IndexedDB (async)
await client.save('my-database');

// Load from IndexedDB (async)
const loaded = await EdgeVecClient.load('my-database', { dimensions: 128 });
```

## Using the Config Builder

```typescript
import { EdgeVecConfigBuilder } from '@edgevec/client';

const config = new EdgeVecConfigBuilder(128)
  .withMetric('cosine')
  .withQuantization('sq8')
  .build();

const client = await EdgeVecClient.create(config);
```

## API Reference

### EdgeVecClient

#### `EdgeVecClient.create(config: EdgeVecClientConfig): Promise<EdgeVecClient>`

Create a new EdgeVec instance with auto WASM initialization.

**Parameters:**
- `config.dimensions` (number): Vector dimensions (must match all inserted vectors)
- `config.metric?` ('l2' | 'cosine' | 'dot'): Distance metric for similarity calculation
- `config.quantization?` ('none' | 'sq8'): Quantization mode for memory optimization

#### `EdgeVecClient.load(name: string, config: EdgeVecClientConfig): Promise<EdgeVecClient>`

Load an existing database from IndexedDB.

**Parameters:**
- `name` (string): Database name
- `config` (EdgeVecClientConfig): Configuration (must match saved database)

#### `insert(vector: Float32Array): number`

Insert a vector into the index. **Synchronous operation.**

**Returns:** Assigned vector ID

**Throws:** Error if vector dimension doesn't match config

#### `search(query: Float32Array, k: number): SearchResult[]`

Search for k nearest neighbors. **Synchronous operation.**

**Returns:** Search results sorted by distance

**Throws:** Error if query dimension doesn't match config or k <= 0

#### `save(name: string): Promise<void>`

Save database to IndexedDB.

#### `length: number`

Get the number of vectors inserted in the current session.

**Important Limitation:** This only tracks vectors inserted via `insert()` after the client was created or loaded. For databases loaded from IndexedDB, this will be 0 until new vectors are inserted, as the WASM API doesn't expose the total vector count from loaded databases.

#### `dimensions: number`

Get the configured dimensions.

### EdgeVecConfigBuilder

Fluent builder for EdgeVec configuration.

```typescript
const config = new EdgeVecConfigBuilder(dimensions)
  .withMetric(metric)
  .withQuantization(quantization)
  .build();
```

## Types

### SearchResult

```typescript
interface SearchResult {
  id: number;        // Unique identifier of the matched vector
  distance: number;  // Distance from query (lower is more similar for L2/cosine)
}
```

### EdgeVecClientConfig

```typescript
interface EdgeVecClientConfig {
  dimensions: number;
  metric?: 'l2' | 'cosine' | 'dot';
  quantization?: 'none' | 'sq8';
}
```

## Development

```bash
# Install dependencies
npm install

# Run tests
npm test

# Run tests with coverage
npm run test:coverage

# Type check
npm run typecheck

# Build
npm run build
```

## Testing

The test suite includes:
- Unit tests for `EdgeVecClient` and `EdgeVecConfig`
- Integration tests for full workflows
- Coverage target: >80%

Run tests:
```bash
npm test
```

## Browser Compatibility

- Chrome 120+
- Firefox 120+
- Safari (deferred to future release)

## License

MIT
