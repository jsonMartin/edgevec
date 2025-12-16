# Week 22, Day 1: Query Syntax Design & EBNF Grammar

**Date:** 2025-12-17
**Sprint:** Week 22 (v0.5.0 Phase)
**Day Theme:** EBNF Grammar & Formal Query Syntax
**Status:** PLANNED

---

## Task W22.1: Query Syntax Design & EBNF Grammar

**Priority:** CRITICAL (P0)
**Estimated Effort:** 8 hours (3x rule: 2.5h optimistic × 3 = 7.5h + 0.5h buffer)
**Status:** PLANNED
**Depends On:** GATE_W21_COMPLETE.md exists
**Blocks:** W22.2, W22.3, W22.4, W22.5

---

### Context

Week 22 is a **DESIGN SPRINT** - NO implementation code allowed. Day 1 establishes the formal query syntax using EBNF grammar. This is the contract that the parser, evaluator, and all documentation will be built against in Week 23.

**Strategic Importance:**
- EBNF grammar is the FOUNDATION for Week 23 parser implementation
- Syntax decisions made today affect all future filtering features
- This is a non-negotiable prerequisite for filtering API

**Reference Documents:**
- [GATE_W21_COMPLETE.md](../../../.claude/GATE_W21_COMPLETE.md)
- [METADATA_SCHEMA_V1.md](../../schemas/METADATA_SCHEMA_V1.md) (FROZEN)
- [V0.5.0_STRATEGIC_ROADMAP.md](../V0.5.0_STRATEGIC_ROADMAP.md)

**Industry Research (Completed 2025-12-17):**
- Pinecone: MongoDB-style JSON (`$eq`, `$gt`, `$in`, `$and`)
- Milvus: SQL-like string expressions (has formal EBNF)
- Qdrant: Structured JSON (`must`, `should`, `must_not`)
- Weaviate: GraphQL with `where` clause

**Decision:** SQL-like syntax (Milvus pattern) - most familiar to developers, has formal EBNF

---

### Objective

Create `docs/architecture/FILTERING_SYNTAX.md` with:
1. Complete EBNF grammar specification (30+ rules)
2. Operator precedence table (6+ levels)
3. 20+ example queries with expected AST output
4. Type coercion rules for all 5 metadata types
5. Error message catalog

---

### Technical Approach

#### 1. EBNF Grammar Structure

**Target Grammar Complexity:**
- Max rules: 50
- Max nesting depth: 5 levels
- Max alternations per rule: 7

**Core Grammar Rules (Minimum 30):**
```ebnf
(* Top-level *)
filter_expr     = logical_expr ;

(* Logical operators *)
logical_expr    = or_expr ;
or_expr         = and_expr , { "OR" , and_expr } ;
and_expr        = not_expr , { "AND" , not_expr } ;
not_expr        = [ "NOT" ] , primary_expr ;

(* Primary expressions *)
primary_expr    = comparison_expr | "(" , logical_expr , ")" ;

(* Comparisons *)
comparison_expr = field , comp_op , value
                | field , string_op , string_value
                | field , array_op , array_value
                | field , null_check ;

(* Operators *)
comp_op         = "=" | "!=" | "<" | "<=" | ">" | ">=" ;
string_op       = "CONTAINS" | "STARTS_WITH" | "ENDS_WITH" ;
array_op        = "IN" | "NOT IN" | "ANY" | "ALL" ;
null_check      = "IS NULL" | "IS NOT NULL" ;

(* Values *)
field           = identifier ;
identifier      = letter , { letter | digit | "_" } ;
value           = string_value | number_value | boolean_value ;
string_value    = '"' , { char } , '"' ;
number_value    = integer_value | float_value ;
integer_value   = [ "-" ] , digit , { digit } ;
float_value     = [ "-" ] , digit , { digit } , "." , digit , { digit } ;
boolean_value   = "true" | "false" ;
array_value     = "[" , value , { "," , value } , "]" ;

(* Tokens *)
letter          = "a" | ... | "z" | "A" | ... | "Z" ;
digit           = "0" | ... | "9" ;
char            = ? any character except '"' or '\\' ?
                | escape_seq ;
escape_seq      = '\\' , ( '"' | '\\' | 'n' | 't' ) ;
```

#### 2. Operator Precedence Table

| Level | Operators | Associativity | Description |
|:------|:----------|:--------------|:------------|
| 1 (lowest) | `OR` | Left | Logical disjunction |
| 2 | `AND` | Left | Logical conjunction |
| 3 | `NOT` | Right (prefix) | Logical negation |
| 4 | `=`, `!=`, `<`, `<=`, `>`, `>=` | None | Comparison |
| 5 | `CONTAINS`, `STARTS_WITH`, `ENDS_WITH` | None | String operations |
| 6 (highest) | `IN`, `NOT IN`, `IS NULL`, `IS NOT NULL` | None | Set/null operations |

#### 3. Required Query Examples (20+)

**Simple Equality (4):**
```
category = "electronics"
price = 100
is_active = true
rating = 4.5
```

**Numeric Comparisons (4):**
```
price >= 100
price < 500
year > 2020
rating <= 3.0
```

**String Operations (3):**
```
title CONTAINS "NVIDIA"
name STARTS_WITH "Dr."
email ENDS_WITH "@example.com"
```

**Array Operations (3):**
```
tags IN ["gpu", "cuda"]
category NOT IN ["deprecated", "archived"]
features ANY ["bluetooth", "wifi"]
```

**Null Checks (2):**
```
description IS NOT NULL
optional_field IS NULL
```

**Complex Nested (4+):**
```
(category = "gpu" OR category = "tpu") AND price < 1000
NOT (status = "draft") AND published = true
price >= 100 AND price < 500 AND category = "electronics"
(tags IN ["premium"] OR rating >= 4.5) AND is_active = true
```

#### 4. Type Coercion Rules

| Left Type | Operator | Right Type | Result | Notes |
|:----------|:---------|:-----------|:-------|:------|
| Integer | `=`, `<`, etc. | Float | Compare as Float | Implicit upcast |
| Float | `=`, `<`, etc. | Integer | Compare as Float | Implicit upcast |
| String | `IN` | StringArray | Check membership | Valid |
| Integer | `IN` | StringArray | ERROR | Type mismatch |
| Any | `IS NULL` | - | Boolean | Always valid |

---

### Deliverables

1. **`docs/architecture/FILTERING_SYNTAX.md`** containing:
   - Complete EBNF grammar (30+ rules)
   - Operator precedence table (6+ levels)
   - Escape sequences documentation
   - Reserved keywords list
   - Error message catalog (10+ errors)
   - 20+ example queries with AST output

2. **`docs/architecture/FILTERING_SYNTAX.pest`** (optional)
   - Pest-compatible grammar for validation
   - Must parse without errors

---

### Acceptance Criteria

**CRITICAL (Must Pass):**
- [ ] EBNF grammar contains minimum 30 rules
- [ ] All 5 metadata types have at least 3 operators defined each
- [ ] Operator precedence table with 6+ levels documented
- [ ] 20+ example queries with expected AST output (JSON format)
- [ ] Grammar complexity within limits (max 50 rules, 5 nesting levels)

**MAJOR (Should Pass):**
- [ ] Edge cases documented: empty strings, MAX_INT, MIN_INT, NaN rejection, 64KB string
- [ ] Reserved keywords list complete
- [ ] Escape sequences for strings documented (`\"`, `\\`, `\n`, `\t`)
- [ ] Type coercion rules for all type pairs

**BINARY CHECKS:**
- [ ] Grammar file parses without errors via `pest_meta::validate` (if .pest created)
- [ ] All Rust code examples pass `cargo fmt --check`
- [ ] All vector references specify dimensionality (100k × 384-dim)

---

### Competitive Analysis Verification

**Sources (MUST document for each competitor):**

| Competitor | Primary Source | Secondary Source | Verified |
|:-----------|:---------------|:-----------------|:---------|
| Pinecone | [Official Docs](https://docs.pinecone.io/docs/metadata-filtering) | GitHub Issues | [ ] |
| Milvus | [Boolean.md](https://milvus.io/docs/boolean.md) | Source Code | [ ] |
| Qdrant | [Filtering Docs](https://qdrant.tech/documentation/concepts/filtering/) | Source Code | [ ] |
| Weaviate | [GraphQL Filters](https://weaviate.io/developers/weaviate/api/graphql/filters) | GraphQL Schema | [ ] |

---

### Implementation Checklist

- [ ] Create `docs/architecture/FILTERING_SYNTAX.md`
- [ ] Write EBNF grammar (30+ rules)
- [ ] Create operator precedence table
- [ ] Document escape sequences
- [ ] List reserved keywords
- [ ] Write 20+ example queries
- [ ] Document AST output format for each example
- [ ] Write type coercion rules table
- [ ] Create error message catalog
- [ ] (Optional) Create `FILTERING_SYNTAX.pest` for validation
- [ ] Verify grammar is unambiguous
- [ ] Cross-reference with competitor analysis

---

### Documentation Requirements

- [ ] File header with version, date, status
- [ ] Table of contents
- [ ] EBNF notation explanation for readers
- [ ] Examples section with copy-pasteable queries
- [ ] Error catalog with causes and fixes
- [ ] Links to related documents

---

### Dependencies

**Blocks:**
- W22.2 (Evaluator needs grammar defined)
- W22.3 (Strategy needs operators defined)
- W22.4 (WASM API needs syntax defined)
- W22.5 (Test strategy needs grammar for test cases)

**Blocked By:**
- GATE_W21_COMPLETE.md ✅
- METADATA_SCHEMA_V1.md (FROZEN) ✅

**External Dependencies:**
- None (design document only)

---

### Verification Method

**Day 1 is COMPLETE when:**

1. `docs/architecture/FILTERING_SYNTAX.md` exists
2. File contains 30+ EBNF grammar rules
3. All 15 operators have grammar rules
4. 20+ example queries documented
5. Type coercion rules table complete
6. Error catalog has 10+ error types

**Verification Commands:**
```bash
# Check file exists
test -f docs/architecture/FILTERING_SYNTAX.md && echo "PASS" || echo "FAIL"

# Check grammar rule count (grep for '=' assignments in EBNF)
grep -c "=" docs/architecture/FILTERING_SYNTAX.md

# Optional: Validate pest grammar
# pest_meta::validate(grammar_file)
```

---

### Estimated Timeline

| Phase | Time | Cumulative |
|:------|:-----|:-----------|
| Research review & outline | 0.5h | 0.5h |
| EBNF grammar writing | 2.5h | 3h |
| Operator precedence table | 0.5h | 3.5h |
| Example queries (20+) | 1.5h | 5h |
| Type coercion rules | 0.5h | 5.5h |
| Error catalog | 1h | 6.5h |
| Documentation polish | 1h | 7.5h |
| Buffer | 0.5h | 8h |

---

### Hostile Review Checkpoint

**End of Day 1:** Submit for `/review` with:
- `docs/architecture/FILTERING_SYNTAX.md`

**Expected Review Focus:**
- Grammar completeness and unambiguity
- Operator coverage for all 5 types
- Example query correctness
- Type coercion edge cases
- Error message clarity

---

### Rollback Plan

If Day 1 encounters blocking issues:

1. **Grammar too complex:** Simplify by removing less common operators (ANY, ALL)
2. **Ambiguity found:** Add explicit precedence rules or parentheses requirements
3. **Type coercion conflicts:** Make all coercions explicit (no implicit conversions)
4. **Pest validation fails:** Proceed with EBNF only, fix pest in Week 23

---

**Task Owner:** META_ARCHITECT
**Review Required:** HOSTILE_REVIEWER
**Next Task:** W22.2 (Filter Evaluator Architecture)

---

*"A grammar well-defined is a parser half-written."*
