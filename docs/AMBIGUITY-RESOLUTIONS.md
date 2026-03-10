# Ambiguity Resolution Registry

This document records every ambiguity encountered during KEBNF-to-tree-sitter conversion and the resolution chosen. Each decision is documented with:

- **Pattern**: The ambiguity type
- **KEBNF Source**: Where in the specification this arises
- **Resolution**: How we resolved it
- **Rationale**: Why this resolution was chosen
- **Alternative**: Other options considered
- **Tree-sitter Mechanism**: How it's implemented (conflict array, precedence, rule merge, etc.)

## Resolution Categories

| Category | Count | Description |
|----------|-------|-------------|
| Definition/Usage Pairs | 21 | Same keyword prefix, distinguished by `def` |
| Syntactically Identical | 23 groups | Different semantic types, same syntax |
| Context-Sensitive Bodies | ~100+ | Member types valid in multiple contexts |
| Expression Ambiguities | ~50+ | Feature chains vs qualified names |
| Empty-String Rules | 23 | Rules that match empty by design |

## Key Finding: Structural Mismatch

**The KEBNF specification structure has 335+ LR conflicts** when converted directly to tree-sitter, compared to **28 conflicts in the brute-force grammar**.

This indicates a fundamental mismatch between:
- KEBNF's design (for metamodel binding, not parsing)
- LR parsing requirements (unambiguous lookahead)

### Why This Matters

The brute-force grammar achieved low conflict count by:
1. Flattening context-sensitive bodies into one `_usage_member`
2. Using `prec()` extensively for expression precedence
3. Restructuring ambiguous patterns manually

The spec-driven approach must choose:
1. **Preserve specification structure** → 335+ conflicts (GLR heavy)
2. **Restructure for parsing** → Lose specification traceability
3. **Hybrid approach** → Use brute-force patterns where needed, document deviations

---

## Category 1: Definition/Usage Pairs

### Pattern Description

SysML v2 uses paired constructs where definitions and usages share keyword prefixes:
- `part def MyPart { }` (PartDefinition)
- `part myPart : MyPart;` (PartUsage)

The parser cannot distinguish these until it sees (or doesn't see) the `def` keyword.

### Resolution

**Mechanism**: Tree-sitter GLR conflict array

All 21 pairs are declared in the `conflicts` array, allowing the GLR parser to explore both interpretations and backtrack as needed.

### Affected Rules

| Definition | Usage | Shared Prefix |
|------------|-------|---------------|
| `AttributeDefinition` | `AttributeUsage` | `attribute` |
| `EnumerationDefinition` | `EnumerationUsage` | `enumeration` |
| `OccurrenceDefinition` | `OccurrenceUsage` | `occurrence` |
| `IndividualDefinition` | `IndividualUsage` | `individual` |
| `ItemDefinition` | `ItemUsage` | `item` |
| `PartDefinition` | `PartUsage` | `part` |
| `PortDefinition` | `PortUsage` | `port` |
| `ConnectionDefinition` | `ConnectionUsage` | `connection` |
| `InterfaceDefinition` | `InterfaceUsage` | `interface` |
| `AllocationDefinition` | `AllocationUsage` | `allocation` |
| `FlowDefinition` | `FlowUsage` | `flow` |
| `ActionDefinition` | `ActionUsage` | `action` |
| `StateDefinition` | `StateUsage` | `state` |
| `CalculationDefinition` | `CalculationUsage` | `calc` |
| `ConstraintDefinition` | `ConstraintUsage` | `constraint` |
| `RequirementDefinition` | `RequirementUsage` | `requirement` |
| `ConcernDefinition` | `ConcernUsage` | `concern` |
| `CaseDefinition` | `CaseUsage` | `case` |
| `AnalysisCaseDefinition` | `AnalysisCaseUsage` | `analysis` |
| `VerificationCaseDefinition` | `VerificationCaseUsage` | `verification` |
| `UseCaseDefinition` | `UseCaseUsage` | `use case` |
| `ViewDefinition` | `ViewUsage` | `view` |
| `ViewpointDefinition` | `ViewpointUsage` | `viewpoint` |
| `RenderingDefinition` | `RenderingUsage` | `rendering` |
| `MetadataDefinition` | `MetadataUsage` | `metadata` |

### Rationale

This is the standard tree-sitter approach for LR conflicts. The GLR algorithm handles this efficiently, and the conflict is inherent to the language design (not a grammar bug).

### Alternative Considered

**Lookahead factoring**: Refactor grammar to factor out common prefix and branch on `def`. Rejected because:
1. Significantly complicates grammar structure
2. Loses direct correspondence to KEBNF specification
3. Tree-sitter's GLR handles this well

---

## Category 2: Empty-String Rules

### Pattern Description

KEBNF allows rules that match empty strings for semantic reasons:
- `Identification = ('<' name '>')? name?` - Elements can be anonymous
- `MemberPrefix = visibility?` - Visibility is optional
- `TypePrefix = 'abstract'? metadata*` - Both parts optional

Tree-sitter rejects rules that match empty strings (except the start rule).

### Resolution

**Mechanism**: Rule replacement with non-empty alternatives

Each empty-matching rule is replaced with a version that requires at least one component.

### Affected Rules

| Original Rule | Original Body | Replacement | Semantic Impact |
|---------------|---------------|-------------|-----------------|
| `Identification` | `('<' name '>')? name?` | `choice(seq('<', name, '>', name?), name)` | Anonymous elements must use `optional($.identification)` at call sites |
| `FeatureIdentification` | Same pattern | Same replacement | Same impact |
| `RootNamespace` | `NamespaceBodyElement*` | `repeat1($.namespace_body_element)` | Empty files handled by `source_file` |
| `MemberPrefix` | `visibility?` | `$.visibility_indicator` | Visibility now required where `member_prefix` is used |
| `TypePrefix` | `'abstract'? metadata*` | `choice(seq('abstract', repeat(...)), repeat1(...))` | At least abstract or one metadata required |
| `EmptyFeature` | `{ }` | `$._empty_marker` | Semantic-only, uses placeholder token |
| `EmptyMultiplicity` | `{ }` | `$._empty_marker` | Semantic-only, uses placeholder token |
| `EmptyUsage` | `{ }` | `$._empty_marker` | Semantic-only, uses placeholder token |
| `EmptyActionUsage` | `{ }` | `$._empty_marker` | Semantic-only, uses placeholder token |

(Additional 14 rules documented in source code)

### Rationale

Tree-sitter's restriction is fundamental to its parsing algorithm. The replacement preserves parsing behavior while satisfying the constraint.

### Alternative Considered

**Inline at call sites**: Replace `$.identification` with `optional(seq(...))` everywhere. Rejected because:
1. Massive duplication across grammar
2. Loses rule naming for AST
3. Higher maintenance burden

---

## Category 3: Syntactically Identical Rules

### Pattern Description

Some KEBNF rules have identical syntax but different semantic types:

```
PrimaryArgumentMember : ParameterMembership =
    ownedMemberParameter = PrimaryArgument

NonFeatureChainPrimaryArgumentMember : ParameterMembership =
    ownedMemberParameter = PrimaryArgument
```

These are syntactically indistinguishable but represent different metamodel concepts.

### Resolution

**Status**: PENDING - Requires analysis

**Proposed Mechanism**: Rule merging with mapping document

Merge syntactically identical rules into one tree-sitter rule. The mapping document records which KEBNF rules map to it, allowing downstream tools to determine semantic type from context.

### Rationale

Tree-sitter produces concrete syntax trees, not abstract syntax trees. Semantic type determination is a post-parse activity that should use context, not syntax.

### Alternative Considered

**Conflict declaration**: Add both rules and declare conflict. Rejected because:
1. Creates unnecessary parse ambiguity
2. Slower parsing
3. No benefit since syntax is identical

---

## Category 4: Context-Sensitive Bodies

### Pattern Description

Different body contexts allow different member types:

- `NamespaceBody` allows `NamespaceBodyElement`
- `PackageBody` allows `PackageBodyElement` 
- `DefinitionBody` allows `DefinitionBodyItem`

These overlap significantly, causing conflicts when the same member appears in multiple contexts.

### Resolution

**Status**: PENDING - Requires analysis

**Proposed Mechanism**: Unified body element rule

Create a single `body_element` rule that accepts the union of all member types. Context-sensitive validation becomes a semantic pass.

### Rationale

This matches how the brute-force grammar handles it (`_usage_member` accepts all member types). It's simpler and faster to parse, with correctness enforced semantically.

### Alternative Considered

**Preserve context sensitivity**: Keep separate body element types, declare conflicts. Rejected because:
1. Dozens of conflict pairs needed
2. No parsing benefit (all are valid syntax)
3. Semantic validation needed anyway for full correctness

---

## Category 5: Expression Ambiguities

### Pattern Description

Feature chains and qualified names have overlapping syntax:

- `A.B.C` could be a qualified name or feature chain
- `A::B::C` is unambiguously a qualified name
- `A.B::C` mixes both

### Resolution

**Status**: PENDING - Requires analysis

**Proposed Mechanism**: Precedence and/or unified rule

### Rationale

TBD after detailed analysis.

---

## Appendix: Resolution Decision Template

When adding new resolutions, use this template:

```markdown
## [Resolution ID]: [Brief Title]

### Pattern Description
[Describe the ambiguity]

### KEBNF Source
[Line numbers and file references]

### Resolution
**Mechanism**: [conflict array | precedence | rule merge | rule replacement]

[Describe the resolution]

### Rationale
[Why this approach]

### Alternative Considered
[Other options and why rejected]

### Implementation
- File: [source file]
- Line: [line numbers]
- Commit: [commit hash when implemented]
```

---

## Change Log

| Date | Resolution | Author | Description |
|------|------------|--------|-------------|
| 2026-02-10 | Definition/Usage Pairs | AI-assisted | Initial 21 pairs documented |
| 2026-02-10 | Empty-String Rules | AI-assisted | 23 rules documented |
