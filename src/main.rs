use clap::Parser as ClapParser;
use std::path::PathBuf;
use std::process::ExitCode;

mod ast;
mod emitter;
mod fetch;
mod mapping;
mod parser;

#[derive(ClapParser, Debug)]
#[command(name = "kebnf-to-tree-sitter")]
#[command(about = "Convert OMG KEBNF grammars to tree-sitter grammar.js files")]
#[command(version)]
struct Cli {
    /// Input KEBNF file(s)
    #[arg(required_unless_present = "fetch_spec")]
    input: Vec<PathBuf>,

    /// Output grammar.js file path
    #[arg(short, long, default_value = "grammar.js")]
    output: PathBuf,

    /// Output mapping.json file path
    #[arg(short, long)]
    mapping: Option<PathBuf>,

    /// Grammar name (defaults to "sysml")
    #[arg(short, long, default_value = "sysml")]
    name: String,

    /// Include only rules matching these patterns (comma-separated)
    #[arg(long, value_delimiter = ',')]
    include: Vec<String>,

    /// Exclude rules matching these patterns (comma-separated)
    #[arg(long, value_delimiter = ',')]
    exclude: Vec<String>,

    /// Validate output with tree-sitter generate
    #[arg(long)]
    validate: bool,

    /// Fetch latest KEBNF specs from GitHub
    #[arg(long)]
    fetch_spec: bool,

    /// Output automation statistics
    #[arg(long)]
    stats: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("kebnf-to-tree-sitter v{}", env!("CARGO_PKG_VERSION"));
    }

    if cli.fetch_spec {
        match fetch::fetch_specs(cli.verbose) {
            Ok(paths) => {
                for path in paths {
                    println!("Fetched: {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("Error fetching specs: {}", e);
                return ExitCode::FAILURE;
            }
        }
        if cli.input.is_empty() {
            return ExitCode::SUCCESS;
        }
    }

    if cli.input.is_empty() {
        eprintln!("Error: No input files specified");
        return ExitCode::FAILURE;
    }

    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_rules = Vec::new();

    for input_path in &cli.input {
        if cli.verbose {
            eprintln!("Parsing: {}", input_path.display());
        }

        let source = std::fs::read_to_string(input_path)?;
        let rules = parser::parse(&source, input_path.to_string_lossy().as_ref())?;
        all_rules.extend(rules);
    }

    if cli.verbose {
        eprintln!("Parsed {} rules", all_rules.len());
    }

    let filtered_rules = filter_rules(all_rules, &cli.include, &cli.exclude);

    if cli.verbose {
        eprintln!("After filtering: {} rules", filtered_rules.len());
    }

    let grammar_js = emitter::emit(&filtered_rules, &cli.name)?;
    std::fs::write(&cli.output, &grammar_js)?;

    if cli.verbose {
        eprintln!("Wrote grammar to: {}", cli.output.display());
    }

    if let Some(mapping_path) = &cli.mapping {
        let mapping_json = mapping::generate(&filtered_rules)?;
        std::fs::write(mapping_path, mapping_json)?;

        if cli.verbose {
            eprintln!("Wrote mapping to: {}", mapping_path.display());
        }
    }

    if cli.stats {
        let stats = mapping::compute_stats(&filtered_rules);
        println!("{}", serde_json::to_string_pretty(&stats)?);
    }

    if cli.validate {
        if cli.verbose {
            eprintln!("Validating with tree-sitter generate...");
        }
        validate_grammar(&cli.output)?;
    }

    Ok(())
}

fn filter_rules(
    rules: Vec<ast::Rule>,
    include: &[String],
    exclude: &[String],
) -> Vec<ast::Rule> {
    rules
        .into_iter()
        .filter(|rule| {
            let name = &rule.name;
            let included = include.is_empty() || include.iter().any(|p| matches_pattern(name, p));
            let excluded = exclude.iter().any(|p| matches_pattern(name, p));
            included && !excluded
        })
        .collect()
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern.starts_with('*') && pattern.ends_with('*') {
        let middle = &pattern[1..pattern.len() - 1];
        name.contains(middle)
    } else if pattern.starts_with('*') {
        name.ends_with(&pattern[1..])
    } else if pattern.ends_with('*') {
        name.starts_with(&pattern[..pattern.len() - 1])
    } else {
        name == pattern
    }
}

fn validate_grammar(grammar_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("tree-sitter")
        .arg("generate")
        .arg(grammar_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tree-sitter generate failed:\n{}", stderr).into());
    }

    Ok(())
}
