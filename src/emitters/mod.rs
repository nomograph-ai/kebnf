pub mod antlr4;
pub mod tree_sitter;

use crate::ast::Rule;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    TreeSitter,
    Antlr4,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::TreeSitter => write!(f, "tree-sitter"),
            OutputFormat::Antlr4 => write!(f, "antlr4"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tree-sitter" | "treesitter" | "ts" => Ok(OutputFormat::TreeSitter),
            "antlr4" | "antlr" | "g4" => Ok(OutputFormat::Antlr4),
            _ => Err(format!(
                "Unknown format '{}'. Valid formats: tree-sitter, antlr4",
                s
            )),
        }
    }
}

/// Emit rules in the specified output format.
pub fn emit(
    rules: &[Rule],
    grammar_name: &str,
    format: OutputFormat,
) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        OutputFormat::TreeSitter => tree_sitter::emit(rules, grammar_name)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
        OutputFormat::Antlr4 => {
            antlr4::emit(rules, grammar_name).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}

/// Default file extension for the given format.
pub fn default_extension(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::TreeSitter => "js",
        OutputFormat::Antlr4 => "g4",
    }
}
