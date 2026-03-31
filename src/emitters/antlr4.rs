use crate::ast::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct EmitError {
    pub message: String,
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EmitError {}

/// Emit rules as an ANTLR4 combined grammar (.g4 file).
pub fn emit(rules: &[Rule], grammar_name: &str) -> Result<String, EmitError> {
    let mut emitter = Antlr4Emitter::new(grammar_name);
    emitter.emit_grammar(rules)
}

struct Antlr4Emitter {
    grammar_name: String,
    output: String,
    rule_names: HashSet<String>,
    undefined_refs: HashSet<String>,
    emitted_rules: HashSet<String>,
    /// Maps rule names to their inline target when the rule is a trivial wrapper.
    inline_map: HashMap<String, String>,
    /// Rules whose bodies resolve to epsilon (empty). References are dropped.
    epsilon_rules: HashSet<String>,
}

impl Antlr4Emitter {
    fn new(grammar_name: &str) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            output: String::new(),
            rule_names: HashSet::new(),
            undefined_refs: HashSet::new(),
            emitted_rules: HashSet::new(),
            inline_map: HashMap::new(),
            epsilon_rules: HashSet::new(),
        }
    }

    fn emit_grammar(&mut self, rules: &[Rule]) -> Result<String, EmitError> {
        for rule in rules {
            self.rule_names.insert(rule.name.clone());
        }

        self.build_inline_map(rules);
        self.find_epsilon_rules(rules);
        self.collect_undefined_refs(rules);

        self.emit_header();

        // Separate parser rules (CamelCase) from lexer rules (ALL_CAPS).
        // Symbol alias tokens (TYPED_BY, etc.) are moved to the parser partition
        // because their multi-token keyword alternatives require parser-level handling.
        let (parser_rules, lexer_rules): (Vec<_>, Vec<_>) =
            rules.iter().partition(|r| !is_lexer_rule(&r.name) || is_symbol_alias(&r.name));

        // Section 1: Hand-crafted rules that break mutual left recursion
        self.emit_left_recursion_fixes();

        // Section 2: Parser rules
        if !parser_rules.is_empty() {
            self.output
                .push_str("// ─── Parser rules ────────────────────────────────────────\n\n");
            for rule in &parser_rules {
                if self.epsilon_rules.contains(&rule.name) {
                    continue; // skip epsilon rules entirely
                }
                self.emit_rule(rule)?;
            }
        }

        // Section 3: Lexer rules from KeBNF
        if !lexer_rules.is_empty() {
            self.output
                .push_str("// ─── Lexer rules (from KeBNF) ────────────────────────────\n\n");
            for rule in &lexer_rules {
                if self.should_skip_lexer_rule(&rule.name) {
                    continue;
                }
                self.emit_rule(rule)?;
            }
        }

        // Section 4: Built-in lexer rules
        self.emit_builtin_lexer_rules();

        // Section 5: Stub rules for undefined references
        self.emit_stub_rules();

        Ok(self.output.clone())
    }

    /// Build a map of trivial wrapper rules that should be inlined.
    fn build_inline_map(&mut self, rules: &[Rule]) {
        let mut direct: HashMap<String, String> = HashMap::new();
        for rule in rules {
            if let Some(target) = get_single_ref_target(&rule.body) {
                direct.insert(rule.name.clone(), target);
            }
        }

        // Resolve chains transitively (A→B→C becomes A→C, B→C)
        let keys: Vec<_> = direct.keys().cloned().collect();
        for key in &keys {
            let mut target = direct[key].clone();
            let mut seen = HashSet::new();
            seen.insert(key.clone());
            while let Some(next) = direct.get(&target) {
                if seen.contains(next) {
                    break;
                }
                seen.insert(target.clone());
                target = next.clone();
            }
            self.inline_map.insert(key.clone(), target);
        }
    }

    /// Find rules whose bodies resolve to epsilon (empty string).
    /// These are semantic-only rules used for metamodel binding.
    /// References to them are dropped from the output.
    fn find_epsilon_rules(&mut self, rules: &[Rule]) {
        // Direct epsilon: body is Empty or only SemanticActions
        for rule in rules {
            if body_is_epsilon(&rule.body) {
                self.epsilon_rules.insert(rule.name.clone());
            }
        }

        // Transitive: rules that only reference epsilon rules
        let mut changed = true;
        while changed {
            changed = false;
            for rule in rules {
                if self.epsilon_rules.contains(&rule.name) {
                    continue;
                }
                if self.body_resolves_to_epsilon(&rule.body) {
                    self.epsilon_rules.insert(rule.name.clone());
                    changed = true;
                }
            }
        }
    }

    fn body_resolves_to_epsilon(&self, body: &RuleBody) -> bool {
        match body {
            RuleBody::Empty => true,
            RuleBody::SemanticAction(_) => true,
            RuleBody::RuleRef(name) => {
                let resolved = self.inline_map.get(name).unwrap_or(name);
                self.epsilon_rules.contains(resolved)
            }
            RuleBody::Assignment(a) => self.body_resolves_to_epsilon(&a.value),
            RuleBody::Sequence(items) => items.iter().all(|i| self.body_resolves_to_epsilon(i)),
            _ => false,
        }
    }

    fn emit_left_recursion_fixes(&mut self) {
        let fixed_rules = [
            "ownedExpression",
            "binaryOperatorExpression",
            "conditionalBinaryOperatorExpression",
            "classificationExpression",
            "primaryExpression",
            "importDeclaration",
            "namespaceImport",
            "filterPackage",
        ];
        for name in &fixed_rules {
            self.emitted_rules.insert(name.to_string());
        }

        self.output.push_str(concat!(
            "// ─── Expressions ─────────────────────────────────────────\n",
            "// Merged from mutually recursive KeBNF rules into directly\n",
            "// left-recursive forms that ANTLR4's LL(*) parser accepts.\n",
            "// Original rules: OwnedExpression, BinaryOperatorExpression,\n",
            "// ConditionalBinaryOperatorExpression, ClassificationExpression.\n\n",
        ));

        self.output.push_str(concat!(
            "ownedExpression\n",
            "    : conditionalExpression\n",
            "    | ownedExpression conditionalBinaryOperator argumentExpressionMember\n",
            "    | ownedExpression binaryOperator ownedExpression\n",
            "    | unaryOperatorExpression\n",
            "    | ownedExpression classificationTestOperator typeReferenceMember\n",
            "    | ownedExpression castOperator typeResultMember\n",
            "    | classificationTestOperator typeReferenceMember\n",
            "    | castOperator typeResultMember\n",
            "    | metaclassificationExpression\n",
            "    | extentExpression\n",
            "    | primaryExpression\n",
            "    ;\n\n",
        ));

        self.output.push_str(concat!(
            "// Merged from PrimaryExpression, BracketExpression, IndexExpression,\n",
            "// SelectExpression, CollectExpression, FunctionOperationExpression,\n",
            "// FeatureChainExpression.\n\n",
            "primaryExpression\n",
            "    : literalExpression\n",
            "    | invocationExpression\n",
            "    | bodyExpression\n",
            "    | metadataAccessExpression\n",
            "    | nullExpression\n",
            "    | '(' ownedExpression ')'\n",
            "    | primaryExpression '#' '(' ownedExpression ')'\n",
            "    | primaryExpression '[' ownedExpression ']'\n",
            "    | primaryExpression '.' ownedExpression\n",
            "    | primaryExpression '.?' ownedExpression\n",
            "    | primaryExpression '.' bodyExpression\n",
            "    | primaryExpression '.?' bodyExpression\n",
            "    | primaryExpression '.' NAME\n",
            "    ;\n\n",
        ));

        self.output.push_str(concat!(
            "// ─── Imports ─────────────────────────────────────────────\n",
            "// Merged from ImportDeclaration, NamespaceImport, FilterPackage\n",
            "// to break mutual left recursion.\n\n",
            "importDeclaration\n",
            "    : membershipImport\n",
            "    | qualifiedName '::' '*' ('::' '**'?)?\n",
            "    | importDeclaration filterPackageMember+\n",
            "    ;\n\n",
            "namespaceImport\n",
            "    : qualifiedName '::' '*' ('::' '**'?)?\n",
            "    | importDeclaration filterPackageMember+\n",
            "    ;\n\n",
            "filterPackage\n",
            "    : importDeclaration filterPackageMember+\n",
            "    ;\n\n",
        ));
    }

    fn emit_header(&mut self) {
        let name = capitalize(&self.grammar_name);
        self.output.push_str(&format!(concat!(
            "// {name} — ANTLR4 combined grammar\n",
            "//\n",
            "// Generated from OMG KeBNF specifications by kebnf.\n",
            "// Source: https://gitlab.com/nomograph/kebnf\n",
            "//\n",
            "// This grammar was mechanically converted from the KerML and SysML v2\n",
            "// KeBNF specifications. Metamodel annotations (type bindings, property\n",
            "// assignments, cross-references) have been stripped. A mapping file\n",
            "// preserving semantic traceability can be generated with --mapping.\n",
            "//\n",
            "// Known limitations:\n",
            "//   - Expression and import rules were restructured to eliminate mutual\n",
            "//     left recursion (ANTLR4 requires direct left recursion only).\n",
            "//   - Rules named 'import' in KeBNF are emitted as 'import_' because\n",
            "//     'import' is an ANTLR4 reserved word.\n",
            "//   - Some KeBNF rules are purely semantic (e.g., EmptyFeature) and\n",
            "//     have been omitted. References to them are dropped.\n",
            "//   - Symbol alias tokens (SPECIALIZES, SUBSETS, etc.) are emitted as\n",
            "//     parser rules so multi-token keyword alternatives (e.g., 'typed' 'by')\n",
            "//     match correctly with whitespace between tokens.\n",
            "//   - REGULAR_COMMENT is a parser-visible lexer token matching /* ... */.\n",
            "//     It is used as structured comment body in Comment, Documentation,\n",
            "//     and TextualRepresentation rules. Annotation notes (//* ... */)\n",
            "//     are sent to channel(HIDDEN) as MULTILINE_NOTE.\n",
            "//\n\n",
        ), name = name));
        self.output.push_str(&format!("grammar {};\n\n", name));
    }

    fn emit_rule(&mut self, rule: &Rule) -> Result<(), EmitError> {
        let name = to_antlr4_name(&rule.name);

        // Skip duplicate rules (KerML and SysML both define some rules)
        if self.emitted_rules.contains(&name) {
            return Ok(());
        }
        self.emitted_rules.insert(name.clone());

        let body = self.emit_body(&rule.body)?;

        // Skip rules that produce empty bodies
        if body.is_empty() || body == "()" {
            return Ok(());
        }

        let mut comment = String::new();
        if let Some(ref produces_type) = rule.produces_type {
            comment = format!(" // : {}", produces_type);
        }

        // Note renamed rules
        if name.ends_with('_') && ANTLR4_RESERVED.contains(&&name[..name.len() - 1]) {
            comment.push_str(&format!(
                " // renamed from '{}' (ANTLR4 reserved word)",
                &name[..name.len() - 1]
            ));
        }

        self.output
            .push_str(&format!("{}\n    : {}\n    ;{}\n\n", name, body, comment));

        Ok(())
    }

    fn emit_body(&self, body: &RuleBody) -> Result<String, EmitError> {
        match body {
            RuleBody::Empty => Ok(String::new()),

            RuleBody::Terminal(s) => Ok(format!("'{}'", escape_antlr4(s))),

            RuleBody::RuleRef(name) => {
                let resolved = self.inline_map.get(name).unwrap_or(name);
                // Drop references to epsilon rules
                if self.epsilon_rules.contains(resolved) {
                    return Ok(String::new());
                }
                Ok(to_antlr4_name(resolved))
            }

            RuleBody::CrossRef(cross_ref) => {
                let resolved = self
                    .inline_map
                    .get(&cross_ref.rule_name)
                    .unwrap_or(&cross_ref.rule_name);
                if self.epsilon_rules.contains(resolved) {
                    return Ok(String::new());
                }
                Ok(to_antlr4_name(resolved))
            }

            RuleBody::Sequence(items) => {
                let parts: Result<Vec<_>, _> = items.iter().map(|i| self.emit_body(i)).collect();
                let parts: Vec<_> = parts?.into_iter().filter(|s| !s.is_empty()).collect();
                match parts.len() {
                    0 => Ok(String::new()),
                    1 => Ok(parts.into_iter().next().unwrap()),
                    _ => Ok(parts.join(" ")),
                }
            }

            RuleBody::Choice(items) => {
                let parts: Result<Vec<_>, _> = items.iter().map(|i| self.emit_body(i)).collect();
                let parts: Vec<_> = parts?.into_iter().filter(|s| !s.is_empty()).collect();
                match parts.len() {
                    0 => Ok(String::new()),
                    1 => Ok(parts.into_iter().next().unwrap()),
                    _ => Ok(parts.join("\n    | ")),
                }
            }

            RuleBody::Optional(inner) => {
                let inner_str = self.emit_body(inner)?;
                if inner_str.is_empty() {
                    return Ok(String::new());
                }
                // Avoid double-optional: if inner already ends with ?, don't wrap again
                if inner_str.ends_with('?') {
                    return Ok(inner_str);
                }
                if needs_grouping(&inner_str) {
                    Ok(format!("({})?", inner_str))
                } else {
                    Ok(format!("{}?", inner_str))
                }
            }

            RuleBody::Repeat(inner) => {
                let inner_str = self.emit_body(inner)?;
                if inner_str.is_empty() {
                    return Ok(String::new());
                }
                if needs_grouping(&inner_str) {
                    Ok(format!("({})*", inner_str))
                } else {
                    Ok(format!("{}*", inner_str))
                }
            }

            RuleBody::Repeat1(inner) => {
                let inner_str = self.emit_body(inner)?;
                if inner_str.is_empty() {
                    return Ok(String::new());
                }
                if needs_grouping(&inner_str) {
                    Ok(format!("({})+", inner_str))
                } else {
                    Ok(format!("{}+", inner_str))
                }
            }

            RuleBody::Group(inner) => {
                let inner_str = self.emit_body(inner)?;
                if inner_str.is_empty() {
                    return Ok(String::new());
                }
                Ok(format!("({})", inner_str))
            }

            // Assignments: strip the property binding, emit only the value
            RuleBody::Assignment(assignment) => self.emit_body(&assignment.value),

            // Boolean flags: emit as optional keyword
            RuleBody::BooleanFlag(flag) => Ok(format!("'{}'?", escape_antlr4(&flag.terminal))),

            // Semantic actions: no syntactic content
            RuleBody::SemanticAction(_) => Ok(String::new()),
        }
    }

    fn should_skip_lexer_rule(&self, name: &str) -> bool {
        const SKIP: &[&str] = &[
            "LINE_TERMINATOR",
            "LINE_TEXT",
            "WHITE_SPACE",
            "BASIC_INITIAL_CHARACTER",
            "BASIC_NAME_CHARACTER",
            "ALPHABETIC_CHARACTER",
            "DECIMAL_DIGIT",
            "NAME_CHARACTER",
            "UNRESTRICTED_NAME_CHARACTER",
            "ESCAPE_SEQUENCE",
            "STRING_CHARACTER",
            "SINGLE_QUOTE",
            "NAME",
            "BASIC_NAME",
            "UNRESTRICTED_NAME",
            "STRING_VALUE",
            "DECIMAL_VALUE",
            "EXPONENTIAL_VALUE",
            "REAL_VALUE",
            "SINGLE_LINE_NOTE",
            "MULTILINE_NOTE",
            "REGULAR_COMMENT",
            "COMMENT_TEXT",
            "COMMENT_LINE_TEXT",
            "PREFIX_COMMENT",
            // Never referenced by parser rules; cause warning(184) overlaps
            "RESERVED_KEYWORD",
            "RESERVED_SYMBOL",
        ];
        SKIP.contains(&name)
    }

    fn emit_builtin_lexer_rules(&mut self) {
        self.output.push_str(concat!(
            "// ─── Built-in lexer rules ───────────────────────────────\n",
            "// These replace the KeBNF lexical grammar with ANTLR4-native\n",
            "// patterns for whitespace, comments, names, and literals.\n\n",
        ));
        self.output
            .push_str("WS\n    : [ \\t\\r\\n]+ -> skip\n    ;\n\n");
        // MULTILINE_NOTE must precede both SINGLE_LINE_COMMENT and REGULAR_COMMENT:
        // - Before SINGLE_LINE_COMMENT so multi-line //*...\n...*/ blocks are captured
        //   as notes, not truncated as line comments.
        // - Before REGULAR_COMMENT so '//*' matches the note rule (first-match-wins).
        self.output.push_str(
            "MULTILINE_NOTE\n    : '//*' .*? '*/' -> channel(HIDDEN)\n    ;\n\n",
        );
        self.output.push_str(
            "SINGLE_LINE_COMMENT\n    : '//' ~[\\r\\n]* -> channel(HIDDEN)\n    ;\n\n",
        );
        // REGULAR_COMMENT is parser-visible — used as structured comment body
        // in Comment, Documentation, and TextualRepresentation rules.
        self.output
            .push_str("REGULAR_COMMENT\n    : '/*' .*? '*/'\n    ;\n\n");
        self.output
            .push_str("NAME\n    : BASIC_NAME | UNRESTRICTED_NAME\n    ;\n\n");
        self.output
            .push_str("fragment BASIC_NAME\n    : [a-zA-Z_] [a-zA-Z0-9_]*\n    ;\n\n");
        self.output.push_str(
            "fragment UNRESTRICTED_NAME\n    : '\\'' (~['\\\\] | '\\\\' .)* '\\''\n    ;\n\n",
        );
        self.output
            .push_str("STRING_VALUE\n    : '\"' (~[\"\\\\] | '\\\\' .)* '\"'\n    ;\n\n");
        self.output
            .push_str("DECIMAL_VALUE\n    : [0-9]+\n    ;\n\n");
        self.output.push_str(concat!(
            "REAL_VALUE\n",
            "    : [0-9]+ '.' [0-9]* ([eE] [+-]? [0-9]+)?\n",
            "    | '.' [0-9]+ ([eE] [+-]? [0-9]+)?\n",
            "    ;\n\n",
        ));
        self.output
            .push_str("EXPONENTIAL_VALUE\n    : [0-9]+ [eE] [+-]? [0-9]+\n    ;\n\n");
    }

    fn emit_stub_rules(&mut self) {
        if self.undefined_refs.is_empty() {
            return;
        }

        self.output.push_str(concat!(
            "// ─── Stub rules ─────────────────────────────────────────\n",
            "// These rules are referenced in the KeBNF source but not defined.\n",
            "// They may be defined in a different spec file or in the metamodel.\n\n",
        ));
        let mut refs: Vec<_> = self.undefined_refs.iter().cloned().collect();
        refs.sort();
        for name in refs {
            let antlr_name = to_antlr4_name(&name);
            self.output.push_str(&format!(
                "{}\n    : NAME // stub: {} not defined in KeBNF source\n    ;\n\n",
                antlr_name, name
            ));
        }
    }

    fn collect_undefined_refs(&mut self, rules: &[Rule]) {
        for rule in rules {
            self.collect_refs_from_body(&rule.body);
        }
        // Remove epsilon rules from undefined refs (we intentionally drop them)
        self.undefined_refs
            .retain(|name| !self.epsilon_rules.contains(name));
    }

    fn collect_refs_from_body(&mut self, body: &RuleBody) {
        match body {
            RuleBody::RuleRef(name) => {
                if !self.rule_names.contains(name) {
                    self.undefined_refs.insert(name.clone());
                }
            }
            RuleBody::CrossRef(cross_ref) => {
                if !self.rule_names.contains(&cross_ref.rule_name) {
                    self.undefined_refs.insert(cross_ref.rule_name.clone());
                }
            }
            RuleBody::Sequence(items) | RuleBody::Choice(items) => {
                for item in items {
                    self.collect_refs_from_body(item);
                }
            }
            RuleBody::Optional(inner)
            | RuleBody::Repeat(inner)
            | RuleBody::Repeat1(inner)
            | RuleBody::Group(inner) => {
                self.collect_refs_from_body(inner);
            }
            RuleBody::Assignment(assignment) => {
                self.collect_refs_from_body(&assignment.value);
            }
            _ => {}
        }
    }
}

// --- Helper functions ---

/// ANTLR4 reserved words that cannot be used as rule names.
const ANTLR4_RESERVED: &[&str] = &[
    "import", "fragment", "lexer", "parser", "grammar", "returns",
    "locals", "throws", "catch", "finally", "mode", "options",
    "tokens", "channels",
];

/// KeBNF symbol alias tokens that must be emitted as parser rules, not lexer rules.
/// These have multi-token keyword alternatives (e.g., `'typed' 'by'`) that only
/// work correctly as parser rules where token sequences are supported.
const SYMBOL_ALIAS_TOKENS: &[&str] = &[
    "TYPED_BY", "SPECIALIZES", "SUBSETS", "REFERENCES",
    "CROSSES", "REDEFINES", "CONJUGATES", "DEFINED_BY",
];

fn is_symbol_alias(name: &str) -> bool {
    SYMBOL_ALIAS_TOKENS.contains(&name)
}

/// Convert a symbol alias ALL_CAPS name to a camelCase parser rule name.
/// TYPED_BY → typedBy_, SPECIALIZES → specializes_, etc.
/// Underscore suffix avoids collision with existing parser rules of the same concept.
fn symbol_alias_to_parser_name(name: &str) -> String {
    let mut result = String::new();
    for (i, part) in name.split('_').enumerate() {
        if i == 0 {
            result.push_str(&part.to_lowercase());
        } else {
            let mut chars = part.chars();
            if let Some(c) = chars.next() {
                result.push(c.to_ascii_uppercase());
                result.push_str(&chars.as_str().to_lowercase());
            }
        }
    }
    result.push('_');
    result
}

/// Convert a KeBNF rule name to ANTLR4 convention.
/// CamelCase -> camelCase (parser), ALL_CAPS -> ALL_CAPS (lexer).
/// Symbol aliases are converted to camelCase parser rules.
/// Reserved words get a `_` suffix.
fn to_antlr4_name(name: &str) -> String {
    if is_symbol_alias(name) {
        return symbol_alias_to_parser_name(name);
    }

    let base = if is_lexer_rule(name) {
        name.to_string()
    } else {
        let mut chars = name.chars();
        match chars.next() {
            None => return String::new(),
            Some(c) => {
                let lower: String = c.to_lowercase().collect();
                lower + chars.as_str()
            }
        }
    };

    if ANTLR4_RESERVED.contains(&base.as_str()) {
        format!("{}_", base)
    } else {
        base
    }
}

fn is_lexer_rule(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit())
}

fn escape_antlr4(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

pub(crate) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn needs_grouping(expr: &str) -> bool {
    expr.contains(' ') || expr.contains('|')
}

/// Check if a rule body is epsilon (matches empty string only).
fn body_is_epsilon(body: &RuleBody) -> bool {
    match body {
        RuleBody::Empty => true,
        RuleBody::SemanticAction(_) => true,
        RuleBody::Sequence(items) => items.iter().all(body_is_epsilon),
        RuleBody::Assignment(a) => body_is_epsilon(&a.value),
        _ => false,
    }
}

/// If a rule body is just a single RuleRef, return the target name.
fn get_single_ref_target(body: &RuleBody) -> Option<String> {
    match body {
        RuleBody::RuleRef(name) => Some(name.clone()),
        RuleBody::Assignment(a) => get_single_ref_target(&a.value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_antlr4_name_parser_rule() {
        assert_eq!(to_antlr4_name("PackageDeclaration"), "packageDeclaration");
        assert_eq!(to_antlr4_name("PartDefinition"), "partDefinition");
    }

    #[test]
    fn test_to_antlr4_name_lexer_rule() {
        assert_eq!(to_antlr4_name("NAME"), "NAME");
        assert_eq!(to_antlr4_name("DECIMAL_VALUE"), "DECIMAL_VALUE");
    }

    #[test]
    fn test_to_antlr4_name_reserved() {
        assert_eq!(to_antlr4_name("Import"), "import_");
        assert_eq!(to_antlr4_name("Fragment"), "fragment_");
    }

    #[test]
    fn test_is_lexer_rule() {
        assert!(is_lexer_rule("NAME"));
        assert!(is_lexer_rule("DECIMAL_VALUE"));
        assert!(!is_lexer_rule("PackageDeclaration"));
    }

    #[test]
    fn test_escape_antlr4() {
        assert_eq!(escape_antlr4("foo"), "foo");
        assert_eq!(escape_antlr4("it's"), "it\\'s");
    }

    #[test]
    fn test_needs_grouping() {
        assert!(!needs_grouping("NAME"));
        assert!(needs_grouping("NAME 'keyword'"));
        assert!(needs_grouping("NAME | 'keyword'"));
    }

    #[test]
    fn test_body_is_epsilon() {
        assert!(body_is_epsilon(&RuleBody::Empty));
        assert!(body_is_epsilon(&RuleBody::SemanticAction(SemanticAction {
            property: None,
            value: None,
            is_empty: true,
        })));
        assert!(!body_is_epsilon(&RuleBody::Terminal("x".to_string())));
    }

    #[test]
    fn test_emit_simple_terminal() {
        let rules = vec![Rule {
            name: "Foo".to_string(),
            produces_type: None,
            body: RuleBody::Terminal("bar".to_string()),
            span: 0..0,
            source_line: 0,
        }];
        let result = emit(&rules, "test").unwrap();
        assert!(result.contains("foo\n    : 'bar'\n    ;"));
    }

    #[test]
    fn test_emit_choice() {
        let rules = vec![Rule {
            name: "Foo".to_string(),
            produces_type: None,
            body: RuleBody::Choice(vec![
                RuleBody::Terminal("a".to_string()),
                RuleBody::Terminal("b".to_string()),
                RuleBody::Terminal("c".to_string()),
            ]),
            span: 0..0,
            source_line: 0,
        }];
        let result = emit(&rules, "test").unwrap();
        assert!(result.contains("'a'\n    | 'b'\n    | 'c'"));
    }

    #[test]
    fn test_emit_sequence() {
        let rules = vec![Rule {
            name: "Foo".to_string(),
            produces_type: None,
            body: RuleBody::Sequence(vec![
                RuleBody::Terminal("a".to_string()),
                RuleBody::RuleRef("Bar".to_string()),
                RuleBody::Terminal("c".to_string()),
            ]),
            span: 0..0,
            source_line: 0,
        }];
        let result = emit(&rules, "test").unwrap();
        assert!(result.contains("'a' bar 'c'"));
    }

    #[test]
    fn test_emit_optional() {
        let rules = vec![Rule {
            name: "Foo".to_string(),
            produces_type: None,
            body: RuleBody::Optional(Box::new(RuleBody::Terminal("bar".to_string()))),
            span: 0..0,
            source_line: 0,
        }];
        let result = emit(&rules, "test").unwrap();
        assert!(result.contains("'bar'?"));
    }

    #[test]
    fn test_emit_boolean_flag() {
        let rules = vec![Rule {
            name: "Foo".to_string(),
            produces_type: None,
            body: RuleBody::BooleanFlag(BooleanFlag {
                property: "isAbstract".to_string(),
                terminal: "abstract".to_string(),
                variable_prefix: None,
            }),
            span: 0..0,
            source_line: 0,
        }];
        let result = emit(&rules, "test").unwrap();
        assert!(result.contains("'abstract'?"));
    }

    #[test]
    fn test_epsilon_rules_dropped() {
        let rules = vec![
            Rule {
                name: "Foo".to_string(),
                produces_type: None,
                body: RuleBody::Sequence(vec![
                    RuleBody::Terminal("x".to_string()),
                    RuleBody::RuleRef("Bar".to_string()),
                ]),
                span: 0..0,
                source_line: 0,
            },
            Rule {
                name: "Bar".to_string(),
                produces_type: None,
                body: RuleBody::Empty,
                span: 0..0,
                source_line: 0,
            },
        ];
        let result = emit(&rules, "test").unwrap();
        // Bar is epsilon, so Foo should just be 'x' with no reference to bar
        assert!(result.contains("foo\n    : 'x'\n    ;"));
        // Bar should not appear as a rule
        assert!(!result.contains("\nbar\n"));
    }
}
