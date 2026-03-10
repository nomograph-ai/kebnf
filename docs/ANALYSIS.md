# Grammar Analysis: tree-sitter-sysml vs KEBNF

This document analyzes the existing hand-written tree-sitter-sysml grammar to inform the kebnf-to-tree-sitter converter design.

## tree-sitter-sysml Grammar Summary

**Location**: `../tree-sitter-sysml/grammar.js`  
**Lines**: 1723  
**Rules**: ~120 named rules

### Key Structural Patterns

#### 1. Precedence Constants

```javascript
const PREC = {
  COMMENT: 1,
  VISIBILITY: 2,
  SPECIALIZATION: 3,
  TYPING: 4,
};
```

Low precedence values (1-4) suggest minimal use of explicit precedence.

#### 2. Conflicts Declaration

The grammar declares 19 explicit conflicts:

| Conflict Pair | Reason |
|--------------|--------|
| `part_definition` vs `part_usage` | `part def` vs `part` ambiguity |
| `item_definition` vs `item_usage` | Same pattern |
| `port_definition` vs `port_usage` | Same pattern |
| `action_definition` vs `action_usage` | Same pattern |
| `state_definition` vs `state_usage` | Same pattern |
| `constraint_definition` vs `constraint_usage` | Same pattern |
| `requirement_definition` vs `requirement_usage` | Same pattern |
| `connection_definition` vs `connection_usage` | Same pattern |
| `interface_definition` vs `interface_usage` | Same pattern |
| `allocation_definition` vs `allocation_usage` | Same pattern |
| `attribute_definition` vs `attribute_usage` | Same pattern |
| `flow_definition` vs `flow_usage` | Same pattern |
| `flow_part` vs `flow_statement` | Flow parsing ambiguity |
| `usage_declaration` vs `end_usage` | Multiplicity placement |
| `usage_declaration` vs `perform_statement` | Perform parsing |
| `usage_declaration` (self) | Internal repeat ambiguity |
| `feature_chain` vs `qualified_name` | Name parsing |
| `feature_chain` vs `identification` | Perform feature chain |
| `identification` vs `qualified_name` | Variant reference |
| `definition_body` vs `usage_body` | Body parsing |
| `rendering_definition` vs `usage_declaration` | Rendering |
| `rendering_definition` vs `rendering_usage` | Rendering |
| `event_occurrence_usage` vs `event_usage` | Event parsing |

**Key insight**: Definition vs Usage conflicts are the most common pattern. The KEBNF handles this by having separate rule hierarchies, but tree-sitter needs GLR conflicts.

#### 3. Rule Naming Conventions

- **Snake_case** for all rules: `part_definition`, `usage_declaration`
- **Underscore prefix** for hidden rules: `_definition`, `_usage`, `_expression`
- **Suffix patterns**: `_definition`, `_usage`, `_statement`, `_part`, `_body`

#### 4. Common Patterns

**Optional prefix metadata**:
```javascript
seq(repeat($.prefix_metadata), ...)
```

**Optional visibility**:
```javascript
seq(optional($.visibility), ...)
```

**Body pattern**:
```javascript
choice(";", seq("{", repeat($._member), "}"))
```

**Comma-separated lists**:
```javascript
function commaSep1(rule) {
  return seq(rule, repeat(seq(",", rule)));
}
```

### Precedence Usage Analysis

| Function | Count | Context |
|----------|-------|---------|
| `prec()` | 1 | `parenthesized_expression` (2) |
| `prec.left()` | 10 | Binary operators, chains, expressions |
| `prec.right()` | 1 | `unary_expression` (2) |

Most rules don't use explicit precedence - conflicts are declared instead.

### Expression Handling

Expressions use low precedence numbers (1-5):

| Precedence | Rules |
|------------|-------|
| 1 | `binary_expression`, `collection_expression`, `meta_expression`, `feature_chain` |
| 2 | `parenthesized_expression`, `unary_expression`, `range_expression` |
| 3 | `feature_chain_expression` |
| 4 | `select_expression`, `invocation_expression`, `function_call_expression` |
| 5 | `index_expression`, `measurement_expression` |

### KEBNF Pattern Mapping

| KEBNF Pattern | tree-sitter-sysml Implementation |
|--------------|--------------------------------|
| `A B C` | `seq(A, B, C)` |
| `A \| B` | `choice(A, B)` |
| `A?` | `optional(A)` |
| `A*` | `repeat(A)` |
| `A+` | `repeat1(A)` |
| `: Type` | Not represented (semantic) |
| `prop = X` | Just `X` |
| `prop += X` | Just `X` (repeated in context) |
| `prop ?= 'x'` | `optional('x')` |
| `[Name]` | `$.qualified_name` |
| `{ }` | No visible pattern |
| `SYMBOL = ...` | Inlined as `choice()` |

### Notable Design Decisions

1. **No type annotations** - The grammar doesn't track metamodel types
2. **No property tracking** - Assignments are implicit in structure
3. **Heavy use of conflicts** - Rather than precedence tuning
4. **Flat expression hierarchy** - Single `_expression` choice, not layered precedence
5. **Generous optionals** - Many things optional that KEBNF makes mandatory in context

### Gaps from KEBNF

Things in KEBNF not fully represented:

1. **Variable prefixes** (`s.prop`, `e.prop`) - Not tracked
2. **Semantic actions** - Not represented
3. **Type annotations** - Stripped
4. **Negated cross-references** - Parsed as regular qualified_name
5. **Symbol aliases** - Inlined at definition sites

## Implications for kebnf-to-tree-sitter

### What to Generate

1. **Conflicts array**: Detect definition/usage pairs automatically
2. **Snake_case names**: Convert `PascalCase` from KEBNF
3. **Hidden rules**: Rules used only as components get `_` prefix
4. **Inline symbol aliases**: Expand `SPECIALIZES` etc. at use sites
5. **commaSep helpers**: Generate reusable helper functions

### Precedence Strategy

Based on tree-sitter-sysml analysis:

1. **Avoid heavy precedence** - Use conflicts array instead
2. **Expressions only**: Apply precedence within expression grammar
3. **Low numbers**: 1-5 range is sufficient
4. **Default left-associative**: `prec.left()` for binary operators

### Conflict Detection Heuristics

Generate conflicts for:
1. Rules with same keyword prefix but different suffixes (`X_definition` vs `X_usage`)
2. Rules with overlapping optional prefixes
3. Expression alternatives that share prefix tokens

### Mapping Document Structure

Track in mapping.json:
```json
{
  "rules": {
    "part_definition": {
      "kebnf_name": "PartDefinition",
      "kebnf_line": 123,
      "produces_type": "PartDefinition",
      "stripped_annotations": [...],
      "conflicts_with": ["part_usage"]
    }
  }
}
```

## Next Steps

1. Build KEBNF parser that captures all patterns
2. Implement conflict detection heuristics
3. Generate grammar.js matching tree-sitter-sysml conventions
4. Compare generated vs hand-written for validation
