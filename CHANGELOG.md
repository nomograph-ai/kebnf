# Changelog

## [Unreleased]

### Added
- `--closure` flag: transitively extends `--include` so the emitted
  tree-sitter grammar has no dangling `$.x` references. Implemented as an
  iterative fixed point (emit, scan emitted text for dangling references,
  reverse-map them to KeBNF rule names, extend the include set, re-emit)
  because the tree-sitter emitter is non-compositional -- a single static
  reachability pass over the full grammar's reference graph is not
  sufficient to predict what a smaller rule subset will emit. Only
  implemented for `--format tree-sitter`; a no-op warning is printed if
  passed with `--format antlr4` or with an empty `--include`. Fails with a
  clear error (listing the remaining dangling references) if the fixed
  point doesn't converge within 20 iterations, or gets stuck because a
  dangling reference has no corresponding KeBNF rule at all.
- Warning printed to stderr whenever a `--format tree-sitter` emission
  (with or without `--closure`) has dangling `$.x` references with no
  defining rule; previously these were emitted silently.
- `ambiguity_resolutions.merged_rules` in `mapping.json` now reports the
  expression-chain rules the tree-sitter emitter inlines into
  `owned_expression` and `primary_expression` instead of emitting as their
  own rule (was always an empty array).

### Fixed
- `mapping.json`'s `tree_sitter_name` field for ALL-CAPS lexical terminals
  (e.g. `NAME`, `BASIC_NAME`, `UNRESTRICTED_NAME`, `WHITE_SPACE`,
  `CONJUGATES`, `SPECIALIZES`) no longer disagrees with what the
  tree-sitter emitter actually writes. `mapping.rs` had its own copy of
  `to_snake_case` that mangled all-caps names per letter (e.g. `NAME` ->
  `n_a_m_e`); it now shares the corrected implementation (already used by
  the tree-sitter emitter) via a new `src/naming.rs` module.

## 0.2.1 (2026-03-31)

### Fixed
- ANTLR4 output filename: default is now `Sysml.g4` (matching the `grammar Sysml;`
  declaration) instead of `grammar.g4`, which caused ANTLR4 error(8)
- `REGULAR_COMMENT` is now emitted as a parser-visible lexer token (`/* ... */`),
  fixing dead `comment`, `documentation`, and `textualRepresentation` rules
- `MULTILINE_NOTE` (`//* ... */`) emitted as hidden-channel annotation note
- Symbol alias tokens (`TYPED_BY`, `SPECIALIZES`, `SUBSETS`, `REFERENCES`,
  `CROSSES`, `REDEFINES`, `CONJUGATES`, `DEFINED_BY`) converted from lexer to
  parser rules so multi-token keyword alternatives (`'typed' 'by'`) work correctly
- `RESERVED_KEYWORD` and `RESERVED_SYMBOL` rules removed from output (unreferenced,
  caused 16 ANTLR4 warnings)
- Test fixtures renamed to match grammar names (`Sysml.g4`, `Simple.g4`)

### Added
- CI integration test: generated parser now parses a minimal SysML v2 file via
  ANTLR4 TestRig to verify end-to-end correctness
- Unit tests for symbol alias conversion (34 tests total, was 30)

## 0.2.0 (2026-03-23)

### Added
- ANTLR4 backend: generates valid .g4 grammars from KeBNF specifications
  - Validated with antlr4 4.13.2 (zero errors) and javac 21 (518 class files)
  - Handles reserved words, duplicate deduplication, mutual left-recursion
- tree-sitter backend: generates grammar.js with 96.9% corpus coverage
  - Pattern-based emission with inlined prefix keywords
  - 0.15ms parse speed, 11 of 15 test categories at 100%
- CI pipeline: 5 validation jobs (build, test, clippy, antlr4, tree-sitter)
- Repair loop harness for LLM-driven conflict resolution (scripts/)

### Changed
- Renamed from kebnf-to-tree-sitter to kebnf
- CLI: `kebnf --format antlr4|tree-sitter` (default: tree-sitter)
- Output file extension follows format (.g4 or .js)

## 0.1.0 (2025-02-10)

### Added
- Initial KeBNF parser (chumsky)
- tree-sitter emitter (prototype, 335+ LR conflicts)
- KeBNF spec fetcher
- Semantic traceability mapping
