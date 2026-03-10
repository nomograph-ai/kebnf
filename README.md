# kebnf-to-tree-sitter

[![pipeline status](https://gitlab.com/nomograph/kebnf-to-tree-sitter/badges/main/pipeline.svg)](https://gitlab.com/nomograph/kebnf-to-tree-sitter/-/pipelines)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Nomograph Labs](https://img.shields.io/badge/Nomograph_Labs-nomograph.ai-8B5CF6)](https://nomograph.ai)

Convert OMG KEBNF grammars to tree-sitter `grammar.js` files.

> **Status: Prototype / On Hold.** This tool was developed as part of the initial
> [tree-sitter-sysml](https://gitlab.com/nomograph/tree-sitter-sysml) effort to
> bootstrap a grammar from the OMG SysML v2 specification. The parser and emitter
> work for a subset of KEBNF, but the tree-sitter-sysml grammar has since been
> hand-tuned well beyond what automated transpilation produces. We plan to return
> to this tool to support additional OMG languages and to improve the fidelity of
> the generated grammars.

KEBNF (Kernel Extended BNF) is the grammar notation used by the Object Management
Group (OMG) to define the concrete syntax of modeling languages like SysML v2, KerML,
and others. This tool parses KEBNF grammar specifications and generates tree-sitter
grammar definitions, enabling incremental parsing support for OMG-specified languages.

## Usage

```bash
cargo run -- --input path/to/grammar.kebnf --output grammar.js
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## License

MIT — see [LICENSE](LICENSE).

## Links

- [Nomograph Labs](https://nomograph.ai)
- [tree-sitter-sysml](https://gitlab.com/nomograph/tree-sitter-sysml) — the grammar produced by this tool
- [SysML v2 specification](https://www.omg.org/spec/SysML/2.0)
