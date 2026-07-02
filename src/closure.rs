//! `--closure`: transitively extend `--include` so the emitted tree-sitter
//! grammar has no dangling `$.x` references.
//!
//! The tree-sitter emitter is non-compositional: most of the grammar
//! (source_file, the definition/usage patterns, body rules, the expression
//! precedence chain, the conflicts array, and ~80 "builtin" rules) is
//! emitted unconditionally, but that scaffolding references many rule
//! names that only get *defined* if the corresponding KeBNF rule was also
//! present in the filtered rule set. A single static reachability pass over
//! the full grammar's reference graph is not enough to predict this,
//! because which rules the emitter decides to synthesize depends on which
//! rules are present. Closing the gap requires an actual fixed-point loop:
//! emit, scan the emitted text for dangling references, reverse-map them
//! back to KeBNF rule names, extend the include set, and re-emit -- until
//! nothing is left dangling or no further progress can be made.

use crate::ast::Rule;
use crate::emitters::{self, OutputFormat};
use crate::naming::to_snake_case;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;

/// Hard cap on fixed-point iterations. If closure hasn't converged by then,
/// something structural is wrong (a scaffold reference with no KeBNF origin,
/// or a genuine bug) and we should fail loudly rather than loop forever.
const MAX_ITERATIONS: usize = 20;

/// Result of a successful closure run.
#[derive(Debug)]
pub struct ClosureResult {
    pub rules: Vec<Rule>,
    pub output: String,
    pub iterations: usize,
    pub include: Vec<String>,
}

/// Scan emitted tree-sitter output for rule names defined on the left-hand
/// side of a `name: $ => ...` line (equivalent to the multiline regex
/// `^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*\$\s*=>`).
pub fn find_defined_rules(text: &str) -> HashSet<String> {
    let mut defined = HashSet::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let ident_len = ident_prefix_len(trimmed);
        if ident_len == 0 {
            continue;
        }
        let name = &trimmed[..ident_len];
        let rest = trimmed[ident_len..].trim_start();
        let Some(rest) = rest.strip_prefix(':') else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('$') else {
            continue;
        };
        let rest = rest.trim_start();
        if rest.starts_with("=>") {
            defined.insert(name.to_string());
        }
    }
    defined
}

/// Scan emitted text for every `$.<ident>` reference (equivalent to the
/// regex `\$\.([A-Za-z_][A-Za-z0-9_]*)`).
pub fn find_referenced_rules(text: &str) -> HashSet<String> {
    let mut referenced = HashSet::new();
    for (idx, _) in text.match_indices("$.") {
        let rest = &text[idx + 2..];
        let ident_len = ident_prefix_len(rest);
        if ident_len > 0 {
            referenced.insert(rest[..ident_len].to_string());
        }
    }
    referenced
}

/// Referenced-but-not-defined rule names in emitted tree-sitter output.
pub fn find_dangling(text: &str) -> BTreeSet<String> {
    let defined = find_defined_rules(text);
    let referenced = find_referenced_rules(text);
    referenced.difference(&defined).cloned().collect()
}

/// Length, in bytes, of the identifier prefix `[A-Za-z_][A-Za-z0-9_]*` at
/// the start of `s`. Zero if `s` doesn't start with an identifier.
fn ident_prefix_len(s: &str) -> usize {
    let mut chars = s.char_indices();
    match chars.next() {
        Some((_, c)) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return 0,
    }
    let mut end = 1;
    for (i, c) in chars {
        if c.is_ascii_alphanumeric() || c == '_' {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    end
}

/// Build a snake_case -> KeBNF-name(s) reverse map from the full, unfiltered
/// rule set. More than one KeBNF name can produce the same snake form (rare,
/// but possible between ALL_CAPS and PascalCase names); over-inclusion here
/// is safe, so all candidates are kept.
pub fn build_reverse_map(all_rules: &[Rule]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for rule in all_rules {
        let snake = to_snake_case(&rule.name);
        let candidates = map.entry(snake).or_default();
        if !candidates.contains(&rule.name) {
            candidates.push(rule.name.clone());
        }
    }
    map
}

/// Run the include-set fixed point described at the top of this module.
///
/// `all_rules` is the full, unfiltered parse of every input file; `include`
/// and `exclude` are the user-supplied filters to start from.
pub fn close_includes(
    all_rules: &[Rule],
    include: &[String],
    exclude: &[String],
    grammar_name: &str,
    verbose: bool,
) -> Result<ClosureResult, Box<dyn Error>> {
    let reverse_map = build_reverse_map(all_rules);
    let mut current_include: Vec<String> = include.to_vec();
    let mut seen: HashSet<String> = current_include.iter().cloned().collect();

    for iteration in 1..=MAX_ITERATIONS {
        let filtered = crate::filter_rules(all_rules, &current_include, exclude);
        let output = emitters::emit(&filtered, grammar_name, OutputFormat::TreeSitter)?;
        let dangling = find_dangling(&output);

        if dangling.is_empty() {
            return Ok(ClosureResult {
                rules: filtered,
                output,
                iterations: iteration,
                include: current_include,
            });
        }

        if iteration == MAX_ITERATIONS {
            let names: Vec<_> = dangling.into_iter().collect();
            return Err(format!(
                "--closure did not converge within {} iterations ({} dangling reference(s) remain, {} kebnf rule(s) included): {}",
                MAX_ITERATIONS,
                names.len(),
                current_include.len(),
                names.join(", ")
            )
            .into());
        }

        let mut added_any = false;
        let mut unmappable = Vec::new();
        for name in &dangling {
            match reverse_map.get(name) {
                Some(candidates) => {
                    for candidate in candidates {
                        if seen.insert(candidate.clone()) {
                            current_include.push(candidate.clone());
                            added_any = true;
                        }
                    }
                }
                None => unmappable.push(name.clone()),
            }
        }

        if !added_any {
            let names: Vec<_> = dangling.into_iter().collect();
            return Err(format!(
                "--closure got stuck after {} iteration(s) with {} kebnf rule(s) included: {} dangling reference(s) remain and no new KeBNF rule could be found to resolve them. Remaining dangling references: {}. Unmappable (no KeBNF rule name reverse-maps to this snake_case name): {}.",
                iteration,
                current_include.len(),
                names.len(),
                names.join(", "),
                if unmappable.is_empty() {
                    "none".to_string()
                } else {
                    unmappable.join(", ")
                }
            )
            .into());
        }

        if verbose {
            eprintln!(
                "closure: iteration {} found {} dangling reference(s), extended include set to {} kebnf rule(s)",
                iteration,
                dangling.len(),
                current_include.len()
            );
        }
    }

    unreachable!("loop above always returns by iteration == MAX_ITERATIONS");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::RuleBody;

    fn rule(name: &str, body: RuleBody) -> Rule {
        Rule {
            name: name.to_string(),
            produces_type: None,
            body,
            span: 0..0,
            source_line: 0,
        }
    }

    #[test]
    fn finds_defined_rules_across_whitespace_variants() {
        let text =
            "  foo: $ => seq('x'),\n\nbar:$=>'y',\n  baz : $ => choice(),\nconflicts: $ => [\n],\n";
        let defined = find_defined_rules(text);
        assert!(defined.contains("foo"));
        assert!(defined.contains("bar"));
        assert!(defined.contains("baz"));
        // Non-rule top-level grammar properties also match the LHS pattern;
        // that's harmless over-inclusion (see module docs).
        assert!(defined.contains("conflicts"));
    }

    #[test]
    fn ignores_lines_that_are_not_rule_definitions() {
        let text = "  $.foo,\n  optional($.bar),\n";
        let defined = find_defined_rules(text);
        assert!(defined.is_empty());
    }

    #[test]
    fn finds_all_dollar_dot_references() {
        let text = "foo: $ => seq($.bar, $.baz, $.bar),\n";
        let referenced = find_referenced_rules(text);
        assert_eq!(referenced.len(), 2);
        assert!(referenced.contains("bar"));
        assert!(referenced.contains("baz"));
    }

    #[test]
    fn dangling_is_referenced_minus_defined() {
        let text = "foo: $ => seq($.bar, $.baz),\n\nbar: $ => 'x',\n";
        let dangling = find_dangling(text);
        assert_eq!(dangling, BTreeSet::from(["baz".to_string()]));
    }

    #[test]
    fn reverse_map_handles_all_caps_and_collisions() {
        // "ABC" (all-caps branch) and "Abc" (mixed-case branch) both reduce
        // to "abc" -- over-inclusion (keeping both candidates) is required.
        let rules = vec![
            rule("ABC", RuleBody::Terminal("x".to_string())),
            rule("Abc", RuleBody::Terminal("y".to_string())),
            rule("BASIC_NAME", RuleBody::Terminal("z".to_string())),
        ];
        let map = build_reverse_map(&rules);
        let mut abc_candidates = map.get("abc").cloned().unwrap_or_default();
        abc_candidates.sort();
        assert_eq!(abc_candidates, vec!["ABC".to_string(), "Abc".to_string()]);
        assert_eq!(
            map.get("basic_name").cloned(),
            Some(vec!["BASIC_NAME".to_string()])
        );
    }

    #[test]
    fn close_includes_converges_on_real_fixtures() {
        // A deliberately small --include set against the real pinned
        // KerML+SysML fixtures: several custom KeBNF rules that, on their
        // own, emit a grammar with dangling references into rules like
        // general_type / visibility_indicator / feature_specialization
        // that are only defined if those KeBNF rules are also included.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let mut all_rules = Vec::new();
        for file in [
            "tests/kebnf/KerML-textual-bnf.kebnf",
            "tests/kebnf/SysML-textual-bnf.kebnf",
        ] {
            let path = std::path::Path::new(manifest_dir).join(file);
            let source = std::fs::read_to_string(&path).expect("read fixture");
            let rules = crate::parser::parse(&source, file).expect("parse fixture");
            all_rules.extend(rules);
        }

        let seed = vec![
            "PartUsage".to_string(),
            "PartDefinition".to_string(),
            "AttributeUsage".to_string(),
            "AttributeDefinition".to_string(),
            "Multiplicity".to_string(),
        ];

        // Without closure: the seed set alone leaves dangling references.
        let filtered = crate::filter_rules(&all_rules, &seed, &[]);
        let output = emitters::emit(&filtered, "test", OutputFormat::TreeSitter).unwrap();
        assert!(
            !find_dangling(&output).is_empty(),
            "expected the small seed set to leave dangling references without --closure"
        );

        // With closure: converges to zero dangling references.
        let result = close_includes(&all_rules, &seed, &[], "test", false)
            .expect("closure should converge on the real fixtures");
        assert!(find_dangling(&result.output).is_empty());
        assert!(result.include.len() > seed.len());
        assert!(result.iterations >= 1);
    }

    #[test]
    fn close_includes_fails_cleanly_when_a_scaffold_reference_has_no_kebnf_origin() {
        // emit_builtin_rules unconditionally references $.general_type
        // (via typed_by/typings/subsettings/redefinitions/relationship_part)
        // regardless of the include set. If the corpus has no rule that
        // reverse-maps to "general_type" at all, closure can never resolve
        // that dangling reference and must fail with a clear error instead
        // of looping or emitting a broken grammar silently.
        let rules = vec![rule("SomeOtherRule", RuleBody::Terminal("x".to_string()))];
        let include = vec!["SomeOtherRule".to_string()];
        let result = close_includes(&rules, &include, &[], "test", false);
        let err = result.expect_err("closure should fail when a dangling ref has no KeBNF origin");
        let message = err.to_string();
        assert!(message.contains("general_type"), "message was: {message}");
    }
}
