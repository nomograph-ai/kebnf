# Product Requirements Document: kebnf-to-tree-sitter

## Executive Summary

**kebnf-to-tree-sitter** is a command-line tool that converts OMG KEBNF (KerML Extended BNF) grammar specifications to tree-sitter `grammar.js` files. This enables automated generation of syntax highlighting and parsing support for MBSE languages like SysML v2.

### Vision

Enable one-shot direct conversion from OMG KEBNF releases to tree-sitter grammars with maximum accuracy. Where tree-sitter limitations require mapping, explicitly document whether each gap represents a potential enhancement opportunity or an inherent semantic/syntactic difference.

**Contrast with tree-sitter-sysml**: That project uses brute-force iteration against test files. This project aims for spec-driven generation.

### Target Users

1. **Grammar maintainers** - People maintaining tree-sitter grammars for OMG languages
2. **Tool developers** - Building IDE support, linters, or analysis tools for SysML/KerML
3. **Researchers** - Studying grammar transposition and MBSE tooling

## Problem Statement

### Current State

The official SysML v2 grammar is defined in KEBNF format (~3000 lines across KerML and SysML). To create tree-sitter support, developers must:

1. Manually read the KEBNF specification
2. Hand-write tree-sitter grammar rules
3. Iterate until tests pass
4. Repeat when the specification updates

This is error-prone, time-consuming, and creates drift between specification and implementation.

### Evidence of Problem

Our `tree-sitter-sysml` project achieved 100% parse coverage on training files through empirical iteration, but:
- Built without systematic reference to KEBNF specification
- May over-accept invalid syntax
- No formal mapping between our rules and spec rules
- Maintenance burden when OMG releases updates

### Desired State

A tool that:
1. Parses official KEBNF files
2. Generates tree-sitter `grammar.js` automatically
3. Documents semantic gaps that require manual attention
4. Enables specification updates to flow through to tooling

## Goals and Non-Goals

### Goals

| Goal | Success Metric |
|------|----------------|
| Parse 100% of official SysML/KerML KEBNF | Zero parse errors on official files |
| Generate valid tree-sitter grammar | Output passes `tree-sitter generate` |
| Document semantic gaps | Mapping JSON covers all stripped annotations |
| Enable round-trip verification | Can compare generated vs hand-written grammar |
| Support incremental adoption | Can generate partial grammars for testing |

### Non-Goals (Out of Scope)

- Full semantic analysis of SysML models
- Runtime name resolution or type checking
- GUI or IDE integration (CLI only)
- Support for non-OMG EBNF dialects (future extension)
- Automatic conflict resolution (documents conflicts for manual fixing)

## User Stories

### US-1: Generate Grammar from Spec

**As a** grammar maintainer  
**I want to** convert KEBNF files to tree-sitter grammar  
**So that** I can create spec-aligned parsers quickly

**Acceptance Criteria:**
- CLI accepts one or more `.kebnf` files as input
- Outputs valid `grammar.js` file
- Generated grammar passes `tree-sitter generate`

### US-2: Understand Semantic Gaps

**As a** tool developer  
**I want to** see what semantic information was stripped  
**So that** I can implement post-parse processing

**Acceptance Criteria:**
- Generates `mapping.json` documenting all stripped annotations
- Includes type annotations, property assignments, semantic actions
- Links tree-sitter rules back to KEBNF rule names

### US-3: Verify Against Existing Grammar

**As a** grammar maintainer  
**I want to** compare generated grammar against my hand-written one  
**So that** I can identify gaps and improvements

**Acceptance Criteria:**
- Outputs rule-by-rule comparison report
- Identifies rules present in one but not other
- Highlights structural differences

### US-4: Handle Spec Updates

**As a** grammar maintainer  
**I want to** regenerate grammar when OMG updates the spec  
**So that** my grammar stays aligned with the specification

**Acceptance Criteria:**
- Deterministic output (same input → same output)
- Can diff old vs new generated grammar
- Documents new/changed/removed rules

### US-5: Research Analysis

**As a** researcher  
**I want to** analyze the transposition process  
**So that** I can publish findings on grammar conversion

**Acceptance Criteria:**
- Outputs statistics on automation rate
- Categorizes rules by conversion complexity
- Tracks semantic information loss

## Functional Requirements

### FR-1: KEBNF Parsing

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | Parse KEBNF rule definitions (`Name = Body`) | Must |
| FR-1.2 | Parse type annotations (`Name : Type = Body`) | Must |
| FR-1.3 | Parse sequences, choices, optionals, repetition | Must |
| FR-1.4 | Parse property assignments (`prop +=`, `prop =`) | Must |
| FR-1.5 | Parse boolean flags (`prop ?= 'terminal'`) | Must |
| FR-1.6 | Parse cross-references (`[QualifiedName]`) | Must |
| FR-1.7 | Parse semantic actions (`{ prop = val }`) | Must |
| FR-1.8 | Parse comments (`//` line, `/* */` block) | Must |
| FR-1.9 | Parse symbol definitions (`SPECIALIZES = ':>' \| 'specializes'`) | Should |
| FR-1.10 | Provide meaningful error messages with source locations | Must |
| FR-1.11 | Parse empty semantic blocks (`{ }`) | Must |
| FR-1.12 | Parse variable prefixes (`e.prop`, `s.prop`) | Must |
| FR-1.13 | Parse negated cross-references (`~[QualifiedName]`) | Must |

### FR-2: Tree-sitter Emission

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | Generate valid JavaScript `grammar.js` | Must |
| FR-2.2 | Convert rule names to snake_case | Must |
| FR-2.3 | Generate `seq()`, `choice()`, `optional()`, `repeat()`, `repeat1()` | Must |
| FR-2.4 | Generate terminal strings | Must |
| FR-2.5 | Generate rule references (`$.rule_name`) | Must |
| FR-2.6 | Mark hidden rules with `_` prefix | Should |
| FR-2.7 | Generate `prec()` for known conflicts | Should |
| FR-2.8 | Generate `token()` for lexical rules | Should |
| FR-2.9 | Include source comments showing KEBNF origin | Should |

### FR-3: Semantic Mapping

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | Generate JSON mapping document | Must |
| FR-3.2 | Document type annotations per rule | Must |
| FR-3.3 | Document property assignments | Must |
| FR-3.4 | Document stripped semantic actions | Must |
| FR-3.5 | Document cross-reference locations | Should |
| FR-3.6 | Include KEBNF source line numbers | Should |

### FR-4: CLI Interface

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | Accept input file paths as positional arguments | Must |
| FR-4.2 | `-o, --output` flag for grammar.js path | Must |
| FR-4.3 | `-m, --mapping` flag for mapping.json path | Should |
| FR-4.4 | `-n, --name` flag for grammar name | Should |
| FR-4.5 | `-v, --verbose` flag for detailed output | Should |
| FR-4.6 | `--stats` flag for automation statistics | Should |
| FR-4.7 | Exit code 0 on success, non-zero on error | Must |
| FR-4.8 | `--fetch-spec` flag to download official KEBNF from GitHub | Should |
| FR-4.9 | `--include` / `--exclude` flags for rule name pattern filtering | Should |
| FR-4.10 | `--validate` flag to run `tree-sitter generate` on output | Should |

## Non-Functional Requirements

### NFR-1: Performance

- Parse and convert SysML+KerML (~3000 lines) in < 5 seconds
- Memory usage < 100MB for typical inputs

### NFR-2: Reliability

- No panics on malformed input (graceful error handling)
- Deterministic output (same input always produces same output)

### NFR-3: Maintainability

- Comprehensive test coverage (>80% line coverage)
- Snapshot tests for regression detection
- Clear separation between parsing, transformation, and emission

### NFR-4: Usability

- Helpful error messages with source locations
- Examples in documentation
- `--help` output documents all options

## Technical Design

### Architecture Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   KEBNF     │     │    AST      │     │ grammar.js  │
│   Files     │────▶│ (Internal)  │────▶│   Output    │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ mapping.json│
                    └─────────────┘
```

### Module Structure

| Module | Responsibility |
|--------|----------------|
| `parser` | KEBNF lexing and parsing via Chumsky 1.0-alpha |
| `ast` | AST types representing KEBNF constructs |
| `transform` | AST normalization and preparation |
| `emitter` | Tree-sitter grammar.js code generation |
| `mapping` | Semantic mapping document generation |
| `fetch` | GitHub spec fetching with local caching |
| `cli` | Command-line interface via Clap |

### Key Data Structures

```rust
// KEBNF Rule
struct Rule {
    name: String,
    produces_type: Option<String>,  // Type annotation
    body: RuleBody,
    span: Span,
}

// Rule body variants
enum RuleBody {
    Sequence(Vec<RuleBody>),
    Choice(Vec<RuleBody>),
    Optional(Box<RuleBody>),
    Repeat(Box<RuleBody>),
    Repeat1(Box<RuleBody>),
    Terminal(String),
    RuleRef(String),
    CrossRef(String),
    Assignment(Assignment),
    BooleanFlag(BooleanFlag),
    SemanticAction(SemanticAction),
}
```

## Test Plan

### Unit Tests

| Area | Tests |
|------|-------|
| Parser | Each KEBNF construct parses correctly |
| Emitter | Each tree-sitter construct emits correctly |
| Transform | Rule name conversion, normalization |

### Integration Tests

| Test | Description |
|------|-------------|
| SysML conversion | Convert official SysML KEBNF, validate output |
| KerML conversion | Convert official KerML KEBNF, validate output |
| Round-trip | Generated grammar passes `tree-sitter generate` |

### Snapshot Tests

- Full grammar output for known inputs
- Mapping document structure
- Error message formatting

## Milestones

### M1: Parser Complete (Week 1)

- [ ] KEBNF lexer
- [ ] KEBNF parser (all patterns)
- [ ] Error reporting with spans
- [ ] Unit tests for parser

### M2: Basic Emission (Week 2)

- [ ] Tree-sitter grammar.js emitter
- [ ] Basic mapping.json emitter
- [ ] CLI with core flags
- [ ] Integration test with simple grammar

### M3: Full SysML Support (Week 3)

- [ ] Handle all SysML KEBNF patterns
- [ ] Generate mapping document
- [ ] Statistics output
- [ ] Documentation

### M4: Validation & Polish (Week 4)

- [ ] Compare against hand-written tree-sitter-sysml
- [ ] Conflict documentation
- [ ] README and examples
- [ ] Release v0.1.0

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| KEBNF patterns we haven't seen | Medium | Medium | Parse all official files early; add patterns as discovered |
| Tree-sitter conflicts in generated grammar | High | Medium | Document conflicts; provide precedence hints |
| Specification updates break parser | Low | Low | Version-pin test fixtures; CI against official repo |
| Scope creep (supporting more dialects) | Medium | Medium | Strict non-goals; defer to future versions |

## Success Criteria

### MVP (v0.1.0)

1. ✅ Parses official SysML-textual-bnf.kebnf without errors
2. ✅ Generates grammar.js that passes `tree-sitter generate`
3. ✅ Generates mapping.json documenting stripped annotations
4. ✅ CLI with `--output` and `--mapping` flags
5. ✅ README with usage examples

### v1.0.0

1. ✅ All MVP criteria
2. ✅ Handles both KerML and SysML KEBNF files
3. ✅ Generated grammar achieves >90% parse rate on training files
4. ✅ Published to crates.io
5. ✅ INCOSE paper submitted

## Appendix A: KEBNF Pattern Catalog

### A.1 Basic Patterns

```ebnf
// Simple rule
Foo = 'bar'

// Sequence
Foo = A B C

// Choice
Foo = A | B | C

// Optional
Foo = A?

// Repetition
Foo = A*
Foo = A+

// Grouping
Foo = (A B)?
```

### A.2 Annotation Patterns

```ebnf
// Type annotation
Foo : Bar = ...

// Property assignment (append)
ownedRelationship += Element

// Property assignment (single)
name = Identifier

// Boolean flag
isAbstract ?= 'abstract'

// Cross-reference
memberElement = [QualifiedName]

// Semantic action
{ isPortion = true }
```

### A.3 Symbol Definitions

```ebnf
SPECIALIZES = ':>' | 'specializes'
REDEFINES = ':>>' | 'redefines'
```

## Appendix B: Official KEBNF File Excerpts

### B.1 PartDefinition (SysML)

```ebnf
PartDefinition =
    OccurrenceDefinitionPrefix 'part' 'def' Definition

OccurrenceDefinitionPrefix : OccurrenceDefinition =
    BasicDefinitionPrefix?
    ( isIndividual ?= 'individual'
      ownedRelationship += EmptyMultiplicityMember
    )?
    DefinitionExtensionKeyword*
```

### B.2 Expression (KerML)

```ebnf
OwnedExpression : Expression =
      ConditionalExpression
    | ConditionalBinaryOperatorExpression
    | BinaryOperatorExpression
    | UnaryOperatorExpression
    | ClassificationExpression
    | MetaclassificationExpression
    | ExtentExpression
    | PrimaryExpression
```

## Appendix C: Related Work

### C.1 Existing EBNF Converters

| Tool | Input | Output | Status |
|------|-------|--------|--------|
| eatkins/tree-sitter-ebnf-generator | W3C EBNF | tree-sitter | Archived |
| miks1965/yacc-to-tree-sitter | YACC | tree-sitter | Active |
| SEMAFORInformatik/EBNF2TS | YACC | tree-sitter | Active |

None support KEBNF or OMG's metamodel annotations.

### C.2 Prior Art in Grammar Transposition

- Xtext to LSP converters
- ANTLR grammar converters
- BNF Converter (BNFC)

### C.3 Our Contribution

First tool specifically targeting OMG KEBNF format with:
- Semantic mapping preservation
- Research-grade statistics output
- Focus on MBSE language ecosystem

## Appendix D: Research Metrics

The following metrics will be captured for the INCOSE paper:

### D.1 Rule Conversion Rate by Category

| Category | Description | Metric |
|----------|-------------|--------|
| Direct conversion | Sequences, choices, terminals, repetition | Count and % |
| Strip & convert | Type annotations, property assignments, cross-refs | Count and % |
| Best-effort approximation | Semantic actions, empty blocks | Count and % |
| Manual review required | Variable prefixes, conjugated ports, conflicts | Count and % |

### D.2 Semantic Information Loss

| Annotation Type | Count | Example |
|-----------------|-------|---------|
| Type annotations | N | `: PartDefinition` |
| Property assignments (=) | N | `name = NAME` |
| Property appends (+=) | N | `ownedRelationship += X` |
| Boolean flags (?=) | N | `isAbstract ?= 'abstract'` |
| Semantic actions | N | `{ isPortion = true }` |
| Cross-references | N | `[QualifiedName]` |

### D.3 Conflict Analysis

| Conflict Type | Count | Resolution | Enhancement Opportunity? |
|---------------|-------|------------|-------------------------|
| Expression precedence | N | prec() | No - standard |
| Keyword ambiguity | N | varies | Possible |
| Optional prefix | N | varies | Possible |
| Context-sensitive | N | manual | No - semantic gap |

### D.4 Tree-sitter DSL Feature Usage

| Feature | Used? | Count | Notes |
|---------|-------|-------|-------|
| `seq()` | | | |
| `choice()` | | | |
| `optional()` | | | |
| `repeat()` | | | |
| `repeat1()` | | | |
| `prec()` | | | |
| `prec.left()` | | | |
| `prec.right()` | | | |
| `token()` | | | |
| `field()` | | | |
| `alias()` | | | |
