/**
 * EdgeVec Filter Functions - Functional Composition API
 *
 * Provides standalone functions for building filters with functional composition.
 * These functions wrap the Filter class methods for a cleaner, more composable syntax.
 *
 * @module filter-functions
 * @version 0.8.0
 *
 * @example
 * ```typescript
 * import { filter, and, or, eq, gt, lt } from 'edgevec';
 *
 * // Simple filter
 * const byCategory = eq('category', 'electronics');
 *
 * // Composed filter
 * const query = filter(
 *   and(
 *     eq('category', 'electronics'),
 *     gt('price', 100),
 *     lt('price', 1000)
 *   )
 * );
 *
 * // Nested logic
 * const complex = filter(
 *   and(
 *     eq('status', 'active'),
 *     or(
 *       eq('brand', 'Apple'),
 *       eq('brand', 'Samsung')
 *     )
 *   )
 * );
 *
 * // Use in search
 * const results = await index.search(embedding, 10, { filter: query });
 * ```
 */

import { Filter, FilterExpression, MetadataValue } from './filter.js';

// =============================================================================
// Comparison Functions
// =============================================================================

/**
 * Equal to: field = value
 *
 * @example eq('category', 'gpu')
 */
export function eq(field: string, value: MetadataValue): FilterExpression {
  return Filter.eq(field, value);
}

/**
 * Not equal to: field != value
 *
 * @example ne('status', 'deleted')
 */
export function ne(field: string, value: MetadataValue): FilterExpression {
  return Filter.ne(field, value);
}

/**
 * Greater than: field > value
 *
 * @example gt('price', 100)
 */
export function gt(field: string, value: number): FilterExpression {
  return Filter.gt(field, value);
}

/**
 * Less than: field < value
 *
 * @example lt('price', 1000)
 */
export function lt(field: string, value: number): FilterExpression {
  return Filter.lt(field, value);
}

/**
 * Greater than or equal: field >= value
 *
 * @example ge('rating', 4.0)
 */
export function ge(field: string, value: number): FilterExpression {
  return Filter.ge(field, value);
}

/**
 * Less than or equal: field <= value
 *
 * @example le('stock', 100)
 */
export function le(field: string, value: number): FilterExpression {
  return Filter.le(field, value);
}

/**
 * Between (inclusive): low <= field <= high
 *
 * @example between('price', 100, 500)
 */
export function between(field: string, low: number, high: number): FilterExpression {
  return Filter.between(field, low, high);
}

// =============================================================================
// String Functions
// =============================================================================

/**
 * Contains substring: field CONTAINS substring
 *
 * @example contains('title', 'NVIDIA')
 */
export function contains(field: string, substring: string): FilterExpression {
  return Filter.contains(field, substring);
}

/**
 * Starts with prefix: field STARTS_WITH prefix
 *
 * @example startsWith('sku', 'GPU-')
 */
export function startsWith(field: string, prefix: string): FilterExpression {
  return Filter.startsWith(field, prefix);
}

/**
 * Ends with suffix: field ENDS_WITH suffix
 *
 * @example endsWith('filename', '.pdf')
 */
export function endsWith(field: string, suffix: string): FilterExpression {
  return Filter.endsWith(field, suffix);
}

/**
 * LIKE pattern match: field LIKE pattern
 * Use % for wildcard matching.
 *
 * @example like('email', '%@company.com')
 */
export function like(field: string, pattern: string): FilterExpression {
  return Filter.like(field, pattern);
}

// =============================================================================
// Array/Set Functions
// =============================================================================

/**
 * In array of values: field IN [values]
 *
 * @example inArray('category', ['gpu', 'cpu', 'ram'])
 */
export function inArray(field: string, values: MetadataValue[]): FilterExpression {
  return Filter.in(field, values);
}

/**
 * Not in array of values: field NOT IN [values]
 *
 * @example notInArray('status', ['deleted', 'archived'])
 */
export function notInArray(field: string, values: MetadataValue[]): FilterExpression {
  return Filter.notIn(field, values);
}

/**
 * ANY - array field contains value: ANY(field, value)
 *
 * @example any('tags', 'nvidia')
 */
export function any(field: string, value: MetadataValue): FilterExpression {
  return Filter.any(field, value);
}

/**
 * ALL - array field contains all values: ALL(field, values)
 *
 * @example all('tags', ['gpu', 'gaming'])
 */
export function all(field: string, values: MetadataValue[]): FilterExpression {
  return Filter.allOf(field, values);
}

/**
 * NONE - array field contains none of values: NONE(field, values)
 *
 * @example none('tags', ['nsfw', 'spam'])
 */
export function none(field: string, values: MetadataValue[]): FilterExpression {
  return Filter.none(field, values);
}

// =============================================================================
// Null Functions
// =============================================================================

/**
 * Is null: field IS NULL
 *
 * @example isNull('deletedAt')
 */
export function isNull(field: string): FilterExpression {
  return Filter.isNull(field);
}

/**
 * Is not null: field IS NOT NULL
 *
 * @example isNotNull('verifiedAt')
 */
export function isNotNull(field: string): FilterExpression {
  return Filter.isNotNull(field);
}

// =============================================================================
// Logical Combinators
// =============================================================================

/**
 * AND - all conditions must match
 * Accepts variadic arguments for composing multiple filters.
 *
 * @example and(eq('a', 1), gt('b', 2), lt('c', 10))
 */
export function and(...filters: FilterExpression[]): FilterExpression {
  if (filters.length === 0) {
    throw new Error('and() requires at least one filter');
  }
  if (filters.length === 1) {
    return filters[0];
  }
  return filters.reduce((acc, f) => Filter.and(acc, f));
}

/**
 * OR - any condition must match
 * Accepts variadic arguments for composing multiple filters.
 *
 * @example or(eq('brand', 'Apple'), eq('brand', 'Samsung'))
 */
export function or(...filters: FilterExpression[]): FilterExpression {
  if (filters.length === 0) {
    throw new Error('or() requires at least one filter');
  }
  if (filters.length === 1) {
    return filters[0];
  }
  return filters.reduce((acc, f) => Filter.or(acc, f));
}

/**
 * NOT - negate condition
 *
 * @example not(eq('status', 'deleted'))
 */
export function not(f: FilterExpression): FilterExpression {
  return Filter.not(f);
}

// =============================================================================
// Top-level Wrapper
// =============================================================================

/**
 * Identity wrapper for filter expressions.
 * Useful for readability at the top level of composed filters.
 *
 * @example
 * const query = filter(
 *   and(eq('a', 1), gt('b', 2))
 * );
 */
export function filter(expression: FilterExpression): FilterExpression {
  return expression;
}

// =============================================================================
// Special Filters
// =============================================================================

/**
 * Match all vectors (no filtering)
 *
 * @example matchAll()
 */
export function matchAll(): FilterExpression {
  return Filter.matchAll;
}

/**
 * Match no vectors (empty result)
 *
 * @example matchNone()
 */
export function matchNone(): FilterExpression {
  return Filter.nothing;
}
