# Week 34 Day 4: Filter Examples (Basic)

**Date:** 2026-01-23
**Focus:** Create filter examples document (basic & comparison)
**Hours:** 2h
**Status:** [ ] PENDING

---

## Objectives

Create the first half of the filter examples document with basic and comparison filters.

---

## Tasks

### W34.2.1: Basic & Comparison Examples (2h)

**Goal:** Examples 1-10 of the filter examples document.

**Subtasks:**

- [ ] **4.1** Create document structure (15min)
  - Create `docs/guides/FILTER_EXAMPLES.md`
  - Add introduction and overview
  - Set up section structure

- [ ] **4.2** Basic filter examples (45min)
  - Example 1: Equality (`eq`)
  - Example 2: Inequality (`ne`)
  - Example 3: Greater than (`gt`)
  - Example 4: Less than (`lt`)
  - Example 5: Greater than or equal (`ge`)
  - Example 6: Less than or equal (`le`)
  - Example 7: Between range (`between`)

- [ ] **4.3** String filter examples (30min)
  - Example 8: Contains (`contains`)
  - Example 9: Starts with (`startsWith`)
  - Example 10: Ends with (`endsWith`)

- [ ] **4.4** Verify all examples compile (30min)
  - Test each example in isolation
  - Ensure both string syntax and functional API shown
  - Fix any issues

---

## Document Template

```markdown
# EdgeVec Filter Examples

A comprehensive collection of copy-paste ready filter examples for EdgeVec.

## Table of Contents

1. [Basic Filters](#basic-filters)
2. [String Filters](#string-filters)
3. [Array/Set Filters](#arrayset-filters)
4. [Null Filters](#null-filters)
5. [Logical Combinations](#logical-combinations)
6. [Real-World Examples](#real-world-examples)

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

**Use Case:** Product search by category, user filtering by role, document type filtering.

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

**Use Case:** Exclude deleted items, filter out specific categories.

---
```

---

## Example Format

Each example should include:

1. **Title** — Clear, descriptive name
2. **Description** — What the filter does
3. **String Syntax** — Using filter string
4. **Functional API** — Using `eq`, `gt`, etc.
5. **Use Case** — Real-world application

---

## Verification

- [ ] Document created at `docs/guides/FILTER_EXAMPLES.md`
- [ ] Examples 1-10 complete
- [ ] Both syntax styles shown for each
- [ ] All examples compile/work
- [ ] Use cases provided

---

## Next

Day 5: Advanced filter examples (11-25)
