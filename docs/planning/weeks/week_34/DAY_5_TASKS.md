# Week 34 Day 5: Filter Examples (Advanced)

**Date:** 2026-01-24
**Focus:** Complete filter examples document (advanced & real-world)
**Hours:** 2h
**Status:** [x] COMPLETE

---

## Objectives

Complete the filter examples document with advanced patterns and real-world examples.

---

## Tasks

### W34.2.2: Advanced & Real-World Examples (2h)

**Goal:** Examples 11-25 of the filter examples document.

**Subtasks:**

- [x] **5.1** Array/Set filter examples (30min)
  - Example 11: IN array (`inArray`)
  - Example 12: NOT IN array (`notInArray`)
  - Example 13: ANY (array membership) (`any`)
  - Example 14: ALL (all values present) (`all`)
  - Example 15: NONE (no values present) (`none`)

- [x] **5.2** Null filter examples (15min)
  - Example 16: IS NULL (`isNull`)
  - Example 17: IS NOT NULL (`isNotNull`)

- [x] **5.3** Logical combination examples (30min)
  - Example 18: AND (multiple conditions)
  - Example 19: OR (alternatives)
  - Example 20: NOT (negation)
  - Example 21: Nested AND/OR
  - Example 22: Complex multi-level

- [x] **5.4** Real-world examples (30min)
  - Example 23: E-commerce product search
  - Example 24: Document retrieval with categories
  - Example 25: Multi-tenant filtering

- [x] **5.5** Add cross-reference to README (15min)
  - Link to filter examples from main README
  - Update filter documentation section

---

## Real-World Example Templates

### Example 23: E-commerce Product Search

```typescript
import { filter, and, or, eq, gt, lt, between, inArray } from 'edgevec';

// Find electronics under $500 from trusted brands
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

const results = await index.search(queryEmbedding, 20, {
  filter: productFilter
});
```

### Example 24: Document Retrieval

```typescript
import { filter, and, eq, contains, isNotNull } from 'edgevec';

// Find published technical docs containing "API"
const docFilter = filter(
  and(
    eq('type', 'documentation'),
    eq('status', 'published'),
    contains('content', 'API'),
    isNotNull('lastReviewedAt')
  )
);

const results = await index.search(queryEmbedding, 10, {
  filter: docFilter
});
```

### Example 25: Multi-Tenant Filtering

```typescript
import { filter, and, eq, inArray } from 'edgevec';

// Filter by tenant and user's accessible departments
const tenantFilter = filter(
  and(
    eq('tenantId', currentUser.tenantId),
    inArray('departmentId', currentUser.accessibleDepartments),
    ne('status', 'archived')
  )
);

const results = await index.search(queryEmbedding, 10, {
  filter: tenantFilter
});
```

---

## Verification

- [x] Examples 11-25 complete
- [x] All array/set operators covered
- [x] Logical combinations demonstrated
- [x] Real-world use cases realistic
- [x] All examples compile/work
- [x] README cross-referenced

---

## Next

Day 6: Embedding integration guide
