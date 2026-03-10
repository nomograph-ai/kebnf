# Conflict Patterns Documentation

This document catalogs conflicts encountered when converting KEBNF grammars to tree-sitter, along with resolution strategies and analysis of whether each represents a tree-sitter enhancement opportunity or an inherent semantic/syntactic gap.

## Document Structure

Each conflict is documented with:
- **Pattern**: The conflicting grammar construct
- **Why It Conflicts**: Technical explanation of the LR(1) conflict
- **Resolution**: The heuristic/approach applied
- **Classification**: Enhancement opportunity vs. inherent gap
- **Status**: Documented / Resolved / Needs Manual Review

---

## Conflict Categories

### Category A: Expression Precedence

Standard operator precedence conflicts that occur in any expression grammar.

### Category B: Keyword Ambiguity

Cases where keywords can appear in multiple contexts with different meanings.

### Category C: Optional Prefix Ambiguity

Conflicts from optional prefixes that create ambiguous parse states.

### Category D: Recursive Structure

Left-recursive or mutually recursive rules that need restructuring.

### Category E: Context-Sensitive Semantics

Rules where the same syntax has different meanings based on context (inherent gaps).

---

## Documented Conflicts

### CONFLICT-001: [Template]

**Pattern:**
```ebnf
// KEBNF that causes conflict
```

**Why It Conflicts:**
Technical explanation of why this creates an LR(1) conflict.

**Resolution:**
```javascript
// Tree-sitter code showing resolution
```

**Classification:** Enhancement Opportunity / Inherent Gap

**Rationale:**
Why this classification was chosen.

**Status:** Documented / Resolved / Needs Manual Review

---

*Conflicts will be added here as they are discovered during implementation and testing.*

## Tree-sitter Enhancement Opportunities

This section tracks patterns that could potentially be handled better if tree-sitter added new features.

| ID | Pattern | Proposed Enhancement | Priority |
|----|---------|---------------------|----------|
| | | | |

## Inherent Semantic Gaps

These patterns represent fundamental differences between KEBNF (which encodes semantic information) and tree-sitter (which is purely syntactic). No tree-sitter enhancement would resolve these; they require post-parse processing.

| ID | Pattern | Why Inherent | Workaround |
|----|---------|--------------|------------|
| GAP-001 | Type annotations | Metamodel types are semantic, not syntactic | Document in mapping.json |
| GAP-002 | Property assignments | Property binding is semantic | Document in mapping.json |
| GAP-003 | Cross-reference resolution | Name resolution requires semantic analysis | Parse as name, resolve post-parse |
| GAP-004 | Semantic actions | Runtime property setting | Strip, document required post-processing |

## Analysis Methodology

When a conflict is encountered:

1. **Identify the minimal conflicting pattern** - Reduce to simplest reproduction
2. **Classify the conflict type** - Which category does it fall into?
3. **Research tree-sitter precedent** - How do other grammars handle similar cases?
4. **Determine if enhancement could help** - Would a tree-sitter feature resolve this?
5. **Document the resolution** - What heuristic do we apply?
6. **Test the resolution** - Verify it parses correctly

## References

- [Tree-sitter Conflicts Documentation](https://tree-sitter.github.io/tree-sitter/creating-parsers/3-writing-the-grammar.html#the-first-few-rules)
- [LR(1) Parsing Theory](https://en.wikipedia.org/wiki/LR_parser)
- [Tree-sitter GitHub Issues](https://github.com/tree-sitter/tree-sitter/issues) - Search for similar conflicts
