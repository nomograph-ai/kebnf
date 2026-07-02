use clap::Parser as ClapParser;
use std::path::PathBuf;
use std::process::ExitCode;

mod ast;
mod closure;
mod emitters;
mod fetch;
mod mapping;
mod naming;
mod parser;

#[derive(ClapParser, Debug)]
#[command(name = "kebnf")]
#[command(about = "Swiss army knife for OMG KeBNF grammars")]
#[command(version)]
struct Cli {
    /// Input KEBNF file(s)
    #[arg(required_unless_present = "fetch_spec")]
    input: Vec<PathBuf>,

    /// Output file path (default depends on --format)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format: tree-sitter, antlr4
    #[arg(short, long, default_value = "tree-sitter")]
    format: String,

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

    /// Transitively extend --include so the emitted grammar has no
    /// dangling $.x references. Only meaningful with a non-empty --include;
    /// only implemented for --format tree-sitter.
    #[arg(long)]
    closure: bool,

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
        eprintln!("kebnf v{}", env!("CARGO_PKG_VERSION"));
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
    let format: emitters::OutputFormat = cli
        .format
        .parse()
        .map_err(|e: String| -> Box<dyn std::error::Error> { e.into() })?;

    let output_path = cli.output.clone().unwrap_or_else(|| {
        let ext = emitters::default_extension(format);
        if format == emitters::OutputFormat::Antlr4 {
            // ANTLR4 requires filename == grammar name
            let grammar_name = emitters::antlr4_grammar_name(&cli.name);
            PathBuf::from(format!("{}.{}", grammar_name, ext))
        } else {
            PathBuf::from(format!("grammar.{}", ext))
        }
    });

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

    if cli.closure && cli.include.is_empty() {
        eprintln!(
            "Warning: --closure has no effect with an empty --include (include-all already includes everything); proceeding without closure"
        );
    }
    if cli.closure && !cli.include.is_empty() && format != emitters::OutputFormat::TreeSitter {
        eprintln!(
            "Warning: --closure is only implemented for --format tree-sitter; proceeding without closing"
        );
    }

    let want_closure =
        cli.closure && !cli.include.is_empty() && format == emitters::OutputFormat::TreeSitter;

    let (filtered_rules, output_content) = if want_closure {
        let result = closure::close_includes(
            &all_rules,
            &cli.include,
            &cli.exclude,
            &cli.name,
            cli.verbose,
        )?;
        eprintln!(
            "closure: converged after {} iteration(s), {} kebnf rule(s) included",
            result.iterations,
            result.include.len()
        );
        (result.rules, result.output)
    } else {
        let filtered = filter_rules(&all_rules, &cli.include, &cli.exclude);
        let content = emitters::emit(&filtered, &cli.name, format)?;
        if format == emitters::OutputFormat::TreeSitter {
            let dangling = closure::find_dangling(&content);
            if !dangling.is_empty() {
                let names: Vec<_> = dangling.into_iter().collect();
                eprintln!(
                    "Warning: emitted grammar has {} dangling $.x reference(s) with no defining rule: {}. Re-run with --closure (and a non-empty --include) to transitively pull in the missing rules.",
                    names.len(),
                    names.join(", ")
                );
            }
        }
        (filtered, content)
    };

    if cli.verbose {
        eprintln!("After filtering: {} rules", filtered_rules.len());
    }

    std::fs::write(&output_path, &output_content)?;

    if cli.verbose {
        eprintln!("Wrote {} grammar to: {}", format, output_path.display());
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
        if format != emitters::OutputFormat::TreeSitter {
            eprintln!("Warning: --validate only supported for tree-sitter format");
        } else {
            if cli.verbose {
                eprintln!("Validating with tree-sitter generate...");
            }
            validate_grammar(&output_path)?;
        }
    }

    Ok(())
}

pub(crate) fn filter_rules(
    rules: &[ast::Rule],
    include: &[String],
    exclude: &[String],
) -> Vec<ast::Rule> {
    rules
        .iter()
        .filter(|rule| {
            let name = &rule.name;
            let included = include.is_empty() || include.iter().any(|p| matches_pattern(name, p));
            let excluded = exclude.iter().any(|p| matches_pattern(name, p));
            included && !excluded
        })
        .cloned()
        .collect()
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    if let Some(middle) = pattern.strip_prefix('*').and_then(|s| s.strip_suffix('*')) {
        name.contains(middle)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        name.ends_with(suffix)
    } else if let Some(prefix) = pattern.strip_suffix('*') {
        name.starts_with(prefix)
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
