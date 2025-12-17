# EdgeVec Filtering WASM API Architecture

**Document:** `FILTERING_WASM_API.md`
**Version:** 1.0.0
**Status:** [APPROVED]
**Author:** WASM_SPECIALIST / META_ARCHITECT
**Date:** 2025-12-17
**Week:** 22 | **Day:** 4 | **Task:** W22.4

---

## Executive Summary

This document defines the JavaScript/TypeScript API for EdgeVec's metadata filtering system. The API must be ergonomic for developers, type-safe for TypeScript users, and efficient across the WASM boundary.

**Key Decision:** EdgeVec will implement a **Hybrid API** offering both string-based queries (familiar to SQL users) and a builder pattern (type-safe for complex queries).

**Design Principles:**
1. **Familiar Syntax**: SQL-like string filters for quick adoption
2. **Type Safety**: Full TypeScript definitions with IntelliSense support
3. **Minimal Bundle**: <50KB for filter module, <300KB total
4. **Rich Errors**: Structured exceptions with position info and suggestions
5. **Zero-Copy Where Possible**: Minimize WASM boundary crossings

---

## Table of Contents

1. [API Design Philosophy](#1-api-design-philosophy)
2. [TypeScript Interface Definitions](#2-typescript-interface-definitions)
3. [API Usage Examples](#3-api-usage-examples)
4. [WASM Serialization Format](#4-wasm-serialization-format)
5. [Error Handling Specification](#5-error-handling-specification)
6. [Bundle Size Analysis](#6-bundle-size-analysis)
7. [Performance Considerations](#7-performance-considerations)
8. [Migration Guide](#8-migration-guide)
9. [Implementation Roadmap](#9-implementation-roadmap)

---

## 1. API Design Philosophy

### 1.1 Design Options Analysis

| Aspect | String-Based | Builder Pattern | Hybrid (Recommended) |
|:-------|:-------------|:----------------|:---------------------|
| Learning curve | Low (SQL-like) | Medium | Low (simple) / Medium (complex) |
| Type safety | None | Full | Optional |
| IDE support | None | Excellent | Good |
| Compile-time errors | No | Yes | Partial |
| Code readability | Good | Verbose | Good |
| Dynamic filters | Easy | Easy | Easy |
| Bundle size | Smaller | Larger | Medium |

### 1.2 Recommended Approach: Hybrid API

**Rationale:**
- Simple queries (80% of use cases): Use string syntax
- Complex/dynamic queries (20% of use cases): Use builder pattern
- TypeScript users get full type safety when needed
- JavaScript users get SQL-like familiarity

```typescript
// Simple: String-based (most common)
const results = await index.search(query, 10, {
  filter: 'category = "gpu" AND price < 1000'
});

// Complex: Builder pattern (when needed)
const filter = Filter.and(
  Filter.eq('category', 'gpu'),
  Filter.or(
    Filter.lt('price', 500),
    Filter.eq('discount', true)
  )
);
const results = await index.search(query, 10, { filter });
```

### 1.3 API Surface Summary

| Category | Count | Purpose |
|:---------|:------|:--------|
| Core Types | 4 | MetadataValue, SearchOptions, SearchResult, Filter |
| Builder Types | 3 | FilterBuilder, FieldCondition, FilterStatic |
| Error Types | 5 | FilterException, FilterErrorCode, ErrorPolicy, etc. |
| Strategy Types | 2 | FilterStrategy, FilteredSearchResult |
| Utility Types | 3 | ParsedFilter, FilterStats, FilterValidation |
| **Total** | **17** | Complete filter API surface |

---

## 2. TypeScript Interface Definitions

### 2.1 Core Types

```typescript
// ═══════════════════════════════════════════════════════════════════════════
// FILE: edgevec.d.ts (additions to existing types)
// ═══════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────
// METADATA VALUE TYPES
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Supported metadata value types.
 *
 * Maps to Rust `MetadataValue` enum:
 * - string  → MetadataValue::String(String)
 * - number  → MetadataValue::Int(i64) | MetadataValue::Float(f64)
 * - boolean → MetadataValue::Bool(bool)
 * - string[] → MetadataValue::Array(Vec<String>)
 *
 * @example
 * const metadata: Record<string, MetadataValue> = {
 *   category: "electronics",     // string
 *   price: 599.99,              // number (float)
 *   stock: 42,                  // number (int)
 *   inStock: true,              // boolean
 *   tags: ["gpu", "nvidia"],    // string[]
 * };
 */
export type MetadataValue = string | number | boolean | string[];

/**
 * Metadata record for a vector.
 *
 * Keys are field names, values are MetadataValue.
 * Keys must be valid identifiers (alphanumeric + underscore).
 */
export type Metadata = Record<string, MetadataValue>;

// ─────────────────────────────────────────────────────────────────────────────
// SEARCH OPTIONS
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Filter strategy for combining filtering with HNSW search.
 *
 * @see docs/architecture/FILTER_STRATEGY.md for detailed analysis
 */
export type FilterStrategy =
  | 'auto'      // Let EdgeVec choose (recommended)
  | 'pre'       // Filter all vectors first, then search
  | 'post'      // Search first, then filter candidates
  | 'hybrid';   // Adaptive oversampling based on selectivity

/**
 * Options for filtered search.
 */
export interface SearchOptions {
  /**
   * Filter expression.
   *
   * Can be:
   * - String: SQL-like filter syntax (e.g., 'category = "gpu"')
   * - Filter: Pre-built filter object from builder
   *
   * @example
   * // String syntax
   * { filter: 'price < 1000 AND category = "gpu"' }
   *
   * // Builder syntax
   * { filter: Filter.eq('category', 'gpu') }
   */
  filter?: string | Filter;

  /**
   * Filter strategy selection.
   *
   * - 'auto' (default): EdgeVec selects based on estimated selectivity
   * - 'pre': Force pre-filter (best for >80% selectivity)
   * - 'post': Force post-filter (best for <10% selectivity)
   * - 'hybrid': Force hybrid with configurable oversample
   *
   * @default 'auto'
   */
  strategy?: FilterStrategy;

  /**
   * Oversample factor for hybrid/post strategy.
   *
   * Multiplies k to get candidate count before filtering.
   * Only used when strategy is 'post' or 'hybrid'.
   *
   * @default 3.0
   * @minimum 1.0
   * @maximum 10.0
   */
  oversampleFactor?: number;

  /**
   * Include metadata in search results.
   *
   * When true, each SearchResult includes its metadata.
   * Slightly increases response size.
   *
   * @default false
   */
  includeMetadata?: boolean;

  /**
   * Include original vectors in search results.
   *
   * When true, each SearchResult includes the vector data.
   * Significantly increases response size.
   *
   * @default false
   */
  includeVectors?: boolean;

  /**
   * Override ef_search parameter for this query.
   *
   * Higher values increase recall but also latency.
   * If not set, uses index default or strategy-calculated value.
   */
  efSearch?: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// SEARCH RESULTS
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Single search result.
 */
export interface SearchResult {
  /**
   * Vector ID (0-based index).
   */
  id: number;

  /**
   * Similarity score.
   *
   * For cosine similarity: 0.0 to 1.0 (higher = more similar)
   * For L2 distance: 0.0 to Infinity (lower = more similar)
   */
  score: number;

  /**
   * Vector metadata (if includeMetadata was true).
   */
  metadata?: Metadata;

  /**
   * Original vector (if includeVectors was true).
   */
  vector?: Float32Array;
}

/**
 * Extended search result with filter diagnostics.
 */
export interface FilteredSearchResult {
  /**
   * Search results (may be fewer than k if filter is restrictive).
   */
  results: SearchResult[];

  /**
   * Whether the full k results were found.
   *
   * If false, filter may be too restrictive.
   */
  complete: boolean;

  /**
   * Observed selectivity (fraction of vectors passing filter).
   *
   * Useful for tuning filter strategy.
   */
  observedSelectivity: number;

  /**
   * Strategy actually used for this query.
   */
  strategyUsed: FilterStrategy;

  /**
   * Number of vectors evaluated during search.
   */
  vectorsEvaluated: number;

  /**
   * Filter evaluation time in milliseconds.
   */
  filterTimeMs: number;

  /**
   * Total search time in milliseconds.
   */
  totalTimeMs: number;
}
```

### 2.2 Filter Builder Types

```typescript
// ─────────────────────────────────────────────────────────────────────────────
// FILTER BUILDER API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Compiled filter expression.
 *
 * Opaque type representing a validated, optimized filter.
 * Can be reused across multiple searches.
 */
export interface Filter {
  /**
   * Convert filter back to string representation.
   */
  toString(): string;

  /**
   * Serialize filter to JSON for debugging.
   */
  toJSON(): object;

  /**
   * Check if filter is a tautology (always true).
   */
  readonly isTautology: boolean;

  /**
   * Check if filter is a contradiction (always false).
   */
  readonly isContradiction: boolean;

  /**
   * Estimated complexity (1-10 scale).
   */
  readonly complexity: number;
}

/**
 * Static filter factory methods.
 */
export declare const Filter: FilterStatic;

/**
 * Filter factory interface.
 */
export interface FilterStatic {
  // ═══════════════════════════════════════════════════════════════════════
  // PARSING
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Parse a filter string into a Filter object.
   *
   * @param query - Filter string in EdgeVec syntax
   * @returns Compiled filter
   * @throws FilterException on syntax error
   *
   * @example
   * const filter = Filter.parse('category = "gpu" AND price < 1000');
   */
  parse(query: string): Filter;

  /**
   * Try to parse a filter string, returning null on error.
   *
   * @param query - Filter string in EdgeVec syntax
   * @returns Compiled filter or null
   *
   * @example
   * const filter = Filter.tryParse(userInput);
   * if (filter) {
   *   // Valid filter
   * }
   */
  tryParse(query: string): Filter | null;

  /**
   * Validate a filter string without compiling.
   *
   * @param query - Filter string to validate
   * @returns Validation result with any errors
   *
   * @example
   * const result = Filter.validate('price <');
   * if (!result.valid) {
   *   console.log(result.errors[0].message);
   * }
   */
  validate(query: string): FilterValidation;

  // ═══════════════════════════════════════════════════════════════════════
  // COMPARISON OPERATORS
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Create equality filter: field = value
   *
   * @example
   * Filter.eq('category', 'gpu')
   * // Equivalent to: category = "gpu"
   */
  eq(field: string, value: MetadataValue): Filter;

  /**
   * Create inequality filter: field != value
   *
   * @example
   * Filter.ne('status', 'deleted')
   * // Equivalent to: status != "deleted"
   */
  ne(field: string, value: MetadataValue): Filter;

  /**
   * Create less-than filter: field < value
   *
   * @example
   * Filter.lt('price', 1000)
   * // Equivalent to: price < 1000
   */
  lt(field: string, value: number): Filter;

  /**
   * Create less-than-or-equal filter: field <= value
   *
   * @example
   * Filter.le('price', 1000)
   * // Equivalent to: price <= 1000
   */
  le(field: string, value: number): Filter;

  /**
   * Create greater-than filter: field > value
   *
   * @example
   * Filter.gt('rating', 4.0)
   * // Equivalent to: rating > 4.0
   */
  gt(field: string, value: number): Filter;

  /**
   * Create greater-than-or-equal filter: field >= value
   *
   * @example
   * Filter.ge('stock', 10)
   * // Equivalent to: stock >= 10
   */
  ge(field: string, value: number): Filter;

  // ═══════════════════════════════════════════════════════════════════════
  // RANGE OPERATORS
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Create range filter: low <= field <= high
   *
   * @example
   * Filter.between('price', 100, 500)
   * // Equivalent to: price BETWEEN 100 500
   */
  between(field: string, low: number, high: number): Filter;

  // ═══════════════════════════════════════════════════════════════════════
  // STRING OPERATORS
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Create contains filter: field CONTAINS substring
   *
   * @example
   * Filter.contains('title', 'NVIDIA')
   * // Equivalent to: title CONTAINS "NVIDIA"
   */
  contains(field: string, substring: string): Filter;

  /**
   * Create starts-with filter: field STARTS_WITH prefix
   *
   * @example
   * Filter.startsWith('sku', 'GPU-')
   * // Equivalent to: sku STARTS_WITH "GPU-"
   */
  startsWith(field: string, prefix: string): Filter;

  /**
   * Create ends-with filter: field ENDS_WITH suffix
   *
   * @example
   * Filter.endsWith('filename', '.pdf')
   * // Equivalent to: filename ENDS_WITH ".pdf"
   */
  endsWith(field: string, suffix: string): Filter;

  /**
   * Create pattern match filter: field LIKE pattern
   *
   * Pattern uses SQL-like wildcards:
   * - % matches any sequence of characters
   * - _ matches any single character
   *
   * @example
   * Filter.like('email', '%@company.com')
   * // Equivalent to: email LIKE "%@company.com"
   */
  like(field: string, pattern: string): Filter;

  // ═══════════════════════════════════════════════════════════════════════
  // SET OPERATORS
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Create IN filter: field IN [values]
   *
   * @example
   * Filter.in('category', ['gpu', 'cpu', 'ram'])
   * // Equivalent to: category IN ["gpu", "cpu", "ram"]
   */
  in(field: string, values: MetadataValue[]): Filter;

  /**
   * Create NOT IN filter: field NOT IN [values]
   *
   * @example
   * Filter.notIn('status', ['deleted', 'archived'])
   * // Equivalent to: status NOT IN ["deleted", "archived"]
   */
  notIn(field: string, values: MetadataValue[]): Filter;

  // ═══════════════════════════════════════════════════════════════════════
  // ARRAY OPERATORS (for array-valued fields)
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Create ANY filter: ANY(field, value)
   *
   * True if any element in field array equals value.
   *
   * @example
   * Filter.any('tags', 'nvidia')
   * // Equivalent to: ANY(tags, "nvidia")
   */
  any(field: string, value: MetadataValue): Filter;

  /**
   * Create ALL filter: ALL(field, values)
   *
   * True if all values are present in field array.
   *
   * @example
   * Filter.all('tags', ['gpu', 'gaming'])
   * // Equivalent to: ALL(tags, ["gpu", "gaming"])
   */
  all(field: string, values: MetadataValue[]): Filter;

  /**
   * Create NONE filter: NONE(field, values)
   *
   * True if none of the values are in field array.
   *
   * @example
   * Filter.none('tags', ['nsfw', 'spam'])
   * // Equivalent to: NONE(tags, ["nsfw", "spam"])
   */
  none(field: string, values: MetadataValue[]): Filter;

  // ═══════════════════════════════════════════════════════════════════════
  // NULL OPERATORS
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Create IS NULL filter
   *
   * @example
   * Filter.isNull('deletedAt')
   * // Equivalent to: deletedAt IS NULL
   */
  isNull(field: string): Filter;

  /**
   * Create IS NOT NULL filter
   *
   * @example
   * Filter.isNotNull('verifiedAt')
   * // Equivalent to: verifiedAt IS NOT NULL
   */
  isNotNull(field: string): Filter;

  // ═══════════════════════════════════════════════════════════════════════
  // LOGICAL OPERATORS
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Create AND combination of filters
   *
   * @example
   * Filter.and(
   *   Filter.eq('category', 'gpu'),
   *   Filter.lt('price', 1000)
   * )
   * // Equivalent to: category = "gpu" AND price < 1000
   */
  and(...filters: Filter[]): Filter;

  /**
   * Create OR combination of filters
   *
   * @example
   * Filter.or(
   *   Filter.eq('category', 'gpu'),
   *   Filter.eq('category', 'cpu')
   * )
   * // Equivalent to: category = "gpu" OR category = "cpu"
   */
  or(...filters: Filter[]): Filter;

  /**
   * Create NOT (negation) of a filter
   *
   * @example
   * Filter.not(Filter.eq('status', 'deleted'))
   * // Equivalent to: NOT status = "deleted"
   */
  not(filter: Filter): Filter;

  // ═══════════════════════════════════════════════════════════════════════
  // SPECIAL FILTERS
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * Filter that matches all vectors (no filtering).
   */
  readonly all: Filter;

  /**
   * Filter that matches no vectors (empty result).
   */
  readonly none: Filter;
}

/**
 * Fluent filter builder for complex filters.
 *
 * @example
 * const filter = new FilterBuilder()
 *   .where('category').eq('gpu')
 *   .and('price').lt(1000)
 *   .or('discount').eq(true)
 *   .build();
 */
export declare class FilterBuilder {
  constructor();

  /**
   * Start a new condition on a field.
   */
  where(field: string): FieldCondition;

  /**
   * Add an AND condition on a field.
   */
  and(field: string): FieldCondition;

  /**
   * Add an OR condition on a field.
   */
  or(field: string): FieldCondition;

  /**
   * Negate the current filter.
   */
  not(): FilterBuilder;

  /**
   * Group current conditions in parentheses.
   */
  group(): FilterBuilder;

  /**
   * Build the final filter.
   *
   * @throws FilterException if filter is invalid
   */
  build(): Filter;

  /**
   * Get the filter string representation.
   */
  toString(): string;
}

/**
 * Field condition builder (returned by FilterBuilder.where/and/or).
 */
export interface FieldCondition {
  eq(value: MetadataValue): FilterBuilder;
  ne(value: MetadataValue): FilterBuilder;
  lt(value: number): FilterBuilder;
  le(value: number): FilterBuilder;
  gt(value: number): FilterBuilder;
  ge(value: number): FilterBuilder;
  between(low: number, high: number): FilterBuilder;
  contains(substring: string): FilterBuilder;
  startsWith(prefix: string): FilterBuilder;
  endsWith(suffix: string): FilterBuilder;
  like(pattern: string): FilterBuilder;
  in(values: MetadataValue[]): FilterBuilder;
  notIn(values: MetadataValue[]): FilterBuilder;
  isNull(): FilterBuilder;
  isNotNull(): FilterBuilder;
}
```

### 2.3 Error Types

```typescript
// ─────────────────────────────────────────────────────────────────────────────
// ERROR HANDLING
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Filter error codes for programmatic handling.
 */
export enum FilterErrorCode {
  // ═══════════════════════════════════════════════════════════════════════
  // PARSE ERRORS (1xx)
  // ═══════════════════════════════════════════════════════════════════════

  /** Invalid filter syntax */
  SYNTAX_ERROR = 'E100',

  /** Unexpected end of input */
  UNEXPECTED_EOF = 'E101',

  /** Invalid character in filter */
  INVALID_CHAR = 'E102',

  /** Unclosed string literal */
  UNCLOSED_STRING = 'E103',

  /** Unclosed parenthesis */
  UNCLOSED_PAREN = 'E104',

  /** Invalid number format */
  INVALID_NUMBER = 'E105',

  /** Invalid escape sequence */
  INVALID_ESCAPE = 'E106',

  // ═══════════════════════════════════════════════════════════════════════
  // SEMANTIC ERRORS (2xx)
  // ═══════════════════════════════════════════════════════════════════════

  /** Type mismatch in comparison */
  TYPE_MISMATCH = 'E200',

  /** Unknown metadata field (strict mode) */
  UNKNOWN_FIELD = 'E201',

  /** Integer overflow in comparison */
  OVERFLOW = 'E202',

  /** Array operation on non-array field */
  NOT_AN_ARRAY = 'E203',

  /** Empty array in IN/NOT IN */
  EMPTY_ARRAY = 'E204',

  /** Division by zero in expression */
  DIVISION_BY_ZERO = 'E205',

  // ═══════════════════════════════════════════════════════════════════════
  // LIMIT ERRORS (3xx)
  // ═══════════════════════════════════════════════════════════════════════

  /** Filter too complex (too many nodes) */
  TOO_COMPLEX = 'E300',

  /** Filter too deep (nesting limit) */
  TOO_DEEP = 'E301',

  /** String too long */
  STRING_TOO_LONG = 'E302',

  /** Array too large */
  ARRAY_TOO_LARGE = 'E303',

  // ═══════════════════════════════════════════════════════════════════════
  // RUNTIME ERRORS (4xx)
  // ═══════════════════════════════════════════════════════════════════════

  /** Filter evaluation failed */
  EVALUATION_ERROR = 'E400',

  /** Metadata access error */
  METADATA_ERROR = 'E401',

  /** WASM memory error */
  MEMORY_ERROR = 'E402',
}

/**
 * Position in filter string where error occurred.
 */
export interface SourcePosition {
  /** 1-based line number */
  line: number;

  /** 1-based column number */
  column: number;

  /** 0-based character offset */
  offset: number;
}

/**
 * Filter exception with rich error information.
 *
 * @example
 * try {
 *   const filter = Filter.parse('price <');
 * } catch (e) {
 *   if (e instanceof FilterException) {
 *     console.log(e.code);       // 'E101'
 *     console.log(e.message);    // 'Unexpected end of input'
 *     console.log(e.position);   // { line: 1, column: 8 }
 *     console.log(e.suggestion); // 'Expected a value after "<"'
 *   }
 * }
 */
export declare class FilterException extends Error {
  /** Error code for programmatic handling */
  readonly code: FilterErrorCode;

  /** Human-readable error message */
  readonly message: string;

  /** Position in filter string (if applicable) */
  readonly position?: SourcePosition;

  /** Suggestion for fixing the error */
  readonly suggestion?: string;

  /** The filter string that caused the error */
  readonly filterString?: string;

  /** Stack trace */
  readonly stack?: string;

  /**
   * Format error with source snippet.
   *
   * @example
   * console.log(e.format());
   * // Error: Unexpected end of input
   * //   price <
   * //         ^
   * // Expected a value after "<"
   */
  format(): string;
}

/**
 * Validation result from Filter.validate().
 */
export interface FilterValidation {
  /** Whether the filter is valid */
  valid: boolean;

  /** List of validation errors (empty if valid) */
  errors: FilterValidationError[];

  /** List of warnings (valid but suspicious) */
  warnings: FilterValidationWarning[];

  /** Parsed filter (if valid) */
  filter?: Filter;
}

/**
 * Single validation error.
 */
export interface FilterValidationError {
  /** Error code */
  code: FilterErrorCode;

  /** Error message */
  message: string;

  /** Position in filter string */
  position?: SourcePosition;

  /** Suggestion for fixing */
  suggestion?: string;
}

/**
 * Validation warning (valid but suspicious).
 */
export interface FilterValidationWarning {
  /** Warning code */
  code: string;

  /** Warning message */
  message: string;

  /** Position in filter string */
  position?: SourcePosition;
}

/**
 * Error handling policy configuration.
 */
export interface ErrorPolicy {
  /**
   * How to handle unknown fields.
   *
   * - 'strict': Throw UNKNOWN_FIELD error
   * - 'lenient': Treat as NULL (no match)
   *
   * @default 'lenient'
   */
  unknownFields: 'strict' | 'lenient';

  /**
   * How to handle type mismatches.
   *
   * - 'error': Throw TYPE_MISMATCH error
   * - 'coerce': Attempt type coercion
   * - 'false': Return false (no match)
   *
   * @default 'false'
   */
  typeMismatch: 'error' | 'coerce' | 'false';

  /**
   * How to handle null comparisons.
   *
   * - 'sql': NULL comparisons always false (SQL semantics)
   * - 'js': NULL == NULL is true (JavaScript semantics)
   *
   * @default 'sql'
   */
  nullSemantics: 'sql' | 'js';
}
```

### 2.4 Utility Types

```typescript
// ─────────────────────────────────────────────────────────────────────────────
// UTILITY TYPES
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Parsed filter information (for debugging/display).
 */
export interface ParsedFilter {
  /** Original filter string */
  original: string;

  /** Normalized/canonical filter string */
  normalized: string;

  /** AST node count */
  nodeCount: number;

  /** Maximum nesting depth */
  depth: number;

  /** Fields referenced in filter */
  fields: string[];

  /** Operators used */
  operators: string[];
}

/**
 * Filter statistics after evaluation.
 */
export interface FilterStats {
  /** Number of vectors evaluated */
  evaluated: number;

  /** Number of vectors passing filter */
  passed: number;

  /** Selectivity (passed / evaluated) */
  selectivity: number;

  /** Evaluation time in milliseconds */
  evaluationTimeMs: number;

  /** Cache hits (for repeated filters) */
  cacheHits: number;
}

/**
 * Index configuration for filtering.
 */
export interface FilterConfig {
  /** Error handling policy */
  errorPolicy?: Partial<ErrorPolicy>;

  /** Maximum filter complexity (nodes) */
  maxComplexity?: number;

  /** Maximum filter depth (nesting) */
  maxDepth?: number;

  /** Enable filter caching */
  cacheEnabled?: boolean;

  /** Cache size (number of filters) */
  cacheSize?: number;

  /** Cache TTL in milliseconds */
  cacheTtl?: number;
}
```

---

## 3. API Usage Examples

### 3.1 Example 1: Simple Equality Filter

```typescript
import { EdgeVecIndex, Filter } from 'edgevec';

// Create and populate index with metadata
const index = new EdgeVecIndex({ dimensions: 384 });

await index.add(embedding1, { category: 'electronics', price: 999 });
await index.add(embedding2, { category: 'gpu', price: 599 });
await index.add(embedding3, { category: 'gpu', price: 1299 });

// Search with simple string filter
const results = await index.search(queryVector, 10, {
  filter: 'category = "gpu"'
});

console.log(results);
// [
//   { id: 1, score: 0.95 },
//   { id: 2, score: 0.87 }
// ]
```

### 3.2 Example 2: Numeric Range Filter

```typescript
// Price range filter
const results = await index.search(queryVector, 10, {
  filter: 'price >= 500 AND price < 1000'
});

// Equivalent using BETWEEN
const results2 = await index.search(queryVector, 10, {
  filter: 'price BETWEEN 500 999'
});

// Using builder pattern
const filter = Filter.between('price', 500, 999);
const results3 = await index.search(queryVector, 10, { filter });
```

### 3.3 Example 3: String Contains Filter

```typescript
// Find products with "NVIDIA" in title
const results = await index.search(queryVector, 10, {
  filter: 'title CONTAINS "NVIDIA"'
});

// Find documents ending with .pdf
const results2 = await index.search(queryVector, 10, {
  filter: 'filename ENDS_WITH ".pdf"'
});

// Pattern matching
const results3 = await index.search(queryVector, 10, {
  filter: 'email LIKE "%@company.com"'
});
```

### 3.4 Example 4: Complex Boolean Logic

```typescript
// Complex filter with AND/OR/NOT
const results = await index.search(queryVector, 10, {
  filter: `
    (category = "gpu" AND price < 1000)
    OR
    (category = "cpu" AND brand = "AMD")
  `
});

// Equivalent using builder
const filter = Filter.or(
  Filter.and(
    Filter.eq('category', 'gpu'),
    Filter.lt('price', 1000)
  ),
  Filter.and(
    Filter.eq('category', 'cpu'),
    Filter.eq('brand', 'AMD')
  )
);

const results2 = await index.search(queryVector, 10, { filter });
```

### 3.5 Example 5: Array Operations

```typescript
// Find items with specific tag
const results = await index.search(queryVector, 10, {
  filter: 'ANY(tags, "nvidia")'
});

// Find items with all required tags
const results2 = await index.search(queryVector, 10, {
  filter: 'ALL(tags, ["gpu", "gaming"])'
});

// Exclude items with certain tags
const results3 = await index.search(queryVector, 10, {
  filter: 'NONE(tags, ["nsfw", "spam"])'
});

// Using builder
const filter = Filter.any('tags', 'nvidia');
```

### 3.6 Example 6: NULL Handling

```typescript
// Find items not yet processed
const results = await index.search(queryVector, 10, {
  filter: 'processedAt IS NULL'
});

// Find verified items
const results2 = await index.search(queryVector, 10, {
  filter: 'verifiedAt IS NOT NULL AND status = "active"'
});
```

### 3.7 Example 7: Error Handling

```typescript
import { Filter, FilterException, FilterErrorCode } from 'edgevec';

// Validate before searching
const validation = Filter.validate(userInput);
if (!validation.valid) {
  console.error('Invalid filter:', validation.errors[0].message);
  console.error('Suggestion:', validation.errors[0].suggestion);
  return;
}

// Or use try/catch
try {
  const results = await index.search(queryVector, 10, {
    filter: userInput
  });
} catch (e) {
  if (e instanceof FilterException) {
    switch (e.code) {
      case FilterErrorCode.SYNTAX_ERROR:
        console.error('Syntax error:', e.message);
        console.error('At position:', e.position);
        break;
      case FilterErrorCode.TYPE_MISMATCH:
        console.error('Type error:', e.message);
        break;
      default:
        console.error('Filter error:', e.format());
    }
  } else {
    throw e; // Re-throw non-filter errors
  }
}
```

### 3.8 Example 8: Strategy Selection

```typescript
// Let EdgeVec choose the best strategy (recommended)
const results1 = await index.search(queryVector, 10, {
  filter: 'category = "gpu"',
  strategy: 'auto'  // default
});

// Force pre-filter for high-selectivity filter
const results2 = await index.search(queryVector, 10, {
  filter: 'active = true',  // ~90% of items are active
  strategy: 'pre'
});

// Force post-filter with custom oversample for low-selectivity
const results3 = await index.search(queryVector, 10, {
  filter: 'premium = true',  // ~1% are premium
  strategy: 'post',
  oversampleFactor: 10
});

// Get detailed diagnostics
const result = await index.searchFiltered(queryVector, 10, {
  filter: 'category = "gpu" AND price < 500'
}) as FilteredSearchResult;

console.log('Strategy used:', result.strategyUsed);
console.log('Selectivity:', result.observedSelectivity);
console.log('Filter time:', result.filterTimeMs, 'ms');
```

### 3.9 Example 9: Fluent Builder Pattern

```typescript
// Complex filter with fluent builder
const filter = new FilterBuilder()
  .where('category').eq('gpu')
  .and('price').between(500, 1500)
  .and('brand').in(['NVIDIA', 'AMD'])
  .or('featured').eq(true)
  .build();

const results = await index.search(queryVector, 10, { filter });

// With negation and grouping
const filter2 = new FilterBuilder()
  .where('status').ne('deleted')
  .and('category').eq('electronics')
  .group()  // Groups previous conditions
  .or('featured').eq(true)
  .build();
// Equivalent to: (status != "deleted" AND category = "electronics") OR featured = true
```

### 3.10 Example 10: Filter Reuse and Caching

```typescript
// Pre-compile filter for reuse
const frequentFilter = Filter.parse('category = "gpu" AND inStock = true');

// Use in multiple searches (filter is cached internally)
for (const query of queries) {
  const results = await index.search(query, 10, {
    filter: frequentFilter
  });
  // Process results...
}

// Get filter statistics
const info = frequentFilter as ParsedFilter;
console.log('Filter complexity:', frequentFilter.complexity);
console.log('Is tautology:', frequentFilter.isTautology);
```

---

## 4. WASM Serialization Format

### 4.1 Overview

Filters cross the WASM boundary in two scenarios:
1. **JS → WASM**: When executing a search with a filter
2. **WASM → JS**: When returning filter diagnostics

### 4.2 Serialization Options

| Format | Size | Speed | Complexity | Chosen For |
|:-------|:-----|:------|:-----------|:-----------|
| JSON | ~300 bytes | Medium | Low | MVP (Week 23) |
| MessagePack | ~150 bytes | Fast | Medium | Future opt |
| Custom Binary | ~80 bytes | Fastest | High | Future opt |

**Decision:** JSON for MVP, with option to add binary format in v0.6.0 if profiling shows serialization bottleneck.

### 4.3 JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EdgeVec Filter Expression",
  "description": "JSON representation of a filter expression for WASM boundary",

  "definitions": {
    "value": {
      "oneOf": [
        { "type": "string" },
        { "type": "number" },
        { "type": "boolean" },
        {
          "type": "array",
          "items": { "type": "string" }
        }
      ]
    },

    "comparison": {
      "type": "object",
      "required": ["op", "field", "value"],
      "properties": {
        "op": {
          "enum": ["eq", "ne", "lt", "le", "gt", "ge",
                   "contains", "starts_with", "ends_with", "like"]
        },
        "field": { "type": "string" },
        "value": { "$ref": "#/definitions/value" }
      }
    },

    "between": {
      "type": "object",
      "required": ["op", "field", "low", "high"],
      "properties": {
        "op": { "const": "between" },
        "field": { "type": "string" },
        "low": { "type": "number" },
        "high": { "type": "number" }
      }
    },

    "set_op": {
      "type": "object",
      "required": ["op", "field", "values"],
      "properties": {
        "op": { "enum": ["in", "not_in", "any", "all", "none"] },
        "field": { "type": "string" },
        "values": {
          "type": "array",
          "items": { "$ref": "#/definitions/value" }
        }
      }
    },

    "null_check": {
      "type": "object",
      "required": ["op", "field"],
      "properties": {
        "op": { "enum": ["is_null", "is_not_null"] },
        "field": { "type": "string" }
      }
    },

    "binary_logic": {
      "type": "object",
      "required": ["op", "left", "right"],
      "properties": {
        "op": { "enum": ["and", "or"] },
        "left": { "$ref": "#/definitions/expr" },
        "right": { "$ref": "#/definitions/expr" }
      }
    },

    "unary_logic": {
      "type": "object",
      "required": ["op", "inner"],
      "properties": {
        "op": { "const": "not" },
        "inner": { "$ref": "#/definitions/expr" }
      }
    },

    "expr": {
      "oneOf": [
        { "$ref": "#/definitions/comparison" },
        { "$ref": "#/definitions/between" },
        { "$ref": "#/definitions/set_op" },
        { "$ref": "#/definitions/null_check" },
        { "$ref": "#/definitions/binary_logic" },
        { "$ref": "#/definitions/unary_logic" }
      ]
    }
  },

  "$ref": "#/definitions/expr"
}
```

### 4.4 JSON Examples

```json
// category = "gpu"
{
  "op": "eq",
  "field": "category",
  "value": "gpu"
}

// price >= 100 AND price < 500
{
  "op": "and",
  "left": {
    "op": "ge",
    "field": "price",
    "value": 100
  },
  "right": {
    "op": "lt",
    "field": "price",
    "value": 500
  }
}

// category IN ["gpu", "cpu"] AND NOT deleted
{
  "op": "and",
  "left": {
    "op": "in",
    "field": "category",
    "values": ["gpu", "cpu"]
  },
  "right": {
    "op": "not",
    "inner": {
      "op": "eq",
      "field": "deleted",
      "value": true
    }
  }
}
```

### 4.5 WASM Boundary Functions

```rust
// ═══════════════════════════════════════════════════════════════════════════
// RUST WASM EXPORTS (src/wasm/filter.rs)
// ═══════════════════════════════════════════════════════════════════════════

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Parse and validate a filter string.
///
/// # Arguments
/// * `filter_str` - Filter expression string
///
/// # Returns
/// * JSON string of parsed filter, or throws JsValue on error
///
#[wasm_bindgen]
pub fn parse_filter(filter_str: &str) -> Result<String, JsValue> {
    let ast = crate::filter::parse(filter_str)
        .map_err(|e| JsValue::from(FilterException::from(e)))?;

    serde_json::to_string(&ast)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Validate a filter string without fully parsing.
///
/// # Returns
/// * JSON string of validation result
///
#[wasm_bindgen]
pub fn validate_filter(filter_str: &str) -> String {
    let result = crate::filter::validate(filter_str);
    serde_json::to_string(&result).unwrap_or_default()
}

/// Execute a filtered search.
///
/// # Arguments
/// * `index_ptr` - Pointer to index (from create_index)
/// * `query_json` - JSON array of query vector
/// * `k` - Number of results
/// * `options_json` - JSON string of SearchOptions
///
/// # Returns
/// * JSON string of search results
///
#[wasm_bindgen]
pub fn search_filtered(
    index_ptr: *mut HnswIndex,
    query_json: &str,
    k: usize,
    options_json: &str,
) -> Result<String, JsValue> {
    let index = unsafe { &*index_ptr };
    let query: Vec<f32> = serde_json::from_str(query_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid query: {}", e)))?;
    let options: SearchOptions = serde_json::from_str(options_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid options: {}", e)))?;

    let results = index.search_filtered(&query, k, options)
        .map_err(|e| JsValue::from(FilterException::from(e)))?;

    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

### 4.6 JavaScript Wrapper

```typescript
// ═══════════════════════════════════════════════════════════════════════════
// TYPESCRIPT WRAPPER (pkg/edgevec.ts)
// ═══════════════════════════════════════════════════════════════════════════

import * as wasm from './edgevec_bg.wasm';

export class EdgeVecIndex {
  private ptr: number;

  // ... existing methods ...

  /**
   * Search with optional filter.
   */
  async search(
    query: Float32Array | number[],
    k: number,
    options?: SearchOptions
  ): Promise<SearchResult[]> {
    const queryJson = JSON.stringify(Array.from(query));
    const optionsJson = JSON.stringify(this.normalizeOptions(options));

    try {
      const resultJson = wasm.search_filtered(
        this.ptr,
        queryJson,
        k,
        optionsJson
      );
      return JSON.parse(resultJson);
    } catch (e) {
      throw this.wrapError(e);
    }
  }

  /**
   * Search with detailed filter diagnostics.
   */
  async searchFiltered(
    query: Float32Array | number[],
    k: number,
    options?: SearchOptions
  ): Promise<FilteredSearchResult> {
    const queryJson = JSON.stringify(Array.from(query));
    const optionsJson = JSON.stringify({
      ...this.normalizeOptions(options),
      includeDiagnostics: true
    });

    try {
      const resultJson = wasm.search_filtered(
        this.ptr,
        queryJson,
        k,
        optionsJson
      );
      return JSON.parse(resultJson);
    } catch (e) {
      throw this.wrapError(e);
    }
  }

  private normalizeOptions(options?: SearchOptions): object {
    if (!options) return {};

    // Convert Filter object to JSON
    if (options.filter && typeof options.filter !== 'string') {
      return {
        ...options,
        filter: options.filter.toJSON()
      };
    }

    return options;
  }

  private wrapError(e: unknown): Error {
    if (typeof e === 'string') {
      try {
        const parsed = JSON.parse(e);
        return new FilterException(
          parsed.code,
          parsed.message,
          parsed.position,
          parsed.suggestion
        );
      } catch {
        return new Error(e);
      }
    }
    return e instanceof Error ? e : new Error(String(e));
  }
}
```

---

## 5. Error Handling Specification

### 5.1 Error Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ERROR PROPAGATION FLOW                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │   JS Input   │───►│    WASM      │───►│   Rust       │                  │
│  │   (string)   │    │   Boundary   │    │   Parser     │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│                             │                    │                          │
│                             │                    ▼                          │
│                             │           ┌──────────────┐                   │
│                             │           │ FilterError  │                   │
│                             │           │    (Rust)    │                   │
│                             │           └──────────────┘                   │
│                             │                    │                          │
│                             │     ┌──────────────┘                          │
│                             │     │                                         │
│                             ▼     ▼                                         │
│                      ┌──────────────┐                                       │
│                      │   JsValue    │  (serialized error)                   │
│                      │   (JSON)     │                                       │
│                      └──────────────┘                                       │
│                             │                                               │
│                             ▼                                               │
│                      ┌──────────────┐                                       │
│                      │FilterException                                       │
│                      │    (JS)      │                                       │
│                      └──────────────┘                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Error Mapping Table

| Rust Error | Code | JS Exception | Recoverable |
|:-----------|:-----|:-------------|:------------|
| `ParseError::SyntaxError` | E100 | `FilterException` | Yes |
| `ParseError::UnexpectedEof` | E101 | `FilterException` | Yes |
| `ParseError::InvalidChar` | E102 | `FilterException` | Yes |
| `EvalError::TypeMismatch` | E200 | `FilterException` | Depends |
| `EvalError::UnknownField` | E201 | `FilterException` | Depends |
| `LimitError::TooComplex` | E300 | `FilterException` | Yes |
| `LimitError::TooDeep` | E301 | `FilterException` | Yes |
| `RuntimeError::*` | E4xx | `FilterException` | No |
| `panic!()` | - | `Error` (generic) | No |

### 5.3 Error Serialization Format

```json
{
  "error": true,
  "code": "E100",
  "message": "Unexpected token 'AND' at position 5",
  "position": {
    "line": 1,
    "column": 5,
    "offset": 4
  },
  "suggestion": "Did you mean 'field AND value'? An operator requires both operands.",
  "context": {
    "filterString": "foo AND",
    "snippet": "foo AND\n    ^"
  }
}
```

### 5.4 Error Messages by Code

| Code | Message Template | Example |
|:-----|:-----------------|:--------|
| E100 | `Syntax error: {detail}` | "Syntax error: unexpected '}'" |
| E101 | `Unexpected end of input` | "Unexpected end of input, expected value" |
| E102 | `Invalid character '{char}' at position {pos}` | "Invalid character '@' at position 5" |
| E103 | `Unclosed string literal starting at position {pos}` | "Unclosed string literal starting at position 10" |
| E200 | `Type mismatch: cannot compare {type1} with {type2}` | "Type mismatch: cannot compare string with number" |
| E201 | `Unknown field '{field}'` | "Unknown field 'categry' (did you mean 'category'?)" |
| E300 | `Filter too complex: {nodes} nodes exceeds limit of {max}` | "Filter too complex: 150 nodes exceeds limit of 100" |
| E301 | `Filter nesting too deep: depth {depth} exceeds limit of {max}` | "Filter nesting too deep: depth 12 exceeds limit of 10" |

---

## 6. Bundle Size Analysis

### 6.1 Current State

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     EDGEVEC BUNDLE SIZE ANALYSIS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Current Bundle (v0.4.0):                                                   │
│  ├── edgevec_bg.wasm .................. 248 KB                              │
│  ├── edgevec.js ....................... 12 KB                               │
│  └── edgevec.d.ts ..................... 4 KB                                │
│  Total: 264 KB                                                              │
│                                                                             │
│  Breakdown (estimated from nm/wasm-objdump):                                │
│  ├── HNSW index ....................... 80 KB (32%)                         │
│  ├── Binary quantization .............. 35 KB (14%)                         │
│  ├── SIMD implementations ............. 45 KB (18%)                         │
│  ├── Serialization (serde) ............ 40 KB (16%)                         │
│  ├── wasm-bindgen glue ................ 25 KB (10%)                         │
│  └── Other ............................ 23 KB (10%)                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Filter Module Estimate

| Component | Estimated Size | Notes |
|:----------|:---------------|:------|
| **Parser (pest)** | 15 KB | Grammar + generated parser |
| - Grammar rules | 3 KB | EBNF → pest format |
| - Parser code | 10 KB | Generated from grammar |
| - Error handling | 2 KB | Position tracking |
| **AST** | 8 KB | 27 variants + serde |
| - Enum definitions | 3 KB | FilterExpr enum |
| - Serde derive | 5 KB | JSON serialization |
| **Evaluator** | 10 KB | Recursive evaluation |
| - Core evaluation | 6 KB | Match arms |
| - Short-circuit | 2 KB | AND/OR optimization |
| - Error types | 2 KB | 14 error variants |
| **Strategy** | 5 KB | Pre/Post/Hybrid/Auto |
| - Selectivity estimation | 2 KB | Sampling logic |
| - Strategy selection | 3 KB | Decision matrix |
| **WASM bindings** | 4 KB | wasm-bindgen exports |
| - parse_filter | 1 KB | |
| - validate_filter | 1 KB | |
| - search_filtered | 2 KB | |
| **Total Filter Module** | **42 KB** | |

### 6.3 Projected Total Bundle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     PROJECTED BUNDLE SIZE (v0.5.0)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Current (v0.4.0) ..................... 264 KB                              │
│  + Filter module ...................... +42 KB                              │
│  - Potential optimizations ............ -10 KB (wasm-opt)                   │
│  ─────────────────────────────────────────────                              │
│  Projected Total ...................... ~296 KB                             │
│                                                                             │
│  Budget Compliance:                                                         │
│  ├── Filter module budget: 50 KB ...... PASS (42 KB < 50 KB)               │
│  └── Total bundle budget: 500 KB ...... PASS (296 KB < 500 KB)             │
│                                                                             │
│  Comparison with Competitors:                                               │
│  ├── EdgeVec v0.5.0 ................... ~296 KB                             │
│  ├── Voy .............................. ~180 KB (no metadata filtering)     │
│  ├── Orama ............................ ~400 KB (full-text + vectors)       │
│  └── LanceDB WASM ..................... ~2 MB (full database)              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.4 Size Optimization Strategies

If bundle size becomes a concern:

| Strategy | Savings | Trade-off |
|:---------|:--------|:----------|
| Tree-shaking (manual) | 5-10 KB | Complexity |
| Replace pest with hand-written parser | 8 KB | Development time |
| Binary AST format (no JSON) | 3 KB | Less debuggable |
| Optional filter feature flag | Full 42 KB | Two builds |
| wasm-opt -Oz | 10-15 KB | Slightly slower |

---

## 7. Performance Considerations

### 7.1 WASM Boundary Overhead

| Operation | Time | Notes |
|:----------|:-----|:------|
| JSON.stringify (filter) | ~50μs | For complex filter |
| WASM call overhead | ~5μs | Per function call |
| JSON.parse (results) | ~100μs | For 100 results |
| Filter eval (in WASM) | ~2-5ms | Main work |
| **Total overhead** | **~150-200μs** | <1% of search time |

### 7.2 Optimization Techniques

```typescript
// ═══════════════════════════════════════════════════════════════════════════
// OPTIMIZATION: Pre-compile filters for reuse
// ═══════════════════════════════════════════════════════════════════════════

// BAD: Parse filter on every search
for (const query of queries) {
  const results = await index.search(query, 10, {
    filter: 'category = "gpu"'  // Parsed each time!
  });
}

// GOOD: Pre-compile filter once
const filter = Filter.parse('category = "gpu"');
for (const query of queries) {
  const results = await index.search(query, 10, { filter });
}

// ═══════════════════════════════════════════════════════════════════════════
// OPTIMIZATION: Batch searches with same filter
// ═══════════════════════════════════════════════════════════════════════════

// GOOD: Single batch call (future API)
const batchResults = await index.searchBatch(queries, 10, {
  filter: 'category = "gpu"'
});
```

### 7.3 Memory Considerations

```typescript
// Filter objects are lightweight (~100 bytes each)
// Safe to create many filters without concern

// Avoid holding references to large result sets
const results = await index.search(query, 1000, {
  filter: 'active = true',
  includeVectors: true  // Warning: 1000 × 384 × 4 = 1.5MB!
});

// Better: Stream results or use pagination
for (let offset = 0; offset < 1000; offset += 100) {
  const batch = await index.search(query, 100, {
    filter: 'active = true',
    offset: offset
  });
  processBatch(batch);
}
```

---

## 8. Migration Guide

### 8.1 From v0.4.0 to v0.5.0

```typescript
// ═══════════════════════════════════════════════════════════════════════════
// v0.4.0: No filtering (post-filter in JS)
// ═══════════════════════════════════════════════════════════════════════════

// OLD: Fetch more, filter in JS
const allResults = await index.search(query, 100);
const filtered = allResults.filter(r => {
  const metadata = getMetadata(r.id);  // Separate lookup!
  return metadata.category === 'gpu' && metadata.price < 1000;
});
const topK = filtered.slice(0, 10);

// ═══════════════════════════════════════════════════════════════════════════
// v0.5.0: Native filtering
// ═══════════════════════════════════════════════════════════════════════════

// NEW: Filter in WASM
const results = await index.search(query, 10, {
  filter: 'category = "gpu" AND price < 1000'
});

// Benefits:
// - No separate metadata lookup
// - Filter evaluated during HNSW traversal
// - Guaranteed 10 results (if they exist)
// - ~10x faster for selective filters
```

### 8.2 API Compatibility

| v0.4.0 Method | v0.5.0 Equivalent | Notes |
|:--------------|:------------------|:------|
| `search(q, k)` | `search(q, k)` | Unchanged |
| `search(q, k)` + JS filter | `search(q, k, { filter })` | New option |
| `add(v)` | `add(v, metadata?)` | Optional metadata |
| N/A | `searchFiltered(q, k, opts)` | New method |
| N/A | `Filter.parse(str)` | New class |
| N/A | `FilterBuilder` | New class |

---

## 9. Implementation Roadmap

### 9.1 Week 23 Tasks (Filter Implementation)

| Task | Description | Hours | Dependencies |
|:-----|:------------|:------|:-------------|
| W23.W1 | TypeScript type definitions | 4 | FILTERING_WASM_API.md |
| W23.W2 | Filter static methods | 4 | Parser |
| W23.W3 | FilterBuilder class | 4 | W23.W2 |
| W23.W4 | WASM bindings | 6 | Rust evaluator |
| W23.W5 | Error handling | 4 | W23.W4 |
| W23.W6 | JS wrapper | 4 | W23.W5 |
| W23.W7 | Unit tests (TypeScript) | 4 | W23.W6 |
| W23.W8 | Integration tests | 4 | All |
| **Total** | | **34h** | |

### 9.2 File Structure

```
pkg/
├── edgevec.d.ts          # TypeScript definitions (updated)
├── edgevec.js            # JavaScript wrapper (updated)
├── edgevec_bg.wasm       # WASM binary (updated)
├── filter.ts             # Filter builder (new)
└── errors.ts             # Error classes (new)

src/
├── wasm/
│   ├── mod.rs            # Existing
│   └── filter.rs         # WASM filter bindings (new)
├── filter/
│   ├── mod.rs            # Filter module root (new)
│   ├── parser.rs         # Filter parser (new)
│   ├── ast.rs            # AST types (new)
│   ├── evaluator.rs      # Evaluator (new)
│   ├── strategy.rs       # Strategy selection (new)
│   └── error.rs          # Error types (new)
└── ...
```

---

## Document Metadata

| Field | Value |
|:------|:------|
| **Document** | `docs/architecture/FILTERING_WASM_API.md` |
| **Version** | 1.0.0 |
| **Status** | [PROPOSED] |
| **Word Count** | ~5,500 |
| **Type Definitions** | 17 |
| **Example Snippets** | 10 |
| **Author** | WASM_SPECIALIST / META_ARCHITECT |
| **Reviewer** | HOSTILE_REVIEWER |
| **Created** | 2025-12-17 |
| **Last Modified** | 2025-12-17 |

---

**END OF FILTERING_WASM_API.md**

---

*"A good API is invisible. A great API is inevitable."*
