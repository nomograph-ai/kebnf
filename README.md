# kebnf

[![Nomograph Labs](https://img.shields.io/badge/Nomograph_Labs-1a1a1a?style=flat&labelColor=f2f0eb&color=1a1a1a)](https://nomograph.ai)
[![License: MIT](https://img.shields.io/badge/License-MIT-1a1a1a.svg)](LICENSE)
[![pipeline status](https://gitlab.com/nomograph/kebnf/badges/main/pipeline.svg)](https://gitlab.com/nomograph/kebnf/-/pipelines)

Convert OMG KeBNF grammar specifications to parser grammars. Parses the
full KerML + SysML v2 KeBNF specs (640 rules) and emits target-specific
output with semantic traceability.

## Output Formats

| Format | Flag | Output | Status |
|--------|------|--------|--------|
| **ANTLR4** | `--format antlr4` | `.g4` | **CI-validated** -- compiles with antlr4 4.13.2, javac 21 |
| **tree-sitter** | `--format tree-sitter` | `grammar.js` | Prototype -- 335+ LR conflicts, not directly usable. See [tree-sitter-sysml](https://gitlab.com/nomograph/tree-sitter-sysml) for the hand-tuned grammar. |

## Quick Start

```bash
# Build from source
cargo build --release

# Convert SysML v2 KeBNF to ANTLR4 grammar
./target/release/kebnf KerML.kebnf SysML.kebnf --format antlr4 -o Sysml.g4

# Fetch the latest specs from the OMG GitHub repo, then convert
./target/release/kebnf --fetch-spec
./target/release/kebnf ~/.cache/kebnf/*.kebnf --format antlr4 -o Sysml.g4
```

## Getting the .g4 File

The CI pipeline generates and validates `Sysml.g4` on every commit.
Download it from the latest pipeline:

**Pipeline** > **antlr4-validate** job > **Artifacts** > `Sysml.g4`

Or browse: [latest pipeline artifacts](https://gitlab.com/nomograph/kebnf/-/pipelines/latest)

## CI Validation

Every push runs a four-stage validation:

1. **rust-build** -- zero compiler warnings
2. **rust-test** -- 27 tests pass
3. **rust-clippy** -- zero lint warnings
4. **antlr4-validate** -- generate .g4 from full KerML+SysML, compile with
   `antlr4 4.13.2` (zero errors), compile generated Java with `javac 21`
   (529 class files)

## What is KeBNF?

KeBNF (Kernel Extended BNF) is the grammar notation used by the OMG to define
the concrete syntax of SysML v2 and KerML. It extends standard EBNF with
metamodel-binding annotations:

- **Type annotations** (`Rule : Type = ...`) -- bind rules to metamodel types
- **Property assignments** (`prop = Value`, `items += Element`) -- AST construction
- **Boolean flags** (`isAbstract ?= 'abstract'`) -- keyword-driven properties
- **Cross-references** (`[QualifiedName]`) -- name resolution
- **Semantic actions** (`{ isPortion = true }`) -- unconditional property setting

These annotations control metamodel binding but have no syntactic effect.
`kebnf` strips them during conversion and records them in a mapping file
(`--mapping mapping.json`) for downstream tools that need traceability.

See [docs/KEBNF-SPEC.md](docs/KEBNF-SPEC.md) for the full notation reference.

## Architecture

```
KeBNF source (.kebnf)
    |
    v
  Parser (chumsky) --> AST
    |                   |
    |                   +--> ANTLR4 emitter ------> .g4
    |                   |
    |                   +--> tree-sitter emitter --> grammar.js
    |                   |
    |                   +--> mapping generator ----> mapping.json
    v
  Statistics (--stats)
```

The parser handles all 640 KerML + SysML v2 rules. Each emitter walks the
same AST. The ANTLR4 emitter handles:

- Lexer/parser rule split (ALL_CAPS -> lexer, CamelCase -> parser)
- ANTLR4 reserved word escaping (`import` -> `import_`)
- Duplicate rule deduplication (KerML and SysML overlap)
- Mutual left-recursion breaking (wrapper inlining + rule merging)

## Conversion Statistics

```
$ kebnf KerML.kebnf SysML.kebnf --format antlr4 --stats
{
  "total_rules": 640,
  "direct_conversion": 247,
  "strip_and_convert": 353,
  "best_effort": 37,
  "manual_review": 3
}
```

## CLI Reference

```
kebnf [OPTIONS] <INPUT>...

Arguments:
  <INPUT>...    Input .kebnf files

Options:
  -o <PATH>           Output file (default: grammar.{js,g4})
  -f, --format <FMT>  Output format: tree-sitter, antlr4 (default: tree-sitter)
  -n, --name <NAME>   Grammar name (default: sysml)
  -m, --mapping <PATH> Output mapping.json
  --include <PATTERNS> Include rules matching patterns (comma-separated)
  --exclude <PATTERNS> Exclude rules matching patterns
  --stats             Print conversion statistics
  --validate          Validate output with tree-sitter generate
  --fetch-spec        Download latest KeBNF specs from OMG GitHub
  -v, --verbose       Verbose output
```

## License

MIT

## Links

- [Nomograph Labs](https://nomograph.ai)
- [tree-sitter-sysml](https://gitlab.com/nomograph/tree-sitter-sysml) -- hand-tuned SysML v2 grammar for tree-sitter
- [SysML v2 Release](https://github.com/Systems-Modeling/SysML-v2-Release) -- OMG KeBNF source files
