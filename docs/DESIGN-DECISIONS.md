# Design Decisions Requiring Review

This document catalogs all design decisions that need explicit review. Each decision includes context, options, tradeoffs, and a recommendation.

## Document Status

**Generated**: 2026-02-10  
**Total Decisions**: 35  
**Categories**: 7

---

## Category 1: Fundamental Architecture

### DD-001: Spec Fidelity vs Parsability Tradeoff

**Context**: Direct KEBNF conversion produces 335+ LR conflicts. The brute-force grammar has only 28 conflicts.

**Options**:
| Option | Conflicts | Spec Traceability | Maintenance |
|--------|-----------|-------------------|-------------|
| A: Pure spec-driven | 335+ | High | Re-run converter |
| B: Full restructure | ~28 | Low | Manual updates |
| C: Hybrid approach | ~50-100 | Medium | Mixed |

**Tradeoffs**:
- Option A: GLR can handle it but slower parsing, harder debugging
- Option B: Loses research value (INCOSE paper needs spec traceability)
- Option C: Best balance but requires documenting every deviation

**Recommendation**: Option C (Hybrid)

**Questions to resolve**:
1. What conflict count is acceptable for production use?
2. Is GLR performance acceptable for editor integration?
3. How important is spec traceability for downstream tools?

---

### DD-002: Rule Merging Strategy for Syntactically Identical Rules

**Context**: 23 rule groups have identical syntax but different semantic types (e.g., `PrimaryArgumentMember` vs `NonFeatureChainPrimaryArgumentMember`).

**Options**:
| Option | Description | Impact |
|--------|-------------|--------|
| A: Keep all rules | Declare conflicts, let GLR explore both | More conflicts |
| B: Merge to canonical | Pick one name, document aliases | Fewer rules |
| C: Merge with field | Add distinguishing field in AST | More complex |

**Current implementation**: None (pending decision)

**Recommendation**: Option B - simpler grammar, semantic distinction in mapping.json

---

### DD-003: Entry Point / Start Rule

**Context**: KEBNF has `RootNamespace` but tree-sitter needs `source_file`.

**Current implementation**: Generate `source_file: $ => repeat(choice($.root_namespace, $.package, ...))` with hardcoded top-level rules.

**Options**:
| Option | Description |
|--------|-------------|
| A: Hardcode top-level | Current approach |
| B: Detect from KEBNF | Find rules not referenced by others |
| C: CLI flag | `--entry-rules RootNamespace,Package` |

**Recommendation**: Option C for flexibility

---

## Category 2: Empty-String Rule Handling

### DD-004: Empty-Matching Rule Strategy

**Context**: Tree-sitter rejects rules that can match empty strings. KEBNF has 23+ such rules by design.

**Current implementation**: Hardcoded list of replacements in `emitter.rs:309-335`.

**Options**:
| Option | Description | Maintenance |
|--------|-------------|-------------|
| A: Hardcode replacements | Current approach | Manual updates per spec |
| B: Auto-detect + heuristics | Analyze rule bodies | Complex logic |
| C: Flag for manual review | Emit TODO comments | Human intervention |
| D: Hybrid detection | Auto-detect, provide defaults, allow overrides | Most flexible |

**Affected rules** (23 total):
- `Identification`, `FeatureIdentification` - both parts optional
- `RootNamespace`, `MemberPrefix`, `TypePrefix` - optional content
- `EmptyFeature`, `EmptyMultiplicity`, `EmptyUsage`, `EmptyActionUsage` - intentionally empty
- Plus 14 more prefix/body rules

**Recommendation**: Option D - auto-detect with override config file

---

### DD-005: Semantic-Only Empty Rules (`{ }`)

**Context**: Rules like `EmptyFeature : Feature = { }` exist purely for AST construction.

**Current implementation**: Emit `$._empty_marker` placeholder token that never matches real input.

**Options**:
| Option | Description |
|--------|-------------|
| A: Placeholder token | Current - never matches |
| B: Remove from grammar | Don't emit at all |
| C: Emit as `seq()` | Empty sequence (rejected by tree-sitter) |
| D: Comment only | Document but don't emit rule |

**Recommendation**: Option B - these are semantic, not syntactic

---

### DD-006: Optional-to-Required Transformation

**Context**: When converting `A?` that creates empty-matching, we make it required. Call sites need updating.

**Example**: `Identification = shortName? name?` → `identification = choice(shortName, name, seq(shortName, name))`

**Question**: Should the tool:
1. Transform the rule body only?
2. Also find and update call sites to add `optional()`?
3. Emit warnings about call sites?

**Recommendation**: Option 3 - warn but don't auto-transform call sites

---

## Category 3: Conflict Resolution

### DD-007: Definition/Usage Pair Conflicts (21 pairs)

**Context**: `part def X` vs `part x` share prefix keyword.

**Current implementation**: Auto-detect pairs, add to `conflicts` array.

**Options**:
| Option | Description |
|--------|-------------|
| A: GLR conflicts | Current - let parser explore both |
| B: Lookahead factoring | Refactor grammar to branch on `def` |
| C: Lexer modes | Different tokenization in different contexts |

**Recommendation**: Option A - standard tree-sitter approach

---

### DD-008: Context-Sensitive Body Members (~100 conflicts)

**Context**: `NamespaceBodyElement`, `PackageBodyElement`, `DefinitionBodyItem` overlap significantly.

**Options**:
| Option | Description | Spec Fidelity |
|--------|-------------|---------------|
| A: Preserve contexts | Declare all conflicts | High |
| B: Unified `_body_element` | Single rule accepts all | Low |
| C: Intersection + specific | Common rule + context-specific | Medium |

**Brute-force grammar uses**: Option B (`_usage_member`)

**Recommendation**: Option B for practicality, document in mapping

---

### DD-009: Expression Precedence Strategy

**Context**: Expressions need precedence to avoid conflicts.

**Current implementation**: None - expressions emit as flat `choice()`.

**Options**:
| Option | Description |
|--------|-------------|
| A: Flat choice + conflicts | Simple but many conflicts |
| B: Precedence tower | `prec.left(N, ...)` hierarchy |
| C: Copy brute-force | Match existing tree-sitter-sysml |

**Recommendation**: Option C - proven to work

---

### DD-010: Feature Chain vs Qualified Name Ambiguity

**Context**: `A.B.C` could be feature chain or qualified name.

**Current behavior**: Both rules exist, conflict declared.

**Options**:
| Option | Description |
|--------|-------------|
| A: Unified rule | Single `_name_chain` |
| B: Precedence | `prec()` to prefer one |
| C: Context-sensitive | Different rule per context |

**Recommendation**: Research needed - examine brute-force solution

---

## Category 4: Semantic Information Handling

### DD-011: Type Annotation Preservation

**Context**: KEBNF `: TypeName` annotations specify metamodel types.

**Current implementation**: Strip from grammar, record in mapping.json.

**Question**: Should we emit as comments in grammar.js?

**Options**:
| Option | Output |
|--------|--------|
| A: Strip silently | No trace in grammar |
| B: Comment per rule | `// : PartDefinition` |
| C: JSDoc annotation | `/** @type {PartDefinition} */` |

**Current**: Option B

**Recommendation**: Keep Option B

---

### DD-012: Property Assignment Handling

**Context**: `prop += X` and `prop = X` bind values to metamodel properties.

**Stats**: 476 appends, 149 assignments in SysML+KerML.

**Current implementation**: Strip operator, emit only `X`.

**Options**:
| Option | Description |
|--------|-------------|
| A: Strip completely | Current |
| B: Use `field()` | `field('prop', $.x)` |
| C: Alias with name | `alias($.x, 'prop')` |

**Tradeoff**: Option B provides named fields in AST but adds verbosity.

**Recommendation**: Research `field()` usage in other grammars

---

### DD-013: Boolean Flag Conversion

**Context**: `isAbstract ?= 'abstract'` sets boolean if keyword present.

**Stats**: 59 boolean flags.

**Current implementation**: `optional('abstract')`

**Options**:
| Option | Description |
|--------|-------------|
| A: `optional()` | Current - presence indicates true |
| B: `field()` wrapper | `field('is_abstract', optional('abstract'))` |

**Recommendation**: Option A is sufficient

---

### DD-014: Cross-Reference Handling

**Context**: `[QualifiedName]` indicates name resolution needed.

**Stats**: 55 cross-refs, 1 negated.

**Current implementation**: Emit as `$.qualified_name`.

**Questions**:
1. Should cross-refs be marked differently in AST?
2. How to handle negated cross-refs (`~[QualifiedName]`)?

**Recommendation**: Document in mapping, parse as regular names

---

### DD-015: Semantic Action Handling

**Context**: `{ isPortion = true }` sets properties unconditionally.

**Stats**: 37 semantic actions, 6 empty blocks.

**Current implementation**: Emit `seq()` (empty), document in mapping.

**Options**:
| Option | Description |
|--------|-------------|
| A: Empty seq | Current - no syntactic effect |
| B: Hidden marker | `$._semantic_marker` |
| C: Remove entirely | Don't emit anything |

**Recommendation**: Option C - these have no syntactic content

---

### DD-016: Variable Prefix Handling (`s.prop`, `e.prop`)

**Context**: Prefixes indicate scoping context for property binding.

**Stats**: 5 occurrences, flagged for manual review.

**Current implementation**: Strip prefix, flag rule for review.

**Options**:
| Option | Description |
|--------|-------------|
| A: Strip + flag | Current |
| B: Emit as field | `field('s_prop', ...)` |
| C: Separate rules | Split rule per context |

**Recommendation**: Option A - manual review is appropriate

---

## Category 5: Lexical Rules

### DD-017: Lexical Rule Handling

**Context**: KEBNF includes lexical rules (NAME, STRING_VALUE, etc.) that need regex in tree-sitter.

**Current implementation**: Skip problematic lexical rules, emit hardcoded alternatives.

**Options**:
| Option | Description |
|--------|-------------|
| A: Hardcode all | Current - known working patterns |
| B: Convert KEBNF | Parse KEBNF lexical rules, convert to regex |
| C: External scanner | Use tree-sitter external scanner |
| D: Config file | Allow user to provide lexical patterns |

**Recommendation**: Option D for flexibility, default to hardcoded

---

### DD-018: Comment Handling

**Context**: SysML has `//`, `/* */`, and REGULAR_COMMENT (note format).

**Current implementation**: Hardcoded comment regex in `extras`.

**Questions**:
1. Should `//* note text */` be separate token type?
2. How to handle nested comments (if any)?

**Recommendation**: Match existing tree-sitter-sysml behavior

---

### DD-019: String/Name Escape Sequences

**Context**: KEBNF defines escape sequences for names and strings.

**Current implementation**: Simplified regex.

**Options**:
| Option | Description |
|--------|-------------|
| A: Simple regex | Current - may over-accept |
| B: Full spec regex | Complex but correct |
| C: External scanner | Most accurate but complex |

**Recommendation**: Option A for MVP, consider B later

---

## Category 6: Symbol Alias Handling

### DD-020: Symbol Alias Expansion

**Context**: `SPECIALIZES = ':>' | 'specializes'` defines keyword alternatives.

**Current implementation**: Not implemented (symbol aliases not parsed).

**Options**:
| Option | Description |
|--------|-------------|
| A: Inline expansion | Replace `SPECIALIZES` with `choice(':>', 'specializes')` everywhere |
| B: Named rule | Emit `specializes: $ => choice(':>', 'specializes')` |
| C: Token alias | Use tree-sitter `alias()` |

**Recommendation**: Option A - matches tree-sitter-sysml approach

---

### DD-021: Conflicting Symbol Aliases

**Context**: `SPECIALIZES` and `SUBSETS` both map to `:>`.

**Impact**: Same token can have different semantic meanings.

**Options**:
| Option | Description |
|--------|-------------|
| A: Ignore | Parse as token, semantic analysis determines meaning |
| B: Context rule | Different rules per context |

**Recommendation**: Option A - context determines meaning

---

## Category 7: Output & Tooling

### DD-022: Grammar Validation Strategy

**Context**: `--validate` flag runs `tree-sitter generate`.

**Current implementation**: Shell out to `tree-sitter` CLI.

**Options**:
| Option | Description |
|--------|-------------|
| A: CLI shelling | Current - requires tree-sitter installed |
| B: Embedded validation | Link tree-sitter library |
| C: Optional validation | Skip if tree-sitter not found |

**Recommendation**: Option C - graceful degradation

---

### DD-023: Mapping Document Format

**Context**: mapping.json records stripped semantic information.

**Current implementation**: JSON with rules map and statistics.

**Questions**:
1. Should mapping include source KEBNF line numbers?
2. Should it include the original KEBNF text?
3. What format for downstream tool consumption?

**Recommendation**: Include line numbers, consider KEBNF snippets

---

### DD-024: Diff/Comparison Output

**Context**: PRD mentions comparing generated vs hand-written grammar.

**Current implementation**: Not implemented.

**Options**:
| Option | Description |
|--------|-------------|
| A: Text diff | Simple diff output |
| B: Structural diff | Compare AST structures |
| C: Rule mapping | Show which rules correspond |

**Recommendation**: Option C - most useful for validation

---

### DD-025: Incremental Generation

**Context**: `--include`/`--exclude` flags filter rules.

**Current implementation**: Simple pattern matching.

**Questions**:
1. Should filtered output include dependency rules?
2. How to handle broken references?

**Options**:
| Option | Description |
|--------|-------------|
| A: Strict filter | Only matched rules |
| B: Include deps | Auto-include referenced rules |
| C: Stub deps | Generate stubs for missing refs |

**Current**: Option C (stub rules)

**Recommendation**: Option B would be more useful

---

### DD-026: Statistics Output Format

**Context**: `--stats` outputs automation metrics.

**Current implementation**: JSON to stdout.

**Questions**:
1. Should stats go to stderr (allowing stdout for grammar)?
2. What metrics are needed for INCOSE paper?

**Additional metrics to consider**:
- Conflict count by category
- Rules requiring manual review (by reason)
- Semantic information loss by type
- Tree-sitter feature usage counts

---

## Category 8: Parser Implementation

### DD-027: Chumsky Version

**Context**: Using chumsky 1.0.0-alpha.8.

**Risk**: Alpha API may change.

**Options**:
| Option | Description |
|--------|-------------|
| A: Stay on alpha | Current - better API |
| B: Pin version | Lock to specific alpha |
| C: Downgrade to 0.9 | Stable but older API |

**Recommendation**: Option B - pin to working version

---

### DD-028: Error Recovery

**Context**: Parser currently fails on first error.

**Options**:
| Option | Description |
|--------|-------------|
| A: Fail fast | Current |
| B: Collect errors | Report all errors |
| C: Recovery mode | Skip bad rules, continue |

**Recommendation**: Option B for better UX

---

### DD-029: Rule Chunking Strategy

**Context**: Parser splits input into rule chunks before parsing.

**Current implementation**: Detect rule start by `Name =` or `Name : Type =`.

**Edge cases**:
- Rules with complex bodies containing `=`
- Comments between rules
- Symbol alias definitions

**Question**: Is current chunking robust enough?

---

## Category 9: Research & Documentation

### DD-030: INCOSE Paper Metrics

**Context**: Tool should capture metrics for research paper.

**Required metrics**:
1. Rule conversion rate by category
2. Semantic information loss quantified
3. Conflict frequency analysis
4. Tree-sitter feature coverage

**Current implementation**: Partial (stats command).

**Missing**:
- Conflict categorization
- Tree-sitter feature usage counts
- Comparison with brute-force grammar

---

### DD-031: Conflict Classification

**Context**: Need to distinguish "enhancement opportunity" vs "inherent gap".

**Categories proposed**:
| Category | Tree-sitter could help? | Example |
|----------|------------------------|---------|
| Expression precedence | No (standard) | Binary operators |
| Keyword ambiguity | Possibly | `part` vs `part def` |
| Context-sensitive | No (semantic) | Body member types |
| Optional prefix | Possibly | Visibility + definition |

**Recommendation**: Create classification scheme, apply to all conflicts

---

### DD-032: Documentation of Deviations

**Context**: Each deviation from spec should be documented.

**Current**: AMBIGUITY-RESOLUTIONS.md exists but incomplete.

**Proposed format**:
```markdown
## [ID]: Brief Title
- KEBNF source: file:line
- Tree-sitter output: rule name
- Deviation: what changed
- Rationale: why
- Impact: what downstream tools need to know
```

---

## Category 10: Future Extensibility

### DD-033: Support for Other EBNF Dialects

**Context**: PRD mentions ISO 14977, W3C, ABNF as future extensions.

**Current architecture**: Tightly coupled to KEBNF.

**Recommendation**: Consider parser trait abstraction if this is a priority.

---

### DD-034: Bidirectional Sync

**Context**: PRD mentions detecting drift between KEBNF and generated grammar.

**Questions**:
1. What triggers sync check?
2. How to report differences?
3. Should it auto-update grammar?

**Recommendation**: Out of scope for MVP, design for later

---

### DD-035: Configuration File

**Context**: Many decisions could be user-configurable.

**Candidates for config**:
- Entry point rules
- Empty-rule replacements
- Lexical patterns
- Conflict resolutions
- Rule renames

**Format options**: TOML, JSON, YAML

**Recommendation**: TOML for readability

---

## Summary Table

| ID | Decision | Status | Priority |
|----|----------|--------|----------|
| DD-001 | Spec fidelity vs parsability | Open | Critical |
| DD-002 | Rule merging strategy | Open | High |
| DD-003 | Entry point selection | Open | Medium |
| DD-004 | Empty-matching strategy | Partial | High |
| DD-005 | Semantic-only empty rules | Implemented | Low |
| DD-006 | Optional-to-required | Open | Medium |
| DD-007 | Definition/Usage conflicts | Implemented | Done |
| DD-008 | Context-sensitive bodies | Open | Critical |
| DD-009 | Expression precedence | Open | High |
| DD-010 | Feature chain ambiguity | Open | High |
| DD-011 | Type annotation handling | Implemented | Done |
| DD-012 | Property assignment | Implemented | Low |
| DD-013 | Boolean flag handling | Implemented | Done |
| DD-014 | Cross-reference handling | Implemented | Low |
| DD-015 | Semantic action handling | Implemented | Low |
| DD-016 | Variable prefix handling | Implemented | Done |
| DD-017 | Lexical rules | Partial | High |
| DD-018 | Comment handling | Implemented | Done |
| DD-019 | Escape sequences | Partial | Low |
| DD-020 | Symbol alias expansion | Open | Medium |
| DD-021 | Conflicting aliases | Open | Low |
| DD-022 | Validation strategy | Implemented | Done |
| DD-023 | Mapping format | Implemented | Low |
| DD-024 | Diff/comparison | Open | Medium |
| DD-025 | Incremental generation | Partial | Low |
| DD-026 | Statistics format | Implemented | Low |
| DD-027 | Chumsky version | Open | Low |
| DD-028 | Error recovery | Open | Medium |
| DD-029 | Rule chunking | Implemented | Low |
| DD-030 | INCOSE metrics | Partial | Medium |
| DD-031 | Conflict classification | Open | Medium |
| DD-032 | Deviation documentation | Partial | Medium |
| DD-033 | Other EBNF dialects | Deferred | Low |
| DD-034 | Bidirectional sync | Deferred | Low |
| DD-035 | Configuration file | Open | Medium |

## Critical Path

The following decisions block progress:

1. **DD-001** (Architecture) → Must decide before major refactoring
2. **DD-008** (Context-sensitive bodies) → Largest conflict source
3. **DD-009** (Expression precedence) → Second largest conflict source
4. **DD-020** (Symbol aliases) → Required for correct output

## Research Questions

These require investigation before deciding:

1. What is acceptable GLR conflict count for editor performance?
2. How does tree-sitter-sysml handle feature chain vs qualified name?
3. What `field()` patterns do other grammars use?
4. What metrics matter most for INCOSE paper?
