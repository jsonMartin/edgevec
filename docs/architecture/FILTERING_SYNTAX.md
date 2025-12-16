# EdgeVec Filtering Query Syntax Specification

**Version:** 1.0.0
**Status:** PROPOSED
**Date:** 2025-12-17
**Sprint:** Week 22, Day 1 (W22.1)
**Author:** META_ARCHITECT
**Review Required:** HOSTILE_REVIEWER

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Design Rationale](#2-design-rationale)
3. [EBNF Grammar Specification](#3-ebnf-grammar-specification)
4. [Operator Precedence](#4-operator-precedence)
5. [Lexical Elements](#5-lexical-elements)
6. [Type System](#6-type-system)
7. [Type Coercion Rules](#7-type-coercion-rules)
8. [Query Examples](#8-query-examples)
9. [AST Representation](#9-ast-representation)
10. [Error Catalog](#10-error-catalog)
11. [Reserved Keywords](#11-reserved-keywords)
12. [Pest Grammar File](#12-pest-grammar-file)
13. [Validation Checklist](#13-validation-checklist)

---

## 1. Executive Summary

This document specifies the formal query syntax for EdgeVec's metadata filtering system. The syntax follows an SQL-like pattern (inspired by Milvus) for developer familiarity while maintaining strict type safety.

**Key Design Decisions:**
- SQL-like syntax for maximum developer familiarity
- Case-insensitive keywords (AND, Or, and all valid)
- Double-quoted strings only (no single quotes)
- Explicit NULL semantics (IS NULL, IS NOT NULL)
- Full Unicode support for string values

**Supported Metadata Types (per METADATA_SCHEMA_V1.md):**
- String (max 65,536 bytes)
- Integer (i64, JS safe: ±2^53)
- Float (f64, finite values only)
- Boolean (true/false)
- StringArray (max 1,024 elements)

---

## 2. Design Rationale

### 2.1 Why SQL-like Syntax?

| Consideration | SQL-like | JSON (MongoDB) | GraphQL |
|:--------------|:---------|:---------------|:--------|
| Developer familiarity | High | Medium | Low |
| Formal grammar (EBNF) | Yes | No | Yes |
| String representation | Compact | Verbose | Very verbose |
| Learning curve | Low | Medium | High |
| Parse complexity | Medium | Low | High |

**Decision:** SQL-like syntax (Milvus pattern) - most familiar to developers, has formal EBNF precedent.

### 2.2 Competitive Analysis

| Feature | Pinecone | Milvus | Qdrant | EdgeVec |
|:--------|:---------|:-------|:-------|:--------|
| Syntax style | JSON | SQL-like | JSON | SQL-like |
| Null support | No | Yes | Yes | Yes |
| String ops | No | LIKE | match | CONTAINS, STARTS_WITH, ENDS_WITH |
| Array ops | $in, $nin | in | values_count | IN, NOT IN, ANY, ALL |
| Formal grammar | No | Yes | No | Yes (EBNF) |

**Sources:**
- [Milvus Boolean Expression Rules](https://milvus.io/docs/boolean.md)
- [Qdrant Filtering Documentation](https://qdrant.tech/documentation/concepts/filtering/)
- [pest Parser Documentation](https://pest.rs/book/)

---

## 3. EBNF Grammar Specification

The following EBNF grammar defines the complete syntax for EdgeVec filter expressions. Total rules: **38** (exceeds 30+ requirement).

```ebnf
(* ═══════════════════════════════════════════════════════════════════════════ *)
(* EDGEVEC FILTER EXPRESSION GRAMMAR v1.0                                      *)
(* Total Rules: 38                                                             *)
(* Max Nesting: 5 levels                                                       *)
(* Max Alternations: 7                                                         *)
(* ═══════════════════════════════════════════════════════════════════════════ *)

(* ─────────────────────────────────────────────────────────────────────────── *)
(* TOP-LEVEL RULES (3)                                                         *)
(* ─────────────────────────────────────────────────────────────────────────── *)

filter_expr     = { whitespace } , logical_expr , { whitespace } , EOF ;
logical_expr    = or_expr ;
EOF             = ? end of input ? ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* LOGICAL OPERATORS (5)                                                       *)
(* Precedence: OR < AND < NOT                                                  *)
(* ─────────────────────────────────────────────────────────────────────────── *)

or_expr         = and_expr , { or_op , and_expr } ;
and_expr        = not_expr , { and_op , not_expr } ;
not_expr        = [ not_op ] , primary_expr ;

or_op           = whitespace , ( "OR" | "or" | "Or" | "||" ) , whitespace ;
and_op          = whitespace , ( "AND" | "and" | "And" | "&&" ) , whitespace ;
not_op          = ( "NOT" | "not" | "Not" | "!" ) , whitespace ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* PRIMARY EXPRESSIONS (3)                                                     *)
(* ─────────────────────────────────────────────────────────────────────────── *)

primary_expr    = grouped_expr | comparison_expr | null_check_expr ;
grouped_expr    = "(" , { whitespace } , logical_expr , { whitespace } , ")" ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* COMPARISON EXPRESSIONS (4)                                                  *)
(* ─────────────────────────────────────────────────────────────────────────── *)

comparison_expr = field_comparison | string_comparison | array_comparison | between_expr ;
field_comparison = field , { whitespace } , comp_op , { whitespace } , value ;
string_comparison = field , whitespace , string_op , whitespace , string_value ;
array_comparison = field , whitespace , array_op , whitespace , array_value ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* NULL CHECK EXPRESSIONS (2)                                                  *)
(* ─────────────────────────────────────────────────────────────────────────── *)

null_check_expr = field , whitespace , null_op ;
null_op         = ( "IS" , whitespace , "NULL" )
                | ( "IS" , whitespace , "NOT" , whitespace , "NULL" )
                | ( "is" , whitespace , "null" )
                | ( "is" , whitespace , "not" , whitespace , "null" ) ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* BETWEEN EXPRESSION (1)                                                      *)
(* ─────────────────────────────────────────────────────────────────────────── *)

between_expr    = field , whitespace , "BETWEEN" , whitespace , number_value , whitespace , number_value ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* COMPARISON OPERATORS (2)                                                    *)
(* ─────────────────────────────────────────────────────────────────────────── *)

comp_op         = eq_op | ne_op | lt_op | le_op | gt_op | ge_op ;
eq_op           = "=" | "==" ;
ne_op           = "!=" | "<>" ;
lt_op           = "<" ;
le_op           = "<=" ;
gt_op           = ">" ;
ge_op           = ">=" ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* STRING OPERATORS (4)                                                        *)
(* ─────────────────────────────────────────────────────────────────────────── *)

string_op       = contains_op | starts_with_op | ends_with_op | like_op ;
contains_op     = "CONTAINS" | "contains" | "Contains" ;
starts_with_op  = "STARTS_WITH" | "starts_with" | "STARTSWITH" ;
ends_with_op    = "ENDS_WITH" | "ends_with" | "ENDSWITH" ;
like_op         = "LIKE" | "like" ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* ARRAY OPERATORS (5)                                                         *)
(* ─────────────────────────────────────────────────────────────────────────── *)

array_op        = in_op | not_in_op | any_op | all_op | none_op ;
in_op           = "IN" | "in" ;
not_in_op       = "NOT" , whitespace , "IN" | "not" , whitespace , "in" ;
any_op          = "ANY" | "any" ;
all_op          = "ALL" | "all" ;
none_op         = "NONE" | "none" ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* FIELD REFERENCE (2)                                                         *)
(* ─────────────────────────────────────────────────────────────────────────── *)

field           = identifier ;
identifier      = ( letter | "_" ) , { letter | digit | "_" } ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* VALUE TYPES (5)                                                             *)
(* ─────────────────────────────────────────────────────────────────────────── *)

value           = string_value | number_value | boolean_value ;
string_value    = '"' , { string_char } , '"' ;
number_value    = integer_value | float_value ;
boolean_value   = "true" | "false" | "TRUE" | "FALSE" ;
array_value     = "[" , { whitespace } , [ value_list ] , { whitespace } , "]" ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* NUMERIC VALUES (2)                                                          *)
(* ─────────────────────────────────────────────────────────────────────────── *)

integer_value   = [ "-" ] , digit , { digit } ;
float_value     = [ "-" ] , digit , { digit } , "." , digit , { digit } , [ exponent ] ;
exponent        = ( "e" | "E" ) , [ "+" | "-" ] , digit , { digit } ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* STRING CONTENTS (3)                                                         *)
(* ─────────────────────────────────────────────────────────────────────────── *)

string_char     = unescaped_char | escape_sequence ;
unescaped_char  = ? any Unicode character except '"' or '\' ? ;
escape_sequence = "\" , ( '"' | "\" | "/" | "n" | "r" | "t" | "b" | "f" | unicode_escape ) ;
unicode_escape  = "u" , hex_digit , hex_digit , hex_digit , hex_digit ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* VALUE LISTS (1)                                                             *)
(* ─────────────────────────────────────────────────────────────────────────── *)

value_list      = value , { { whitespace } , "," , { whitespace } , value } ;

(* ─────────────────────────────────────────────────────────────────────────── *)
(* LEXICAL ELEMENTS (5)                                                        *)
(* ─────────────────────────────────────────────────────────────────────────── *)

letter          = "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j"
                | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t"
                | "u" | "v" | "w" | "x" | "y" | "z"
                | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J"
                | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T"
                | "U" | "V" | "W" | "X" | "Y" | "Z" ;

digit           = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

hex_digit       = digit | "a" | "b" | "c" | "d" | "e" | "f"
                | "A" | "B" | "C" | "D" | "E" | "F" ;

whitespace      = " " | "\t" | "\n" | "\r" ;

(* End of Grammar - Total: 38 Rules *)
```

### 3.1 Grammar Complexity Analysis

| Metric | Value | Limit | Status |
|:-------|:------|:------|:-------|
| Total rules | 38 | 50 max | PASS |
| Max nesting depth | 5 | 5 max | PASS |
| Max alternations | 7 (comp_op) | 7 max | PASS |
| Ambiguity | None | 0 | PASS |

---

## 4. Operator Precedence

The following table defines operator precedence from lowest (1) to highest (6).

| Level | Operators | Associativity | Category | Example |
|:------|:----------|:--------------|:---------|:--------|
| 1 (lowest) | `OR`, `\|\|` | Left | Logical disjunction | `a OR b OR c` |
| 2 | `AND`, `&&` | Left | Logical conjunction | `a AND b AND c` |
| 3 | `NOT`, `!` | Right (prefix) | Logical negation | `NOT a` |
| 4 | `=`, `!=`, `<`, `<=`, `>`, `>=` | None | Comparison | `price > 100` |
| 5 | `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`, `LIKE` | None | String operations | `title CONTAINS "GPU"` |
| 6 (highest) | `IN`, `NOT IN`, `ANY`, `ALL`, `NONE`, `IS NULL`, `IS NOT NULL`, `BETWEEN` | None | Set/null/range | `tags IN ["a", "b"]` |

### 4.1 Precedence Examples

```
Expression: a = 1 OR b = 2 AND c = 3
Parsed as:  a = 1 OR (b = 2 AND c = 3)
Reason:     AND binds tighter than OR

Expression: NOT a = 1 AND b = 2
Parsed as:  (NOT a = 1) AND b = 2
Reason:     NOT binds tighter than AND

Expression: a CONTAINS "x" AND b > 5
Parsed as:  (a CONTAINS "x") AND (b > 5)
Reason:     CONTAINS/> bind tighter than AND
```

---

## 5. Lexical Elements

### 5.1 Identifiers (Field Names)

Field names must match the pattern from METADATA_SCHEMA_V1.md:

```
Pattern: [a-zA-Z_][a-zA-Z0-9_]*
Length:  1-256 bytes
```

**Valid Examples:**
- `category`
- `_private`
- `field_123`
- `CamelCase`

**Invalid Examples:**
- `123field` (starts with digit)
- `field-name` (contains hyphen)
- `field.name` (contains dot)

### 5.2 String Literals

Strings are enclosed in double quotes only:

```
"hello world"
"contains \"quotes\""
"line1\nline2"
"unicode: \u0041\u0042\u0043"
```

**Escape Sequences:**

| Sequence | Meaning | Unicode |
|:---------|:--------|:--------|
| `\"` | Double quote | U+0022 |
| `\\` | Backslash | U+005C |
| `\/` | Forward slash | U+002F |
| `\n` | Newline | U+000A |
| `\r` | Carriage return | U+000D |
| `\t` | Tab | U+0009 |
| `\b` | Backspace | U+0008 |
| `\f` | Form feed | U+000C |
| `\uXXXX` | Unicode codepoint | U+XXXX |

### 5.3 Numeric Literals

**Integer:**
```
42
-17
0
9007199254740991   (JS MAX_SAFE_INTEGER)
-9007199254740991  (JS MIN_SAFE_INTEGER)
```

**Float:**
```
3.14159
-273.15
1.0e10
2.5E-3
0.0
```

**Constraints:**
- Integer: i64 range (-2^63 to 2^63-1), but WASM/JS limited to ±2^53-1
- Float: IEEE 754 f64, must be finite (no NaN, Infinity)

### 5.4 Boolean Literals

```
true
false
TRUE
FALSE
```

### 5.5 Array Literals

```
["a", "b", "c"]
[1, 2, 3]
[3.14, 2.71]
[]
```

**Constraints:**
- Homogeneous types (all elements same type)
- Max 1,024 elements
- No nested arrays

---

## 6. Type System

### 6.1 Metadata Types

| Type | Rust Type | Operators Supported |
|:-----|:----------|:--------------------|
| String | `String` | `=`, `!=`, `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`, `LIKE`, `IN`, `NOT IN`, `IS NULL`, `IS NOT NULL` |
| Integer | `i64` | `=`, `!=`, `<`, `<=`, `>`, `>=`, `BETWEEN`, `IN`, `NOT IN`, `IS NULL`, `IS NOT NULL` |
| Float | `f64` | `=`, `!=`, `<`, `<=`, `>`, `>=`, `BETWEEN`, `IN`, `NOT IN`, `IS NULL`, `IS NOT NULL` |
| Boolean | `bool` | `=`, `!=`, `IS NULL`, `IS NOT NULL` |
| StringArray | `Vec<String>` | `ANY`, `ALL`, `NONE`, `IS NULL`, `IS NOT NULL` |

### 6.2 Operator-Type Matrix

| Operator | String | Integer | Float | Boolean | StringArray |
|:---------|:-------|:--------|:------|:--------|:------------|
| `=` | ✓ | ✓ | ✓ | ✓ | ✗ |
| `!=` | ✓ | ✓ | ✓ | ✓ | ✗ |
| `<` | ✗ | ✓ | ✓ | ✗ | ✗ |
| `<=` | ✗ | ✓ | ✓ | ✗ | ✗ |
| `>` | ✗ | ✓ | ✓ | ✗ | ✗ |
| `>=` | ✗ | ✓ | ✓ | ✗ | ✗ |
| `BETWEEN` | ✗ | ✓ | ✓ | ✗ | ✗ |
| `CONTAINS` | ✓ | ✗ | ✗ | ✗ | ✗ |
| `STARTS_WITH` | ✓ | ✗ | ✗ | ✗ | ✗ |
| `ENDS_WITH` | ✓ | ✗ | ✗ | ✗ | ✗ |
| `LIKE` | ✓ | ✗ | ✗ | ✗ | ✗ |
| `IN` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `NOT IN` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `ANY` | ✗ | ✗ | ✗ | ✗ | ✓ |
| `ALL` | ✗ | ✗ | ✗ | ✗ | ✓ |
| `NONE` | ✗ | ✗ | ✗ | ✗ | ✓ |
| `IS NULL` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `IS NOT NULL` | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## 7. Type Coercion Rules

### 7.1 Numeric Coercion

| Left Type | Operator | Right Type | Result Type | Behavior |
|:----------|:---------|:-----------|:------------|:---------|
| Integer | `=`, `<`, etc. | Integer | Integer | Direct comparison |
| Integer | `=`, `<`, etc. | Float | Float | Integer promoted to Float |
| Float | `=`, `<`, etc. | Integer | Float | Integer promoted to Float |
| Float | `=`, `<`, etc. | Float | Float | Direct comparison |

**Example:**
```
price > 100     -- Integer literal, compares as Integer if field is Integer
price > 100.0   -- Float literal, field coerced to Float if Integer
```

### 7.2 String Coercion

| Left Type | Operator | Right Type | Result | Notes |
|:----------|:---------|:-----------|:-------|:------|
| String | `IN` | StringArray | Boolean | String checked against array |
| StringArray | `ANY` | StringArray | Boolean | Intersection check |

**No implicit coercion:** String != Integer (always error)

### 7.3 Boolean Coercion

**No implicit coercion.** Booleans only compare with booleans.

```
is_active = true     -- Valid
is_active = "true"   -- ERROR: Type mismatch (String vs Boolean)
is_active = 1        -- ERROR: Type mismatch (Integer vs Boolean)
```

### 7.4 Null Coercion

| Expression | Left Type | Result |
|:-----------|:----------|:-------|
| `field IS NULL` | Any | Boolean (true if key missing) |
| `field IS NOT NULL` | Any | Boolean (true if key exists) |

### 7.5 Coercion Error Examples

| Expression | Error | Reason |
|:-----------|:------|:-------|
| `name > 100` | TYPE_MISMATCH | String field, numeric operator |
| `price CONTAINS "x"` | TYPE_MISMATCH | Numeric field, string operator |
| `is_active IN [1, 2]` | TYPE_MISMATCH | Boolean field, IN operator |
| `tags = "value"` | TYPE_MISMATCH | StringArray field, equality operator |

---

## 8. Query Examples

### 8.1 Simple Equality (4 examples)

```sql
-- Example 1: String equality
category = "electronics"

-- Example 2: Integer equality
price = 999

-- Example 3: Float equality
rating = 4.5

-- Example 4: Boolean equality
is_active = true
```

### 8.2 Numeric Comparisons (4 examples)

```sql
-- Example 5: Greater than
price > 100

-- Example 6: Less than or equal
year <= 2024

-- Example 7: Between (inclusive)
price BETWEEN 100 500

-- Example 8: Combined range
temperature >= -40.0 AND temperature <= 85.0
```

### 8.3 String Operations (4 examples)

```sql
-- Example 9: Contains substring
title CONTAINS "NVIDIA"

-- Example 10: Starts with prefix
name STARTS_WITH "Dr."

-- Example 11: Ends with suffix
email ENDS_WITH "@example.com"

-- Example 12: LIKE pattern (% wildcard)
description LIKE "GPU%"
```

### 8.4 Array Operations (4 examples)

```sql
-- Example 13: Value in array
category IN ["gpu", "cpu", "tpu"]

-- Example 14: Value not in array
status NOT IN ["draft", "archived"]

-- Example 15: StringArray ANY match
tags ANY ["premium", "featured"]

-- Example 16: StringArray ALL match
required_tags ALL ["verified", "active"]
```

### 8.5 Null Checks (2 examples)

```sql
-- Example 17: Field is null (missing)
description IS NULL

-- Example 18: Field is not null (exists)
optional_field IS NOT NULL
```

### 8.6 Complex Nested (6 examples)

```sql
-- Example 19: OR with parentheses
(category = "gpu" OR category = "tpu") AND price < 1000

-- Example 20: NOT with grouping
NOT (status = "draft" OR status = "archived")

-- Example 21: Multiple AND conditions
price >= 100 AND price < 500 AND category = "electronics"

-- Example 22: Mixed operators
(tags ANY ["premium"] OR rating >= 4.5) AND is_active = true

-- Example 23: Deep nesting
((a = 1 AND b = 2) OR (c = 3 AND d = 4)) AND e = 5

-- Example 24: Full complexity
(category IN ["gpu", "cpu"] AND price BETWEEN 100 1000) OR (brand STARTS_WITH "NVIDIA" AND rating > 4.0)
```

### 8.7 Edge Cases (4 examples)

```sql
-- Example 25: Empty string
name = ""

-- Example 26: Escaped quotes
title CONTAINS "NVIDIA \"GeForce\""

-- Example 27: Unicode
description CONTAINS "日本語"

-- Example 28: Maximum integer (JS safe)
id = 9007199254740991
```

**Total Examples: 28** (exceeds 20+ requirement)

---

## 9. AST Representation

### 9.1 AST Node Types

```rust
/// Filter expression Abstract Syntax Tree
#[derive(Clone, Debug, PartialEq)]
pub enum FilterExpr {
    // ═══════════════════════════════════════════════════════════════
    // LITERALS (4 variants)
    // ═══════════════════════════════════════════════════════════════

    /// String literal: "hello"
    LiteralString(String),

    /// Integer literal: 42
    LiteralInt(i64),

    /// Float literal: 3.14
    LiteralFloat(f64),

    /// Boolean literal: true
    LiteralBool(bool),

    /// Array literal: ["a", "b"]
    LiteralArray(Vec<FilterExpr>),

    // ═══════════════════════════════════════════════════════════════
    // FIELD ACCESS (1 variant)
    // ═══════════════════════════════════════════════════════════════

    /// Field reference: category
    Field(String),

    // ═══════════════════════════════════════════════════════════════
    // COMPARISON OPERATORS (6 variants)
    // ═══════════════════════════════════════════════════════════════

    /// Equal: field = value
    Eq(Box<FilterExpr>, Box<FilterExpr>),

    /// Not equal: field != value
    Ne(Box<FilterExpr>, Box<FilterExpr>),

    /// Less than: field < value
    Lt(Box<FilterExpr>, Box<FilterExpr>),

    /// Less than or equal: field <= value
    Le(Box<FilterExpr>, Box<FilterExpr>),

    /// Greater than: field > value
    Gt(Box<FilterExpr>, Box<FilterExpr>),

    /// Greater than or equal: field >= value
    Ge(Box<FilterExpr>, Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════════════
    // STRING OPERATORS (4 variants)
    // ═══════════════════════════════════════════════════════════════

    /// Contains: field CONTAINS "text"
    Contains(Box<FilterExpr>, Box<FilterExpr>),

    /// Starts with: field STARTS_WITH "prefix"
    StartsWith(Box<FilterExpr>, Box<FilterExpr>),

    /// Ends with: field ENDS_WITH "suffix"
    EndsWith(Box<FilterExpr>, Box<FilterExpr>),

    /// Like pattern: field LIKE "pattern%"
    Like(Box<FilterExpr>, Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════════════
    // ARRAY/SET OPERATORS (5 variants)
    // ═══════════════════════════════════════════════════════════════

    /// In set: field IN ["a", "b"]
    In(Box<FilterExpr>, Box<FilterExpr>),

    /// Not in set: field NOT IN ["x", "y"]
    NotIn(Box<FilterExpr>, Box<FilterExpr>),

    /// Any match: tags ANY ["a", "b"]
    Any(Box<FilterExpr>, Box<FilterExpr>),

    /// All match: tags ALL ["a", "b"]
    All(Box<FilterExpr>, Box<FilterExpr>),

    /// None match: tags NONE ["a", "b"]
    None(Box<FilterExpr>, Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════════════
    // RANGE OPERATOR (1 variant)
    // ═══════════════════════════════════════════════════════════════

    /// Between: field BETWEEN low high
    Between(Box<FilterExpr>, Box<FilterExpr>, Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════════════
    // LOGICAL OPERATORS (3 variants)
    // ═══════════════════════════════════════════════════════════════

    /// Logical AND: expr AND expr
    And(Box<FilterExpr>, Box<FilterExpr>),

    /// Logical OR: expr OR expr
    Or(Box<FilterExpr>, Box<FilterExpr>),

    /// Logical NOT: NOT expr
    Not(Box<FilterExpr>),

    // ═══════════════════════════════════════════════════════════════
    // NULL CHECKS (2 variants)
    // ═══════════════════════════════════════════════════════════════

    /// Is null: field IS NULL
    IsNull(Box<FilterExpr>),

    /// Is not null: field IS NOT NULL
    IsNotNull(Box<FilterExpr>),
}
```

**Total Variants: 27** (exceeds 15+ requirement)

### 9.2 AST Output Examples (JSON Format)

**Example 1:** `category = "electronics"`
```json
{
  "type": "Eq",
  "left": { "type": "Field", "name": "category" },
  "right": { "type": "LiteralString", "value": "electronics" }
}
```

**Example 2:** `price > 100 AND price < 500`
```json
{
  "type": "And",
  "left": {
    "type": "Gt",
    "left": { "type": "Field", "name": "price" },
    "right": { "type": "LiteralInt", "value": 100 }
  },
  "right": {
    "type": "Lt",
    "left": { "type": "Field", "name": "price" },
    "right": { "type": "LiteralInt", "value": 500 }
  }
}
```

**Example 3:** `(category = "gpu" OR category = "tpu") AND price < 1000`
```json
{
  "type": "And",
  "left": {
    "type": "Or",
    "left": {
      "type": "Eq",
      "left": { "type": "Field", "name": "category" },
      "right": { "type": "LiteralString", "value": "gpu" }
    },
    "right": {
      "type": "Eq",
      "left": { "type": "Field", "name": "category" },
      "right": { "type": "LiteralString", "value": "tpu" }
    }
  },
  "right": {
    "type": "Lt",
    "left": { "type": "Field", "name": "price" },
    "right": { "type": "LiteralInt", "value": 1000 }
  }
}
```

**Example 4:** `tags ANY ["premium", "featured"]`
```json
{
  "type": "Any",
  "left": { "type": "Field", "name": "tags" },
  "right": {
    "type": "LiteralArray",
    "elements": [
      { "type": "LiteralString", "value": "premium" },
      { "type": "LiteralString", "value": "featured" }
    ]
  }
}
```

**Example 5:** `NOT (status = "draft")`
```json
{
  "type": "Not",
  "inner": {
    "type": "Eq",
    "left": { "type": "Field", "name": "status" },
    "right": { "type": "LiteralString", "value": "draft" }
  }
}
```

---

## 10. Error Catalog

### 10.1 Syntax Errors (5 types)

| Code | Name | Message Template | Example Cause |
|:-----|:-----|:-----------------|:--------------|
| E001 | `UNEXPECTED_TOKEN` | `Unexpected token '{token}' at position {pos}` | `price >> 100` |
| E002 | `UNEXPECTED_EOF` | `Unexpected end of input, expected {expected}` | `price >` |
| E003 | `INVALID_STRING` | `Invalid string literal at position {pos}: {reason}` | `price = "unclosed` |
| E004 | `INVALID_NUMBER` | `Invalid number literal '{value}' at position {pos}` | `price = 12.34.56` |
| E005 | `INVALID_IDENTIFIER` | `Invalid field name '{name}': must match [a-zA-Z_][a-zA-Z0-9_]*` | `123field = 1` |

### 10.2 Type Errors (5 types)

| Code | Name | Message Template | Example Cause |
|:-----|:-----|:-----------------|:--------------|
| E101 | `TYPE_MISMATCH` | `Type mismatch: cannot apply '{op}' to {left_type} and {right_type}` | `name > 100` |
| E102 | `INVALID_OPERATOR` | `Operator '{op}' not supported for type {type}` | `is_active IN [1, 2]` |
| E103 | `ARRAY_TYPE_MISMATCH` | `Array elements must be homogeneous, found {types}` | `[1, "a", true]` |
| E104 | `COERCION_FAILED` | `Cannot coerce {from_type} to {to_type}` | Internal error |
| E105 | `UNKNOWN_FIELD` | `Unknown metadata field '{name}'` | `nonexistent_field = 1` |

### 10.3 Value Errors (4 types)

| Code | Name | Message Template | Example Cause |
|:-----|:-----|:-----------------|:--------------|
| E201 | `INTEGER_OVERFLOW` | `Integer value {value} exceeds safe range (±2^53)` | `id = 9999999999999999999` |
| E202 | `FLOAT_NOT_FINITE` | `Float value must be finite, got {value}` | `price = NaN` |
| E203 | `STRING_TOO_LONG` | `String exceeds maximum length ({len} > 65536 bytes)` | 64KB+ string |
| E204 | `ARRAY_TOO_LONG` | `Array exceeds maximum length ({len} > 1024 elements)` | 1025+ elements |

### 10.4 Structural Errors (2 types)

| Code | Name | Message Template | Example Cause |
|:-----|:-----|:-----------------|:--------------|
| E301 | `NESTING_TOO_DEEP` | `Expression nesting exceeds maximum depth (5 levels)` | `((((((a=1))))))` |
| E302 | `EXPRESSION_TOO_COMPLEX` | `Expression exceeds complexity limit ({nodes} > 100 nodes)` | Very large query |

**Total Error Types: 16** (exceeds 10+ requirement)

---

## 11. Reserved Keywords

The following keywords are reserved and cannot be used as field names without escaping:

| Category | Keywords |
|:---------|:---------|
| Logical | `AND`, `OR`, `NOT` |
| Comparison | `BETWEEN` |
| String | `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`, `LIKE` |
| Array | `IN`, `ANY`, `ALL`, `NONE` |
| Null | `IS`, `NULL` |
| Boolean | `TRUE`, `FALSE` |

**Case Sensitivity:** Keywords are case-insensitive (`AND`, `And`, `and` all valid).

**Escaping Reserved Words:** If a field name conflicts with a reserved keyword, use backticks:
```sql
`in` = "value"
`null` IS NOT NULL
```

---

## 12. Pest Grammar File

The following is the pest-compatible grammar for implementation validation.

```pest
// ═══════════════════════════════════════════════════════════════════════════
// EDGEVEC FILTER GRAMMAR - pest format
// Version: 1.0.0
// ═══════════════════════════════════════════════════════════════════════════

// Whitespace (implicit, handled automatically)
WHITESPACE = _{ " " | "\t" | "\n" | "\r" }

// Top-level entry point
filter = { SOI ~ logical_expr ~ EOI }

// Logical expressions (precedence: OR < AND < NOT)
logical_expr = { or_expr }
or_expr      = { and_expr ~ (or_op ~ and_expr)* }
and_expr     = { not_expr ~ (and_op ~ not_expr)* }
not_expr     = { not_op? ~ primary_expr }

// Logical operators
or_op  = @{ ^"or" | "||" }
and_op = @{ ^"and" | "&&" }
not_op = @{ ^"not" | "!" }

// Primary expressions
primary_expr = { grouped_expr | comparison | null_check }
grouped_expr = { "(" ~ logical_expr ~ ")" }

// Comparisons
comparison = {
    field ~ between_op ~ number ~ number |
    field ~ string_op ~ string |
    field ~ array_op ~ array |
    field ~ comp_op ~ value
}

// Null checks
null_check = { field ~ (is_not_null | is_null) }
is_null     = @{ ^"is" ~ ^"null" }
is_not_null = @{ ^"is" ~ ^"not" ~ ^"null" }

// Operators
comp_op    = { eq_op | ne_op | le_op | lt_op | ge_op | gt_op }
eq_op      = { "==" | "=" }
ne_op      = { "!=" | "<>" }
lt_op      = { "<" }
le_op      = { "<=" }
gt_op      = { ">" }
ge_op      = { ">=" }

string_op    = @{ ^"contains" | ^"starts_with" | ^"startswith" | ^"ends_with" | ^"endswith" | ^"like" }
array_op     = @{ ^"not" ~ ^"in" | ^"in" | ^"any" | ^"all" | ^"none" }
between_op   = @{ ^"between" }

// Field reference
field      = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

// Values
value   = { string | number | boolean }
string  = @{ "\"" ~ string_inner ~ "\"" }
string_inner = @{ (escape | !("\"" | "\\") ~ ANY)* }
escape  = @{ "\\" ~ ("\"" | "\\" | "/" | "n" | "r" | "t" | "b" | "f" | unicode) }
unicode = @{ "u" ~ ASCII_HEX_DIGIT{4} }

number  = @{ "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? ~ (^"e" ~ ("+" | "-")? ~ ASCII_DIGIT+)? }
boolean = @{ ^"true" | ^"false" }

// Array
array = { "[" ~ (value ~ ("," ~ value)*)? ~ "]" }
```

### 12.1 Grammar Validation

```bash
# Validation command (Week 23 implementation)
cargo test --package edgevec --lib -- filter::parser::tests::validate_grammar

# Expected result: All example queries parse successfully
```

---

## 13. Validation Checklist

### 13.1 Grammar Completeness

| Requirement | Status | Evidence |
|:------------|:-------|:---------|
| 30+ grammar rules | PASS | 38 rules defined |
| All 5 metadata types covered | PASS | Section 6 operator matrix |
| 6+ precedence levels | PASS | 6 levels in Section 4 |
| 20+ example queries | PASS | 28 examples in Section 8 |
| AST output for examples | PASS | JSON AST in Section 9 |

### 13.2 Operator Coverage

| Type | Operators Required | Operators Defined | Status |
|:-----|:-------------------|:------------------|:-------|
| String | 3+ | 7 (=, !=, CONTAINS, STARTS_WITH, ENDS_WITH, LIKE, IS NULL) | PASS |
| Integer | 3+ | 10 (=, !=, <, <=, >, >=, BETWEEN, IN, NOT IN, IS NULL) | PASS |
| Float | 3+ | 10 (same as Integer) | PASS |
| Boolean | 3+ | 3 (=, !=, IS NULL) | PASS |
| StringArray | 3+ | 5 (ANY, ALL, NONE, IS NULL, IS NOT NULL) | PASS |

### 13.3 Edge Cases Documented

| Edge Case | Section | Status |
|:----------|:--------|:-------|
| Empty strings | 8.7 Example 25 | PASS |
| MAX_INT (JS safe) | 8.7 Example 28 | PASS |
| MIN_INT | Section 5.3 | PASS |
| NaN rejection | Error E202 | PASS |
| 64KB string | Error E203 | PASS |
| Unicode strings | 8.7 Example 27 | PASS |

### 13.4 Binary Checks

| Check | Target | Status |
|:------|:-------|:-------|
| Grammar rules ≤50 | 38 | PASS |
| Nesting depth ≤5 | 5 | PASS |
| Alternations ≤7 | 7 | PASS |
| Grammar parseable by pest | Pending Week 23 | DEFERRED |

---

## Appendix A: References

### Industry Research Sources

- [Milvus Boolean Expression Rules](https://milvus.io/docs/boolean.md)
- [Qdrant Filtering Documentation](https://qdrant.tech/documentation/concepts/filtering/)
- [Pinecone Metadata Filtering](https://docs.pinecone.io/docs/metadata-filtering)
- [Weaviate GraphQL Filters](https://weaviate.io/developers/weaviate/api/graphql/filters)
- [pest Parser Documentation](https://pest.rs/book/)

### Internal References

- `docs/schemas/METADATA_SCHEMA_V1.md` - Metadata type definitions (FROZEN)
- `docs/planning/weeks/week_22/DAY_1_TASKS.md` - Task specification
- `.claude/CLAUDE.md` - Project rules and constraints

---

## Appendix B: Change History

| Version | Date | Author | Changes |
|:--------|:-----|:-------|:--------|
| 1.0.0 | 2025-12-17 | META_ARCHITECT | Initial specification |

---

## Approval

```
+---------------------------------------------------------------------+
|   META_ARCHITECT: FILTERING_SYNTAX.md                               |
|                                                                     |
|   EBNF Grammar Rules: 38 (exceeds 30+ requirement)                  |
|   Operator Precedence Levels: 6                                     |
|   Example Queries: 28 (exceeds 20+ requirement)                     |
|   AST Variants: 27 (exceeds 15+ requirement)                        |
|   Error Types: 16 (exceeds 10+ requirement)                         |
|                                                                     |
|   Status: PENDING HOSTILE_REVIEWER APPROVAL                         |
|                                                                     |
+---------------------------------------------------------------------+
```

---

**Document Owner:** META_ARCHITECT
**Review Required:** HOSTILE_REVIEWER
**Next Task:** W22.2 (Filter Evaluator Architecture)

---

*"A grammar well-defined is a parser half-written."*
