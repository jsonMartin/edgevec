# EdgeVec Filter Examples

A comprehensive collection of copy-paste ready filter examples for EdgeVec v0.8.0.

Each example shows both **string syntax** (simple, inline) and **functional API** (composable, type-safe).

---

## Table of Contents

1. [Basic Filters](#basic-filters)
   - [Example 1: Equality](#example-1-equality)
   - [Example 2: Inequality](#example-2-inequality)
   - [Example 3: Greater Than](#example-3-greater-than)
   - [Example 4: Less Than](#example-4-less-than)
   - [Example 5: Greater Than or Equal](#example-5-greater-than-or-equal)
   - [Example 6: Less Than or Equal](#example-6-less-than-or-equal)
   - [Example 7: Between Range](#example-7-between-range)
2. [String Filters](#string-filters)
   - [Example 8: Contains](#example-8-contains)
   - [Example 9: Starts With](#example-9-starts-with)
   - [Example 10: Ends With](#example-10-ends-with)
3. [Array/Set Filters](#arrayset-filters)
   - [Example 11: IN Array](#example-11-in-array)
   - [Example 12: NOT IN Array](#example-12-not-in-array)
   - [Example 13: ANY](#example-13-any-array-membership)
   - [Example 14: ALL](#example-14-all-all-values-present)
   - [Example 15: NONE](#example-15-none-no-values-present)
4. [Null Filters](#null-filters)
   - [Example 16: IS NULL](#example-16-is-null)
   - [Example 17: IS NOT NULL](#example-17-is-not-null)
5. [Logical Combinations](#logical-combinations)
   - [Example 18: AND](#example-18-and-multiple-conditions)
   - [Example 19: OR](#example-19-or-alternatives)
   - [Example 20: NOT](#example-20-not-negation)
   - [Example 21: Nested AND/OR](#example-21-nested-andor)
   - [Example 22: Complex Multi-Level](#example-22-complex-multi-level)
6. [Real-World Examples](#real-world-examples)
   - [Example 23: E-commerce](#example-23-e-commerce-product-search)
   - [Example 24: Document Retrieval](#example-24-document-retrieval-with-categories)
   - [Example 25: Multi-Tenant](#example-25-multi-tenant-filtering)

---

## Basic Filters

### Example 1: Equality

Find vectors where a field equals a specific value.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'category = "electronics"'
});
```

**Functional API:**
```typescript
import { eq } from 'edgevec';

const results = await index.search(query, 10, {
  filter: eq('category', 'electronics')
});
```

**Use Cases:**
- Product search by category
- User filtering by role
- Document type filtering
- Status-based filtering

---

### Example 2: Inequality

Find vectors where a field does not equal a value.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'status != "deleted"'
});
```

**Functional API:**
```typescript
import { ne } from 'edgevec';

const results = await index.search(query, 10, {
  filter: ne('status', 'deleted')
});
```

**Use Cases:**
- Exclude deleted items
- Filter out specific categories
- Hide draft content
- Exclude certain users

---

### Example 3: Greater Than

Find vectors where a numeric field is greater than a value.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'price > 100'
});
```

**Functional API:**
```typescript
import { gt } from 'edgevec';

const results = await index.search(query, 10, {
  filter: gt('price', 100)
});
```

**Use Cases:**
- Products above price threshold
- High-rated items (rating > 4)
- Recent items (timestamp > date)
- Large files (size > 1MB)

---

### Example 4: Less Than

Find vectors where a numeric field is less than a value.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'price < 500'
});
```

**Functional API:**
```typescript
import { lt } from 'edgevec';

const results = await index.search(query, 10, {
  filter: lt('price', 500)
});
```

**Use Cases:**
- Budget-friendly products
- Short documents (wordCount < 1000)
- Low-priority items
- Items expiring soon

---

### Example 5: Greater Than or Equal

Find vectors where a numeric field is at least a certain value.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'rating >= 4.0'
});
```

**Functional API:**
```typescript
import { ge } from 'edgevec';

const results = await index.search(query, 10, {
  filter: ge('rating', 4.0)
});
```

**Use Cases:**
- Minimum rating threshold
- Items at or above a price point
- Required minimum stock level
- Minimum score filtering

---

### Example 6: Less Than or Equal

Find vectors where a numeric field is at most a certain value.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'stock <= 10'
});
```

**Functional API:**
```typescript
import { le } from 'edgevec';

const results = await index.search(query, 10, {
  filter: le('stock', 10)
});
```

**Use Cases:**
- Low stock alerts
- Budget constraints
- Maximum age limits
- Priority queue filtering

---

### Example 7: Between Range

Find vectors where a numeric field falls within a range (inclusive).

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'price >= 100 AND price <= 500'
});
```

**Functional API:**
```typescript
import { between } from 'edgevec';

const results = await index.search(query, 10, {
  filter: between('price', 100, 500)
});
```

**Use Cases:**
- Price range filtering
- Date range queries
- Age range filtering
- Score bands (0-25, 25-50, etc.)

---

## String Filters

### Example 8: Contains

Find vectors where a string field contains a substring.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'title CONTAINS "NVIDIA"'
});
```

**Functional API:**
```typescript
import { contains } from 'edgevec';

const results = await index.search(query, 10, {
  filter: contains('title', 'NVIDIA')
});
```

**Use Cases:**
- Full-text search within fields
- Brand name matching
- Keyword filtering
- Description search

---

### Example 9: Starts With

Find vectors where a string field starts with a prefix.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'sku STARTS_WITH "GPU-"'
});
```

**Functional API:**
```typescript
import { startsWith } from 'edgevec';

const results = await index.search(query, 10, {
  filter: startsWith('sku', 'GPU-')
});
```

**Use Cases:**
- SKU/product code filtering
- Category prefixes
- Phone number area codes
- Email domain filtering

---

### Example 10: Ends With

Find vectors where a string field ends with a suffix.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'filename ENDS_WITH ".pdf"'
});
```

**Functional API:**
```typescript
import { endsWith } from 'edgevec';

const results = await index.search(query, 10, {
  filter: endsWith('filename', '.pdf')
});
```

**Use Cases:**
- File extension filtering
- Domain suffix matching
- Version suffix filtering
- Format-specific searches

---

## Array/Set Filters

### Example 11: IN Array

Find vectors where a field value is in a list of allowed values.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'category IN ["gpu", "cpu", "ram"]'
});
```

**Functional API:**
```typescript
import { inArray } from 'edgevec';

const results = await index.search(query, 10, {
  filter: inArray('category', ['gpu', 'cpu', 'ram'])
});
```

**Use Cases:**
- Filter by multiple categories
- User role filtering (admin, editor, viewer)
- Status whitelist
- Region filtering

---

### Example 12: NOT IN Array

Find vectors where a field value is NOT in a list of excluded values.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'status NOT IN ["deleted", "archived", "spam"]'
});
```

**Functional API:**
```typescript
import { notInArray } from 'edgevec';

const results = await index.search(query, 10, {
  filter: notInArray('status', ['deleted', 'archived', 'spam'])
});
```

**Use Cases:**
- Exclude multiple statuses
- Blacklist filtering
- Exclude certain categories
- Hide specific content types

---

### Example 13: ANY (Array Membership)

Find vectors where an array field contains a specific value.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'tags ANY "featured"'
});
```

**Functional API:**
```typescript
import { any } from 'edgevec';

const results = await index.search(query, 10, {
  filter: any('tags', 'featured')
});
```

**Use Cases:**
- Tag-based filtering
- Products with specific feature
- Articles with specific topic
- Users with specific permission

---

### Example 14: ALL (All Values Present)

Find vectors where an array field contains ALL specified values.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'tags ALL ["gpu", "gaming"]'
});
```

**Functional API:**
```typescript
import { all } from 'edgevec';

const results = await index.search(query, 10, {
  filter: all('tags', ['gpu', 'gaming'])
});
```

**Use Cases:**
- Products with multiple required features
- Documents with all required tags
- Users with all required permissions
- Items matching all criteria

---

### Example 15: NONE (No Values Present)

Find vectors where an array field contains NONE of the specified values.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'tags NONE ["nsfw", "spam", "blocked"]'
});
```

**Functional API:**
```typescript
import { none } from 'edgevec';

const results = await index.search(query, 10, {
  filter: none('tags', ['nsfw', 'spam', 'blocked'])
});
```

**Use Cases:**
- Content moderation (exclude flagged content)
- Safe search filtering
- Exclude blacklisted tags
- Privacy filtering

---

## Null Filters

### Example 16: IS NULL

Find vectors where a field is null or not set.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'deletedAt IS NULL'
});
```

**Functional API:**
```typescript
import { isNull } from 'edgevec';

const results = await index.search(query, 10, {
  filter: isNull('deletedAt')
});
```

**Use Cases:**
- Find undeleted items
- Find unassigned tasks
- Find items without reviews
- Find incomplete records

---

### Example 17: IS NOT NULL

Find vectors where a field has a value (is not null).

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'verifiedAt IS NOT NULL'
});
```

**Functional API:**
```typescript
import { isNotNull } from 'edgevec';

const results = await index.search(query, 10, {
  filter: isNotNull('verifiedAt')
});
```

**Use Cases:**
- Find verified items
- Find assigned tasks
- Find reviewed content
- Find complete records

---

## Logical Combinations

### Example 18: AND (Multiple Conditions)

Combine conditions where ALL must be true.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'category = "electronics" AND price < 500 AND inStock = true'
});
```

**Functional API:**
```typescript
import { and, eq, lt } from 'edgevec';

const results = await index.search(query, 10, {
  filter: and(
    eq('category', 'electronics'),
    lt('price', 500),
    eq('inStock', true)
  )
});
```

**Use Cases:**
- Multi-criteria product search
- Filtered document retrieval
- Complex user queries
- Faceted search

---

### Example 19: OR (Alternatives)

Combine conditions where ANY can be true.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'brand = "Apple" OR brand = "Samsung" OR brand = "Google"'
});
```

**Functional API:**
```typescript
import { or, eq } from 'edgevec';

const results = await index.search(query, 10, {
  filter: or(
    eq('brand', 'Apple'),
    eq('brand', 'Samsung'),
    eq('brand', 'Google')
  )
});
```

**Use Cases:**
- Multiple brand selection
- Alternative category matching
- Flexible status filtering
- Multi-region queries

---

### Example 20: NOT (Negation)

Negate a condition or group of conditions.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: 'NOT (status = "deleted" OR status = "archived")'
});
```

**Functional API:**
```typescript
import { not, or, eq } from 'edgevec';

const results = await index.search(query, 10, {
  filter: not(
    or(
      eq('status', 'deleted'),
      eq('status', 'archived')
    )
  )
});
```

**Use Cases:**
- Exclude complex conditions
- Invert selection criteria
- Negative filtering
- Exception handling

---

### Example 21: Nested AND/OR

Combine AND and OR for complex logic.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: '(category = "electronics" OR category = "computers") AND price < 1000 AND rating >= 4.0'
});
```

**Functional API:**
```typescript
import { and, or, eq, lt, ge } from 'edgevec';

const results = await index.search(query, 10, {
  filter: and(
    or(
      eq('category', 'electronics'),
      eq('category', 'computers')
    ),
    lt('price', 1000),
    ge('rating', 4.0)
  )
});
```

**Use Cases:**
- Category groups with constraints
- Complex product filtering
- Advanced search forms
- Business rule implementation

---

### Example 22: Complex Multi-Level

Deep nesting for sophisticated queries.

**String Syntax:**
```typescript
const results = await index.search(query, 10, {
  filter: '((brand = "Apple" AND price < 1500) OR (brand = "Samsung" AND price < 1000)) AND inStock = true AND rating >= 4.0'
});
```

**Functional API:**
```typescript
import { filter, and, or, eq, lt, ge } from 'edgevec';

const complexFilter = filter(
  and(
    or(
      and(eq('brand', 'Apple'), lt('price', 1500)),
      and(eq('brand', 'Samsung'), lt('price', 1000))
    ),
    eq('inStock', true),
    ge('rating', 4.0)
  )
);

const results = await index.search(query, 10, {
  filter: complexFilter
});
```

**Use Cases:**
- Brand-specific pricing rules
- Tiered product filtering
- Complex business logic
- Dynamic query building

---

## Real-World Examples

### Example 23: E-commerce Product Search

Find electronics under $500 from trusted brands with good ratings.

```typescript
import { filter, and, or, eq, lt, gt } from 'edgevec';

// Build a product search filter
const productFilter = filter(
  and(
    eq('category', 'electronics'),
    lt('price', 500),
    gt('rating', 4.0),
    or(
      eq('brand', 'Apple'),
      eq('brand', 'Samsung'),
      eq('brand', 'Sony')
    ),
    eq('inStock', true)
  )
);

// Search for similar products
const results = await index.search(queryEmbedding, 20, {
  filter: productFilter
});

// Process results
results.forEach(product => {
  console.log(`${product.metadata.name}: $${product.metadata.price}`);
});
```

**Scenario:** User searches for "wireless headphones" with category, price, and brand filters applied.

---

### Example 24: Document Retrieval with Categories

Find published technical documentation containing specific keywords.

```typescript
import { filter, and, eq, contains, isNotNull, any } from 'edgevec';

// Build a document search filter
const docFilter = filter(
  and(
    eq('type', 'documentation'),
    eq('status', 'published'),
    contains('content', 'API'),
    isNotNull('lastReviewedAt'),
    any('tags', 'technical')
  )
);

// Semantic search with filters
const results = await index.search(queryEmbedding, 10, {
  filter: docFilter,
  includeMetadata: true
});

// Display results
results.forEach(doc => {
  console.log(`[${doc.metadata.type}] ${doc.metadata.title}`);
  console.log(`  Last reviewed: ${doc.metadata.lastReviewedAt}`);
});
```

**Scenario:** Developer searches for API documentation that has been reviewed and tagged as technical.

---

### Example 25: Multi-Tenant Filtering

Filter data by tenant and user permissions in a SaaS application.

```typescript
import { filter, and, eq, inArray, ne, isNull } from 'edgevec';

// Current user context
const currentUser = {
  tenantId: 'tenant_123',
  accessibleDepartments: ['engineering', 'product', 'design'],
  role: 'editor'
};

// Build tenant-aware filter
const tenantFilter = filter(
  and(
    // Tenant isolation
    eq('tenantId', currentUser.tenantId),
    // Department access control
    inArray('departmentId', currentUser.accessibleDepartments),
    // Exclude archived content
    ne('status', 'archived'),
    // Only show non-deleted items
    isNull('deletedAt')
  )
);

// Search within tenant's data
const results = await index.search(queryEmbedding, 10, {
  filter: tenantFilter,
  includeMetadata: true
});

// Safe to display - all results belong to user's tenant and accessible departments
results.forEach(item => {
  console.log(`[${item.metadata.departmentId}] ${item.metadata.title}`);
});
```

**Scenario:** SaaS application ensures users only see data from their tenant and departments they have access to.

---

## Quick Reference

### Comparison Functions

| Function | String Syntax | Description |
|:---------|:--------------|:------------|
| `eq(field, value)` | `field = value` | Equals |
| `ne(field, value)` | `field != value` | Not equals |
| `gt(field, value)` | `field > value` | Greater than |
| `lt(field, value)` | `field < value` | Less than |
| `ge(field, value)` | `field >= value` | Greater than or equal |
| `le(field, value)` | `field <= value` | Less than or equal |
| `between(field, low, high)` | `field >= low AND field <= high` | Between (inclusive) |

### String Functions

| Function | String Syntax | Description |
|:---------|:--------------|:------------|
| `contains(field, str)` | `field CONTAINS str` | Contains substring |
| `startsWith(field, prefix)` | `field STARTS_WITH prefix` | Starts with |
| `endsWith(field, suffix)` | `field ENDS_WITH suffix` | Ends with |
| `like(field, pattern)` | `field LIKE pattern` | Pattern match (% wildcard) |

### Array/Set Functions

| Function | String Syntax | Description |
|:---------|:--------------|:------------|
| `inArray(field, values)` | `field IN [values]` | Value in list |
| `notInArray(field, values)` | `field NOT IN [values]` | Value not in list |
| `any(field, value)` | `field ANY value` | Array contains value |
| `all(field, values)` | `field ALL [values]` | Array contains all values |
| `none(field, values)` | `field NONE [values]` | Array contains none |

### Null Functions

| Function | String Syntax | Description |
|:---------|:--------------|:------------|
| `isNull(field)` | `field IS NULL` | Field is null |
| `isNotNull(field)` | `field IS NOT NULL` | Field is not null |

### Logical Functions

| Function | String Syntax | Description |
|:---------|:--------------|:------------|
| `and(...filters)` | `a AND b AND c` | All conditions true |
| `or(...filters)` | `a OR b OR c` | Any condition true |
| `not(filter)` | `NOT (condition)` | Negate condition |
| `filter(expr)` | - | Identity wrapper |

---

## Import Reference

```typescript
// Import individual functions
import {
  // Comparison
  eq, ne, gt, lt, ge, le, between,
  // String
  contains, startsWith, endsWith, like,
  // Array/Set
  inArray, notInArray, any, all, none,
  // Null
  isNull, isNotNull,
  // Logical
  and, or, not, filter,
  // Special
  matchAll, matchNone
} from 'edgevec';

// Or import all filter functions
import * as filters from 'edgevec';
const results = await index.search(query, 10, {
  filter: filters.and(
    filters.eq('category', 'electronics'),
    filters.lt('price', 500)
  )
});
```

---

**Version:** 0.8.0
**Last Updated:** 2026-01-07
**Examples:** 25
