# Changelog

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
