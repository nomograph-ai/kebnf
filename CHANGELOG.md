# Changelog

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
