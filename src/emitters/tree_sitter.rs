use crate::ast::*;
use std::collections::{HashMap, HashSet};

pub fn emit(rules: &[Rule], grammar_name: &str) -> Result<String, EmitError> {
    let mut emitter = Emitter::new(grammar_name);
    emitter.emit_grammar(rules)
}

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

/// Definition: (KeBNF name, keyword, body_rule)
const DEFINITIONS: &[(&str, &str, &str)] = &[
    ("AttributeDefinition", "attribute", "definition_body"),
    ("EnumerationDefinition", "enum", "enumeration_body"),
    ("OccurrenceDefinition", "occurrence", "definition_body"),
    ("IndividualDefinition", "individual", "definition_body"),
    ("ItemDefinition", "item", "definition_body"),
    ("PartDefinition", "part", "definition_body"),
    ("PortDefinition", "port", "definition_body"),
    ("ConnectionDefinition", "connection", "definition_body"),
    ("InterfaceDefinition", "interface", "definition_body"),
    ("AllocationDefinition", "allocation", "definition_body"),
    ("FlowDefinition", "flow", "definition_body"),
    ("ActionDefinition", "action", "action_body"),
    ("StateDefinition", "state", "definition_body"),
    ("CalculationDefinition", "calc", "definition_body"),
    ("ConstraintDefinition", "constraint", "definition_body"),
    ("RequirementDefinition", "requirement", "definition_body"),
    ("ConcernDefinition", "concern", "definition_body"),
    ("CaseDefinition", "case", "definition_body"),
    ("AnalysisCaseDefinition", "analysis", "definition_body"),
    (
        "VerificationCaseDefinition",
        "verification",
        "definition_body",
    ),
    ("UseCaseDefinition", "use', 'case", "definition_body"),
    ("ViewDefinition", "view", "view_body"),
    ("ViewpointDefinition", "viewpoint", "definition_body"),
    ("RenderingDefinition", "rendering", "definition_body"),
    ("MetadataDefinition", "metadata", "metadata_body"),
];

/// Usage: (KeBNF name, keyword, body_rule)
const USAGES: &[(&str, &str, &str)] = &[
    ("AttributeUsage", "attribute", "usage_body"),
    ("EnumerationUsage", "enum", "usage_body"),
    ("OccurrenceUsage", "occurrence", "usage_body"),
    ("IndividualUsage", "individual", "usage_body"),
    ("ItemUsage", "item", "usage_body"),
    ("PartUsage", "part", "usage_body"),
    ("PortUsage", "port", "usage_body"),
    ("ConnectionUsage", "connection", "usage_body"),
    ("InterfaceUsage", "interface", "usage_body"),
    ("AllocationUsage", "allocation", "usage_body"),
    ("FlowUsage", "flow", "usage_body"),
    ("ActionUsage", "action", "action_body"),
    ("StateUsage", "state", "state_def_body"),
    ("CalculationUsage", "calc", "calculation_body"),
    ("ConstraintUsage", "constraint", "definition_body"),
    ("RequirementUsage", "requirement", "requirement_body"),
    ("ConcernUsage", "concern", "requirement_body"),
    ("CaseUsage", "case", "case_body"),
    ("AnalysisCaseUsage", "analysis", "case_body"),
    ("VerificationCaseUsage", "verification", "case_body"),
    ("UseCaseUsage", "use', 'case", "case_body"),
    ("ViewUsage", "view", "view_body"),
    ("ViewpointUsage", "viewpoint", "requirement_body"),
    ("RenderingUsage", "rendering", "usage_body"),
    ("MetadataUsage", "metadata", "metadata_body"),
    ("ReferenceUsage", "ref", "usage_body"),
    ("EventOccurrenceUsage", "event', 'occurrence", "usage_body"),
];

struct Emitter {
    grammar_name: String,
    indent: usize,
    output: String,
    rule_names: HashSet<String>,
    emitted_rules: HashSet<String>,
    inline_map: HashMap<String, String>,
    epsilon_rules: HashSet<String>,
    body_member_rules: HashSet<String>,
}

impl Emitter {
    fn new(grammar_name: &str) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            indent: 0,
            output: String::new(),
            rule_names: HashSet::new(),
            emitted_rules: HashSet::new(),
            inline_map: HashMap::new(),
            epsilon_rules: HashSet::new(),
            body_member_rules: HashSet::new(),
        }
    }

    fn emit_grammar(&mut self, rules: &[Rule]) -> Result<String, EmitError> {
        for rule in rules {
            self.rule_names.insert(rule.name.clone());
        }
        self.build_inline_map(rules);
        self.find_epsilon_rules(rules);
        self.find_body_member_rules(rules);

        self.emit_header();
        self.line("");
        self.line("module.exports = grammar({");
        self.indent += 1;
        self.line(&format!("name: '{}',", self.grammar_name));
        self.line("");
        self.line("extras: $ => [/\\s/, $.comment],");
        self.line("");
        self.emit_conflicts();
        self.line("rules: {");
        self.indent += 1;

        // Pattern-based rules (definitions, usages, body structure)
        self.emit_source_file();
        self.emit_definitions();
        self.emit_usages();
        self.emit_body_rules();
        self.emit_expression_rules();

        // Remaining KeBNF rules (non-pattern)
        for rule in rules {
            self.emit_rule(rule)?;
        }

        self.emit_builtin_rules();

        self.indent -= 1;
        self.line("},");
        self.indent -= 1;
        self.line("});");
        Ok(self.output.clone())
    }

    // --- Analysis ---

    fn build_inline_map(&mut self, rules: &[Rule]) {
        let mut direct: HashMap<String, String> = HashMap::new();
        for rule in rules {
            if let Some(target) = get_single_ref_target(&rule.body) {
                direct.insert(rule.name.clone(), target);
            }
        }
        // Skip all prefix rules -- they're inlined into definitions/usages
        for name in &[
            "BasicDefinitionPrefix",
            "DefinitionPrefix",
            "OccurrenceDefinitionPrefix",
            "BasicFeaturePrefix",
            "EndFeaturePrefix",
            "FeaturePrefix",
            "BasicUsagePrefix",
            "UnextendedUsagePrefix",
            "OccurrenceUsagePrefix",
            "RefPrefix",
            "EndUsagePrefix",
            "UsagePrefix",
            "TypePrefix",
            "MemberPrefix",
        ] {
            self.epsilon_rules.insert(name.to_string()); // treat as epsilon = drop references
        }
        // Expression redirects
        for (from, to) in &[
            ("NonFeatureChainPrimaryExpression", "PrimaryExpression"),
            ("BracketExpression", "PrimaryExpression"),
            ("IndexExpression", "PrimaryExpression"),
            ("SelectExpression", "PrimaryExpression"),
            ("CollectExpression", "PrimaryExpression"),
            ("FunctionOperationExpression", "PrimaryExpression"),
            ("FeatureChainExpression", "PrimaryExpression"),
            ("SequenceExpression", "PrimaryExpression"),
            ("BinaryOperatorExpression", "OwnedExpression"),
            ("ConditionalBinaryOperatorExpression", "OwnedExpression"),
            ("ClassificationExpression", "OwnedExpression"),
            ("UnaryOperatorExpression", "OwnedExpression"),
            ("ExtentExpression", "OwnedExpression"),
            ("ConditionalExpression", "OwnedExpression"),
        ] {
            direct.insert(from.to_string(), to.to_string());
        }
        // Resolve chains
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

    fn find_epsilon_rules(&mut self, rules: &[Rule]) {
        for rule in rules {
            if body_is_epsilon(&rule.body) {
                self.epsilon_rules.insert(rule.name.clone());
            }
        }
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
            RuleBody::Empty | RuleBody::SemanticAction(_) => true,
            RuleBody::RuleRef(name) => {
                let resolved = self.inline_map.get(name).unwrap_or(name);
                self.epsilon_rules.contains(resolved)
            }
            RuleBody::Assignment(a) => self.body_resolves_to_epsilon(&a.value),
            RuleBody::Sequence(items) => items.iter().all(|i| self.body_resolves_to_epsilon(i)),
            _ => false,
        }
    }

    fn can_match_empty(&self, body: &RuleBody) -> bool {
        match body {
            RuleBody::Empty | RuleBody::SemanticAction(_) => true,
            RuleBody::Terminal(_) | RuleBody::CrossRef(_) | RuleBody::Repeat1(_) => false,
            RuleBody::RuleRef(name) => {
                let resolved = self.inline_map.get(name).unwrap_or(name);
                self.epsilon_rules.contains(resolved)
            }
            RuleBody::Optional(_) | RuleBody::Repeat(_) | RuleBody::BooleanFlag(_) => true,
            RuleBody::Group(inner) => self.can_match_empty(inner),
            RuleBody::Sequence(items) => items.iter().all(|i| self.can_match_empty(i)),
            RuleBody::Choice(items) => items.iter().any(|i| self.can_match_empty(i)),
            RuleBody::Assignment(a) => self.can_match_empty(&a.value),
        }
    }

    fn find_body_member_rules(&mut self, rules: &[Rule]) {
        const PATTERNS: &[&str] = &[
            "BodyElement",
            "BodyItem",
            "BodyMember",
            "DefinitionMember",
            "UsageMember",
            "VariantMember",
            "DefinitionElement",
            "NonFeatureElement",
            "FeatureElement",
            "MemberElement",
            "UsageElement",
        ];
        for rule in rules {
            for pattern in PATTERNS {
                if rule.name.ends_with(pattern) {
                    self.body_member_rules.insert(rule.name.clone());
                }
            }
        }
    }

    // --- Pattern-based emission ---

    fn emit_header(&mut self) {
        self.line("/// <reference types=\"tree-sitter-cli/dsl\" />");
        self.line("// @ts-check");
        self.line("");
        self.line("/**");
        self.line(" * SysML v2 tree-sitter grammar generated from KeBNF specification.");
        self.line(" * Generated by kebnf -- https://gitlab.com/nomograph/kebnf");
        self.line(" *");
        self.line(" * Pattern-based emission: definitions and usages have inlined prefix");
        self.line(" * keywords for early disambiguation. No shared prefix rule.");
        self.line(" */");
    }

    fn emit_source_file(&mut self) {
        self.line("source_file: $ => repeat(choice($.package, $.library_package, $._definition, $._usage, $.comment_about, $._member)),");
        self.line("");
        self.emitted_rules.insert("source_file".to_string());

        // Package rules
        self.emitted_rules.insert("package".to_string());
        self.emitted_rules.insert("library_package".to_string());
        self.line("package: $ => seq(repeat($.prefix_metadata_annotation), 'package', optional($.identification), $.package_body),");
        self.line("");
        self.line("library_package: $ => seq(optional('standard'), 'library', repeat($.prefix_metadata_annotation), 'package', optional($.identification), $.package_body),");
        self.line("");
    }

    fn emit_definitions(&mut self) {
        // Mark all definition-related KeBNF rules as emitted
        for (name, _, _) in DEFINITIONS {
            self.emitted_rules.insert(to_snake_case(name));
        }
        // Also mark the intermediate rules we're replacing
        for name in &["Definition", "OccurrenceDefinition", "ExtendedDefinition"] {
            self.emitted_rules.insert(to_snake_case(name));
        }

        self.line("_definition: $ => choice(");
        self.indent += 1;
        for (i, (name, _, _)) in DEFINITIONS.iter().enumerate() {
            let comma = if i < DEFINITIONS.len() - 1 { "," } else { "" };
            self.line(&format!("$.{}{}", to_snake_case(name), comma));
        }
        self.indent -= 1;
        self.line("),");
        self.line("");

        for (name, keyword, body) in DEFINITIONS {
            let snake = to_snake_case(name);
            self.line(&format!(
                concat!(
                    "{}: $ => seq(",
                    "repeat($.prefix_metadata_annotation), ",
                    "optional($.visibility_indicator), ",
                    "optional('abstract'), ",
                    "optional(choice('individual', 'variation')), ",
                    "'{}', 'def', ",
                    "optional($.identification), ",
                    "optional($.superclassing_part), ",
                    "$.{}),"
                ),
                snake, keyword, body
            ));
            self.line("");
        }
    }

    fn emit_usages(&mut self) {
        for (name, _, _) in USAGES {
            self.emitted_rules.insert(to_snake_case(name));
        }
        for name in &[
            "Usage",
            "OccurrenceUsage",
            "ExtendedUsage",
            "DefaultReferenceUsage",
            "PortionUsage",
        ] {
            self.emitted_rules.insert(to_snake_case(name));
        }

        self.line("_usage: $ => choice(");
        self.indent += 1;
        for (i, (name, _, _)) in USAGES.iter().enumerate() {
            let comma = if i < USAGES.len() - 1 { "," } else { "" };
            self.line(&format!("$.{}{}", to_snake_case(name), comma));
        }
        self.indent -= 1;
        self.line("),");
        self.line("");

        for (name, keyword, body) in USAGES {
            let snake = to_snake_case(name);
            self.line(&format!(
                concat!(
                    "{}: $ => seq(",
                    "repeat($.prefix_metadata_annotation), ",
                    "optional($.visibility_indicator), ",
                    "optional($.feature_direction), ",
                    "optional('abstract'), ",
                    "optional('ref'), ",
                    "'{}', ",
                    "optional($.usage_declaration), ",
                    "optional($.multiplicity), ",
                    "optional($.feature_value), ",
                    "$.{}),"
                ),
                snake, keyword, body
            ));
            self.line("");
        }
    }

    fn emit_body_rules(&mut self) {
        // _member: everything that can appear in a body
        self.line("_member: $ => choice(");
        self.indent += 1;
        self.line("$._definition,");
        self.line("$._usage,");
        self.line("$.import,");
        self.line("$.dependency,");
        self.line("$.alias_member,");
        self.line("$.comment_about,");
        self.line("$.doc_comment,");
        self.line("$.connect_statement,");
        self.line("$.bind_statement,");
        self.line("$.flow_statement,");
        self.line("$.allocate_statement,");
        self.line("$.satisfy_statement,");
        self.line("$.assert_statement,");
        self.line("$.exhibit_statement,");
        self.line("$.succession_statement,");
        self.line("$.end_feature,");
        self.line("$.package,");
        self.line("$.library_package,");
        self.line("$.generic_feature,");
        self.line("$.redefinition_statement,");
        self.line("$.event_usage,");
        self.line("$.message_usage,");
        self.line("$.snapshot_usage,");
        self.line("$.variation_statement,");
        self.line("$.prefix_metadata_annotation,");
        self.line("$.metadata_statement,");
        self.line("$.metadata_about,");
        self.line("$.documentation_comment,");
        // Member wrapper rules excluded to avoid indirect recursion
        self.indent -= 1;
        self.line("),");
        self.line("");

        // Body types
        self.line("definition_body: $ => choice(';', seq('{', repeat(choice($._member, $.generic_feature, $.end_feature, $.doc_comment, $.owned_expression)), '}')),");
        self.line("");
        self.line("usage_body: $ => choice(';', seq('{', repeat(choice($._member, $.generic_feature, $.assert_statement, $.redefinition_statement, $.doc_comment, $.owned_expression)), '}')),");
        self.line("");
        self.line("action_body: $ => choice(';', seq('{', repeat(choice($._member, $.generic_feature, $.action_node_member, $.guarded_succession_member, $.first_statement, $.then_succession, $.if_then_statement, $.while_loop, $.for_loop, $.loop_action, $.send_statement, $.perform_statement, $.assign_statement, $.accept_then_statement, $.terminate_statement, $.succession_statement, $.doc_comment, $.redefinition_statement, $.owned_expression)), '}')),");
        self.line("");
        self.line("enumeration_body: $ => choice(';', seq('{', repeat(choice($._member, $.enumerated_value)), '}')),");
        self.line("");
        self.line("metadata_body: $ => choice(';', seq('{', repeat(choice($.metadata_body_feature, $.metadata_body_usage, $.generic_feature, $.metadata_statement, $.owned_expression)), '}')),");
        self.line("");
        self.line("requirement_body: $ => choice(';', seq('{', repeat(choice($._member, $.requirement_constraint_member, $.framed_concern_member, $.actor_member, $.stakeholder_member, $.subject_member, $.requirement_verification_member, $.assert_statement, $.require_statement, $.assume_statement, $.satisfy_statement, $.verify_statement, $.doc_comment)), '}')),");
        self.line("");
        self.line("case_body: $ => choice(';', seq('{', repeat(choice($._member, $.objective_member)), '}')),");
        self.line("");
        self.line("view_body: $ => choice(';', seq('{', repeat(choice($._member, $.view_rendering_member, $.expose_statement, $.render_statement, $.filter_statement, $.satisfy_statement, $.frame_statement)), '}')),");
        self.line("");
        self.line("state_def_body: $ => choice(';', seq('parallel', ';'), seq(optional('parallel'), '{', repeat(choice($._member, $.generic_feature, $.entry_action_member, $.do_action_member, $.exit_action_member, $.entry_statement, $.exit_statement, $.do_statement, $.transition_statement, $.accept_then_statement, $.first_statement, $.then_succession, $.doc_comment)), '}')),");
        self.line("");
        self.line("state_usage_body: $ => choice(';', seq('{', repeat(choice($._member, $.entry_action_member, $.do_action_member, $.exit_action_member)), '}')),");
        self.line("");
        self.line("calculation_body: $ => choice(';', seq('{', repeat(choice($._member, $.result_expression_member)), '}')),");
        self.line("");
        self.line("interface_body: $ => choice(';', seq('{', repeat(choice($._member, $.default_interface_end, $.end_feature)), '}')),");
        self.line("");

        // Mark body-related KeBNF rules as emitted
        for name in &[
            "definition_body",
            "usage_body",
            "action_body",
            "enumeration_body",
            "metadata_body",
            "requirement_body",
            "case_body",
            "view_body",
            "state_def_body",
            "state_usage_body",
            "calculation_body",
            "interface_body",
            "namespace_body",
            "type_body",
            "package_body",
        ] {
            self.emitted_rules.insert(name.to_string());
        }

        // Also mark body member rules as emitted (they're replaced by _member)
        for name in &self.body_member_rules.clone() {
            self.emitted_rules.insert(to_snake_case(name));
        }
    }

    fn emit_expression_rules(&mut self) {
        for name in &[
            "OwnedExpression",
            "ConditionalExpression",
            "ConditionalBinaryOperatorExpression",
            "BinaryOperatorExpression",
            "ClassificationExpression",
            "UnaryOperatorExpression",
            "ExtentExpression",
            "PrimaryExpression",
            "BracketExpression",
            "IndexExpression",
            "SequenceExpression",
            "SelectExpression",
            "CollectExpression",
            "FunctionOperationExpression",
            "FeatureChainExpression",
            "NonFeatureChainPrimaryExpression",
        ] {
            self.emitted_rules.insert(to_snake_case(name));
        }

        // Expression rules modeled on hand-tuned grammar pattern
        self.line("owned_expression: $ => choice(");
        self.indent += 1;
        // Literals and atoms
        self.line("$.literal_expression,");
        self.line("$.qualified_name,");
        self.line("seq('(', $.owned_expression, ')'),");
        self.line("$.null_expression,");
        // Metadata access (@ToolVariable)
        self.line("seq('@', $.qualified_name),");
        // Select expression (items.?{x > 0})
        self.line("prec.left(20, seq($.owned_expression, '.?', '{', repeat(choice($._member, $.generic_feature, $.owned_expression)), '}')),");
        // Binary operators (precedence tower)
        self.line("prec.left(1, seq($.owned_expression, '?', $.owned_expression, 'else', $.owned_expression)),");
        self.line("prec.left(2, seq($.owned_expression, '??', $.owned_expression)),");
        self.line("prec.left(3, seq($.owned_expression, 'implies', $.owned_expression)),");
        self.line("prec.left(4, seq($.owned_expression, choice('or', '|'), $.owned_expression)),");
        self.line("prec.left(5, seq($.owned_expression, 'xor', $.owned_expression)),");
        self.line("prec.left(6, seq($.owned_expression, choice('and', '&'), $.owned_expression)),");
        self.line("prec.left(7, seq($.owned_expression, choice('==', '!=', '===', '!=='), $.owned_expression)),");
        self.line("prec.left(8, seq($.owned_expression, choice('<', '>', '<=', '>='), $.owned_expression)),");
        self.line("prec.left(9, seq($.owned_expression, '..', $.owned_expression)),");
        self.line("prec.left(10, seq($.owned_expression, choice('+', '-'), $.owned_expression)),");
        self.line(
            "prec.left(11, seq($.owned_expression, choice('*', '/', '%'), $.owned_expression)),",
        );
        self.line("prec.right(12, seq($.owned_expression, '**', $.owned_expression)),");
        // Classification/cast
        self.line("prec.left(13, seq($.owned_expression, choice('istype', 'hastype', '@', 'as', 'meta'), $.qualified_name)),");
        // Unary
        self.line("prec(14, seq(choice('not', '~', '-', '+'), $.owned_expression)),");
        self.line("prec(15, seq('all', $.qualified_name)),");
        // Feature chain binding (::>)
        self.line("prec.left(16, seq($.owned_expression, '::>', $.owned_expression)),");
        // Feature chain (a.b.c)
        self.line("prec.left(16, seq($.owned_expression, '.', $.name)),");
        self.line("prec.left(16, seq($.owned_expression, '.?', $.name)),");
        // Function call
        self.line("prec.left(17, seq($.owned_expression, '(', optional(seq($.owned_expression, repeat(seq(',', $.owned_expression)))), ')')),");
        // Index/bracket
        self.line("prec.left(18, seq($.owned_expression, '[', $.owned_expression, ']')),");
        self.line("prec.left(18, seq($.owned_expression, '#', '(', $.owned_expression, ')')),");
        // Arrow (select/collect)
        self.line("prec.left(19, seq($.owned_expression, '->', $.name, '(', optional(seq($.owned_expression, repeat(seq(',', $.owned_expression)))), ')')),");
        // Sequence
        self.line("prec.left(0, seq($.owned_expression, ',', $.owned_expression)),");
        self.indent -= 1;
        self.line("),");
        self.line("");

        // Keep primary_expression as alias for backward compat
        self.line("primary_expression: $ => $.owned_expression,");
        self.line("");
    }

    fn emit_conflicts(&mut self) {
        self.line("conflicts: $ => [");
        self.indent += 1;
        // Definition/usage pairs share prefix keywords
        self.line("[$.owned_expression, $.primary_expression],");
        self.line("[$.qualified_name],");
        self.line("[$.qualified_name, $.identification],");
        self.line("[$.qualified_name, $.feature_identification],");
        self.line("[$.qualified_name, $.feature_identification, $.identification],");
        self.line("[$.qualified_name, $.connector_end, $.identification],");
        self.line("[$.feature_identification, $.identification],");
        self.line("[$.feature_identification, $.connector_end],");
        self.line("[$.identification, $.connector_end],");
        self.line("[$.feature_identification, $.interface_end, $.identification],");
        self.line("[$.qualified_name, $.interface_end],");
        self.line("[$.general_type, $.owned_feature_chaining],");
        self.line("[$.general_type, $.element_reference_member],");
        self.line("[$.general_type, $.instantiated_type_reference],");
        self.line("[$.general_type, $.instantiated_type_member],");
        self.line("[$.owned_feature_chaining, $.element_reference_member],");
        self.line("[$.connector, $.nary_connector_declaration],");
        self.line("[$.connector, $.binary_connector_declaration],");
        self.line("[$.connector_end, $.metadata_body_feature, $.variant_reference, $.metadata_body_usage],");
        self.line("[$.connector_end, $.multiplicity],");
        self.line("[$.metadata_body_feature, $.metadata_body_usage],");
        self.line("[$.redefines, $.metadata_body_feature, $.metadata_body_usage],");
        self.line("[$.redefines, $.metadata_body_feature],");
        self.line("[$.feature_specialization_part, $.variant_reference],");
        self.line("[$.enumerated_value, $.enumeration_usage],");
        self.line("[$.filter_package],");
        self.line("[$.metadata_body, $.definition_body],");
        self.line("[$.metadata_body, $.usage_body],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.comment_about, $.documentation_comment],");
        self.line("[$.source_file, $._member],");
        self.line("[$.enumeration_usage, $.usage],");
        self.line("[$.identification],");
        self.line("[$.merge_node, $.decision_node, $.join_node, $.fork_node],");
        self.line("[$.annotating_element, $.comment_about, $.documentation_comment],");
        self.line("[$.payload_feature_specialization_part],");
        self.line("[$.primary_expression],");
        self.line("[$.prefix_metadata_annotation, $.prefix_metadata_member],");
        self.line("[$.primary_expression, $.positional_argument_list],");
        self.line("[$.primary_expression, $.named_argument],");
        self.line("[$.typings, $.metadata_feature_declaration],");
        self.line("[$.qualified_name, $.usage_declaration],");
        self.line("[$.redefines, $.relationship_part],");
        self.line("[$.usage_declaration, $.relationship_part],");
        self.line("[$.first_statement],");
        self.line("[$.qualified_name, $.first_statement],");
        self.line("[$.usage_body, $.flow_statement],");
        self.line("[$.accept_then_statement],");
        self.line("[$.usage_body, $.connect_statement],");
        self.line("[$.usage_body, $.allocate_statement],");
        self.line("[$._member, $.definition_body],");
        self.line("[$.usage_body, $.end_feature],");
        self.line("[$.owned_expression, $.terminate_statement],");
        self.line("[$.owned_expression],");
        self.line("[$._member, $.usage_body],");
        self.line("[$.usage_body, $.generic_feature],");
        self.line("[$.reference_usage, $.generic_feature],");
        self.line("[$.usage_body, $.succession_statement],");
        self.line("[$.usage_body, $.exhibit_statement],");
        self.line("[$.owned_expression, $.owned_feature_chaining],");
        self.line("[$._member, $.action_body],");
        self.line("[$.generic_feature, $.usage],");
        self.line("[$.documentation, $.doc_comment],");
        self.line("[$.usage_body, $.import],");
        self.line("[$.owned_expression, $.feature_reference],");
        self.line("[$.satisfy_statement],");
        self.line("[$._member, $.owned_expression],");
        self.line("[$.feature_specialization_part],");
        self.line("[$._member, $.requirement_body],");
        self.line("[$._member, $.view_body],");
        self.line("[$.usage_body, $.transition_statement],");
        self.line("[$._member, $.state_def_body],");
        self.line("[$._member, $.assert_statement],");
        self.line("[$.action_body, $.first_statement],");
        self.line("[$.action_body, $.then_succession],");
        self.line("[$.package, $.attribute_definition, $.enumeration_definition, $.occurrence_definition, $.individual_definition, $.item_definition, $.part_definition, $.port_definition, $.connection_definition, $.interface_definition, $.allocation_definition, $.flow_definition, $.action_definition, $.state_definition, $.calculation_definition, $.constraint_definition, $.requirement_definition, $.concern_definition, $.case_definition, $.analysis_case_definition, $.verification_case_definition, $.use_case_definition, $.view_definition, $.viewpoint_definition, $.rendering_definition, $.metadata_definition, $.attribute_usage, $.enumeration_usage, $.occurrence_usage, $.individual_usage, $.item_usage, $.part_usage, $.port_usage, $.connection_usage, $.interface_usage, $.allocation_usage, $.flow_usage, $.action_usage, $.state_usage, $.calculation_usage, $.constraint_usage, $.requirement_usage, $.concern_usage, $.case_usage, $.analysis_case_usage, $.verification_case_usage, $.use_case_usage, $.view_usage, $.viewpoint_usage, $.rendering_usage, $.metadata_usage, $.reference_usage, $.event_occurrence_usage, $._member, $.dependency, $.snapshot_usage, $.message_usage, $.event_usage],");
        self.line("[$._member, $.control_node_prefix],");
        self.line("[$._member, $.action_node_prefix],");
        self.line("[$.owned_expression, $.general_type],");
        self.line("[$.metadata_statement],");
        self.line("[$.metadata_about],");
        self.indent -= 1;
        self.line("],");
        self.line("");
    }

    // --- Non-pattern rule emission ---

    fn emit_rule(&mut self, rule: &Rule) -> Result<(), EmitError> {
        let snake = to_snake_case(&rule.name);
        if self.emitted_rules.contains(&snake) {
            return Ok(());
        }
        self.emitted_rules.insert(snake.clone());
        if self.epsilon_rules.contains(&rule.name) {
            return Ok(());
        }
        if self.can_match_empty(&rule.body) {
            return Ok(());
        }
        if is_skip_lexical(&rule.name) {
            return Ok(());
        }

        let body = self.emit_body(&rule.body)?;
        if body.is_empty() || body == "seq()" {
            return Ok(());
        }

        let mut comment = String::new();
        if let Some(ref pt) = rule.produces_type {
            comment.push_str(&format!(" // : {}", pt));
        }

        // Wrap rules with internal ambiguity in prec.left
        let has_optional = body.contains("optional(") || body.contains("repeat(");
        let has_compound = body.starts_with("seq(") || body.starts_with("choice(");
        if has_optional && has_compound {
            self.line(&format!(
                "{}: $ => prec.left(0, {}),{}",
                snake, body, comment
            ));
        } else {
            self.line(&format!("{}: $ => {},{}", snake, body, comment));
        }
        self.line("");
        Ok(())
    }

    fn emit_body(&self, body: &RuleBody) -> Result<String, EmitError> {
        match body {
            RuleBody::Empty => Ok(String::new()),
            RuleBody::Terminal(s) => Ok(format!("'{}'", escape_terminal(s))),
            RuleBody::RuleRef(name) => {
                let resolved = self.inline_map.get(name).unwrap_or(name);
                if self.epsilon_rules.contains(resolved) {
                    return Ok(String::new());
                }
                if self.body_member_rules.contains(resolved) {
                    return Ok("$._member".to_string());
                }
                Ok(format!("$.{}", to_snake_case(resolved)))
            }
            RuleBody::CrossRef(cr) => {
                let resolved = self.inline_map.get(&cr.rule_name).unwrap_or(&cr.rule_name);
                if self.epsilon_rules.contains(resolved) {
                    return Ok(String::new());
                }
                Ok(format!("$.{}", to_snake_case(resolved)))
            }
            RuleBody::Sequence(items) => {
                let parts: Result<Vec<_>, _> = items.iter().map(|i| self.emit_body(i)).collect();
                let parts: Vec<_> = parts?.into_iter().filter(|s| !s.is_empty()).collect();
                match parts.len() {
                    0 => Ok(String::new()),
                    1 => Ok(parts.into_iter().next().unwrap()),
                    _ => Ok(format!("seq({})", parts.join(", "))),
                }
            }
            RuleBody::Choice(items) => {
                let parts: Result<Vec<_>, _> = items.iter().map(|i| self.emit_body(i)).collect();
                let parts: Vec<_> = parts?.into_iter().filter(|s| !s.is_empty()).collect();
                match parts.len() {
                    0 => Ok(String::new()),
                    1 => Ok(parts.into_iter().next().unwrap()),
                    _ => Ok(format!("choice({})", parts.join(", "))),
                }
            }
            RuleBody::Optional(inner) => {
                let s = self.emit_body(inner)?;
                if s.is_empty() {
                    Ok(String::new())
                } else {
                    Ok(format!("optional({})", s))
                }
            }
            RuleBody::Repeat(inner) => {
                let s = self.emit_body(inner)?;
                if s.is_empty() {
                    Ok(String::new())
                } else {
                    Ok(format!("repeat({})", s))
                }
            }
            RuleBody::Repeat1(inner) => {
                let s = self.emit_body(inner)?;
                if s.is_empty() {
                    Ok(String::new())
                } else {
                    Ok(format!("repeat1({})", s))
                }
            }
            RuleBody::Group(inner) => self.emit_body(inner),
            RuleBody::Assignment(a) => self.emit_body(&a.value),
            RuleBody::BooleanFlag(f) => Ok(format!("optional('{}')", escape_terminal(&f.terminal))),
            RuleBody::SemanticAction(_) => Ok(String::new()),
        }
    }

    fn emit_builtin_rules(&mut self) {
        // Non-empty replacements for can-match-empty rules
        // Fix typing rules -- the KeBNF parser drops the type reference from assignments
        self.line("typed_by: $ => seq(choice(':', seq('typed', 'by')), $.general_type),");
        self.line("");
        self.line("typings: $ => prec.left(0, seq(choice(':', seq('typed', 'by')), $.general_type, repeat(seq(',', $.general_type)))),");
        self.line("");
        self.line("subsettings: $ => prec.left(0, seq($.subsets, $.general_type, repeat(seq(',', $.general_type)))),");
        self.line("");
        self.line("redefinitions: $ => prec.left(0, seq($.redefines, $.general_type, repeat(seq(',', $.general_type)))),");
        self.line("");
        self.line("feature_direction: $ => choice('in', 'out', 'inout'),");
        self.line("");
        // Multiplicity: [N], [0..5], [*], [0..*]
        self.line("multiplicity: $ => seq('[', optional(choice(seq($.multiplicity_bound, '..', choice($.multiplicity_bound, '*')), $.multiplicity_bound, '*')), ']'),");
        self.line("");
        self.line("multiplicity_bound: $ => choice($.decimal_value, $.name),");
        self.line("");
        // Import (fix: visibility is optional)
        self.line("import: $ => seq(optional($.visibility_indicator), 'import', optional('all'), $.import_declaration, choice(';', $.usage_body)),");
        self.line("");
        // Comment about
        self.line("comment_about: $ => prec(1, seq('comment', optional(seq('about', $.qualified_name, repeat(seq(',', $.qualified_name)))), optional($.regular_comment))),");
        self.line("");
        // Redefinition statement (:>> x;)
        self.line("redefinition_statement: $ => prec(1, seq(':>>', $.owned_expression, optional($.feature_value), ';')),");
        self.line("");
        // Feature value (= expression)
        self.line("feature_value: $ => seq(choice('=', ':=', seq('default', choice('=', ':='))), $.owned_expression),");
        self.line("");
        // Action body statements
        // first start; | first action focus : Focus; | first a then b;
        self.line("first_statement: $ => prec.left(0, seq('first', choice('start', seq(choice('action', 'state'), optional($.usage_declaration), optional($.feature_value), choice(';', $.action_body)), $.owned_expression), optional(seq('then', choice('done', seq(choice('action', 'state'), optional($.usage_declaration), optional($.feature_value), choice(';', $.action_body)), $.owned_expression, $.action_body))), optional(';'))),");
        self.line("");
        self.line("then_succession: $ => prec.left(0, seq('then', choice('done', seq(choice('action', 'state'), optional($.usage_declaration), optional($.feature_value), choice(';', $.action_body)), seq($.owned_expression, optional($.action_body)), $.if_then_statement, $.while_loop, $.for_loop, $.send_statement, $.perform_statement, $.assign_statement, $.accept_then_statement), optional(';'))),");
        self.line("");
        self.line("if_then_statement: $ => prec.left(0, seq('if', $.owned_expression, 'then', choice($.owned_expression, $.action_body), optional(seq('else', choice($.owned_expression, $.action_body, $.if_then_statement))), optional(';'))),");
        self.line("");
        self.line("while_loop: $ => prec(1, seq('while', $.owned_expression, $.action_body)),");
        self.line("");
        self.line("loop_action: $ => prec.left(0, seq('loop', optional('action'), optional($.usage_declaration), $.action_body, optional(seq('until', $.owned_expression, ';')))),");
        self.line("");
        self.line(
            "for_loop: $ => prec(1, seq('for', $.name, 'in', $.owned_expression, $.action_body)),",
        );
        self.line("");
        self.line("send_statement: $ => prec(1, seq('send', $.owned_expression, choice(seq('to', $.owned_expression), seq('via', $.owned_expression)), ';')),");
        self.line("");
        self.line("perform_statement: $ => prec(1, seq('perform', optional('action'), optional($.usage_declaration), $.action_body)),");
        self.line("");
        self.line("assign_statement: $ => prec(1, seq('assign', $.qualified_name, ':=', $.owned_expression, ';')),");
        self.line("");
        self.line("accept_then_statement: $ => prec(1, seq('accept', choice(seq('when', $.owned_expression), seq('after', $.owned_expression)), optional(seq('then', choice($.owned_expression, $.action_body))), optional(';'))),");
        self.line("");
        self.line("terminate_statement: $ => prec(1, seq('terminate', $.qualified_name, ';')),");
        self.line("");
        // Metadata about: metadata SafetyFeature about vehicle::x, vehicle::y;
        self.line("metadata_about: $ => prec(1, seq('metadata', optional($.usage_declaration), 'about', $.qualified_name, repeat(seq(',', $.qualified_name)), optional(';'))),");
        self.line("");
        // Metadata statement (@Annotation;)
        self.line("metadata_statement: $ => prec(1, seq('@', $.qualified_name, optional(seq('{', repeat(choice($.generic_feature, $.owned_expression)), '}')), optional(';'))),");
        self.line("");
        // Snapshot/timeslice usage
        self.line("snapshot_usage: $ => prec(1, seq(repeat($.prefix_metadata_annotation), optional($.visibility_indicator), 'snapshot', optional($.usage_declaration), optional($.multiplicity), optional($.feature_value), $.usage_body)),");
        self.line("");
        // Message usage: abstract message messages : Message[0..*] nonunique :> transfers { }
        self.line("message_usage: $ => prec(1, seq(repeat($.prefix_metadata_annotation), optional($.visibility_indicator), optional('abstract'), 'message', optional($.usage_declaration), repeat(choice($.feature_specialization, $.multiplicity, 'nonunique')), optional($.feature_value), $.usage_body)),");
        self.line("");
        // Override allocation_usage: allocation myAlloc allocate sw to hw;
        self.line("allocation_usage: $ => seq(repeat($.prefix_metadata_annotation), optional($.visibility_indicator), optional($.feature_direction), optional('abstract'), optional('ref'), 'allocation', optional($.usage_declaration), optional(seq('allocate', $.owned_expression, optional(seq('to', $.owned_expression)))), optional($.feature_value), $.usage_body),");
        self.line("");
        // Event usage (event without occurrence)
        self.line("event_usage: $ => prec(1, seq(repeat($.prefix_metadata_annotation), optional($.visibility_indicator), optional('abstract'), 'event', optional($.usage_declaration), optional($.multiplicity), optional($.feature_value), $.usage_body)),");
        self.line("");
        // Variation keyword
        self.line("variation_statement: $ => prec(1, seq('variation', optional('perform'), optional('action'), optional($.usage_declaration), $.action_body)),");
        self.line("");
        // Generic feature (bare usage without keyword: in x : T;)
        self.line("generic_feature: $ => prec(-1, choice(seq(optional($.feature_direction), optional('abstract'), optional('ref'), $.usage_declaration, optional($.multiplicity), optional($.feature_value), choice(';', $.usage_body)), seq($.feature_specialization_part, optional($.multiplicity), optional($.feature_value), choice(';', $.usage_body)))),");
        self.line("");
        // End features (connection/interface endpoints)
        // end [1] part supplier : FuelPort;  OR  end :>> source : USBPort;
        self.line("end_feature: $ => prec.left(0, seq('end', optional($.multiplicity), optional(choice('part', 'port', 'occurrence')), optional(choice($.usage_declaration, seq(':>>', $.owned_expression, optional($.feature_specialization_part)))), optional($.feature_value), choice(';', $.usage_body))),");
        self.line("");
        // Connection/flow statements
        self.line("connect_statement: $ => prec(1, seq('connect', $.owned_expression, 'to', $.owned_expression, choice(';', $.usage_body))),");
        self.line("");
        self.line(
            "bind_statement: $ => seq('bind', $.owned_expression, '=', $.owned_expression, ';'),",
        );
        self.line("");
        self.line("flow_statement: $ => prec(1, seq(optional('succession'), 'flow', optional('from'), $.owned_expression, 'to', $.owned_expression, choice(';', $.usage_body))),");
        self.line("");
        self.line("allocate_statement: $ => prec(1, seq('allocate', $.owned_expression, optional(seq('to', $.owned_expression)), choice(';', $.usage_body))),");
        self.line("");
        self.line("exhibit_statement: $ => prec(1, seq('exhibit', choice(seq(optional($.usage_declaration), choice(';', $.usage_body)), seq($.owned_expression, choice(';', $.usage_body))))),");
        self.line("");
        self.line("succession_statement: $ => prec.left(0, seq('succession', optional(choice($.usage_declaration, $.feature_specialization_part)), optional($.multiplicity), optional(seq('first', optional($.multiplicity), $.owned_expression)), optional(seq('then', optional($.multiplicity), $.owned_expression)), choice(';', $.usage_body))),");
        self.line("");
        // Requirement body statements
        self.line("satisfy_statement: $ => prec(1, seq('satisfy', $.qualified_name, optional(seq('by', $.qualified_name)), optional(';'))),");
        self.line("");
        self.line("verify_statement: $ => prec(1, seq('verify', $.qualified_name, ';')),");
        self.line("");
        self.line("assert_statement: $ => prec(1, seq(optional('private'), 'assert', optional('constraint'), optional($.usage_declaration), choice(seq('{', repeat(choice($._member, $.generic_feature, $.owned_expression)), '}'), ';'))),");
        self.line("");
        self.line("require_statement: $ => prec(1, seq('require', optional($.prefix_metadata_annotation), optional('constraint'), choice(seq('{', optional($.owned_expression), '}'), seq($.qualified_name, optional(';'))))),");
        self.line("");
        self.line("assume_statement: $ => prec(1, seq('assume', optional($.prefix_metadata_annotation), optional('constraint'), choice(seq('{', optional($.owned_expression), '}'), seq($.qualified_name, optional(';'))))),");
        self.line("");
        // State body statements
        self.line("entry_statement: $ => prec(1, seq('entry', choice(seq('action', optional($.usage_declaration), $.action_body), seq($.assign_statement), ';'))),");
        self.line("");
        self.line("exit_statement: $ => prec(1, seq('exit', choice(seq('action', optional($.usage_declaration), $.action_body), ';'))),");
        self.line("");
        self.line("do_statement: $ => prec(1, seq('do', choice(seq('action', optional($.usage_declaration), $.action_body), seq($.qualified_name, ';')))),");
        self.line("");
        self.line("transition_statement: $ => prec.left(0, seq('transition', optional($.usage_declaration), optional(seq('first', $.qualified_name)), optional(seq('accept', $.owned_expression)), optional(seq('then', $.qualified_name)), choice(';', $.usage_body))),");
        self.line("");
        // View body statements
        // expose vehicle::**; | expose vehicle::**[@Safety];
        self.line("expose_statement: $ => prec(1, seq('expose', $.qualified_name, optional(seq('::', choice('*', '**'))), optional(seq('[', $.owned_expression, ']')), ';')),");
        self.line("");
        self.line("render_statement: $ => prec(1, seq('render', optional($.usage_declaration), choice(';', $.usage_body))),");
        self.line("");
        self.line("filter_statement: $ => prec(1, seq('filter', choice(seq('@', $.qualified_name), $.owned_expression), ';')),");
        self.line("");
        self.line("frame_statement: $ => prec(1, seq('frame', $.qualified_name, ';')),");
        self.line("");
        // Documentation
        self.line("doc_comment: $ => prec(1, seq('doc', optional($.regular_comment))),");
        self.line("");
        // usage_declaration: just identification + specialization (multiplicity handled in usage pattern)
        self.line("");
        self.line("");
        self.line("identification: $ => choice(seq('<', $.name, '>', optional($.name)), $.name),");
        self.line("");
        self.line(
            "feature_identification: $ => choice(seq('<', $.name, '>', optional($.name)), $.name),",
        );
        self.line("");
        self.line("multiplicity_part: $ => $.multiplicity_range,");
        self.line("");
        self.line("binding_connector_declaration: $ => $.feature_declaration,");
        self.line("");
        self.line("succession_declaration: $ => $.feature_declaration,");
        self.line("");
        self.line("function_body_part: $ => $.result_expression_member,");
        self.line("");
        self.line("source_end: $ => $._member,");
        self.line("");
        self.line("port_conjugation: $ => seq('~', $.qualified_name),");
        self.line("");
        self.line("assignment_target_parameter: $ => $.feature_member,");
        self.line("");
        self.line("calculation_body_part: $ => $.result_expression_member,");
        self.line("");
        self.line("namespace_body: $ => choice(';', seq('{', repeat($._member), '}')),");
        self.line("");
        self.line("type_body: $ => choice(';', seq('{', repeat($._member), '}')),");
        self.line("");
        self.line("package_body: $ => choice(';', seq('{', repeat($._member), '}')),");
        self.line("");
        self.line("comment_text: $ => repeat1($.comment_line_text),");
        self.line("");
        self.line("nary_connector_declaration: $ => $.feature_declaration,");
        self.line("");
        self.line("nary_interface_part: $ => $.feature_declaration,");
        self.line("");
        self.line("feature_specialization_part: $ => repeat1($.feature_specialization),");
        self.line("");
        self.line("effect_behavior_usage: $ => choice($.transition_perform_action_usage, $.transition_accept_action_usage),");
        self.line("");
        self.line("root_namespace: $ => repeat1($._member),");
        self.line("");

        self.line("action_node_prefix: $ => choice($.prefix_metadata_annotation, $.visibility_indicator),");
        self.line("");
        self.line("control_node_prefix: $ => choice($.prefix_metadata_annotation, $.visibility_indicator),");
        self.line("");
        self.line("comment_about: $ => seq($.comment, optional($.identification)),");
        self.line("");
        self.line("documentation_comment: $ => seq($.comment),");
        self.line("");
        self.line("usage: $ => seq(optional($.usage_declaration), $.usage_body),");
        self.line("");
        self.line("calculation_usage_declaration: $ => $.usage_declaration,");
        self.line("");
        self.line(
            "relationship_part: $ => choice($.feature_specialization, seq(':>>', $.general_type)),",
        );
        self.line("");
        // Lexical rules
        self.line("line_terminator: $ => /\\r?\\n/,");
        self.line("");
        self.line("comment_line_text: $ => /[^\\n]*/,");
        self.line("");
        self.line("regular_comment: $ => /\\/\\*[^*]*\\*+(?:[^/*][^*]*\\*+)*\\//,");
        self.line("");
        self.line("comment: $ => token(choice(seq('//', /[^\\n]*/), seq('/*', /[^*]*\\*+([^/*][^*]*\\*+)*/, '/'))),");
        self.line("");
        self.line("name: $ => choice($.basic_name, $.unrestricted_name),");
        self.line("");
        self.line("basic_name: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,");
        self.line("");
        self.line("unrestricted_name: $ => /'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'/,");
        self.line("");
        self.line("string_value: $ => /\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"/,");
        self.line("");
        self.line("decimal_value: $ => /[0-9]+/,");
        self.line("");
        self.line("exponential_value: $ => /[0-9]+[eE][+-]?[0-9]+/,");
        self.line("");
        // Require at least one digit after dot to avoid consuming 1. in 1..10
        self.line("real_value: $ => choice(/[0-9]+\\.[0-9]+([eE][+-]?[0-9]+)?/, /\\.[0-9]+([eE][+-]?[0-9]+)?/),");
        self.line("");
    }

    fn line(&mut self, text: &str) {
        if text.is_empty() {
            self.output.push('\n');
        } else {
            let indent = "  ".repeat(self.indent);
            self.output.push_str(&indent);
            self.output.push_str(text);
            self.output.push('\n');
        }
    }
}

// --- Helpers ---

fn to_snake_case(name: &str) -> String {
    if name.chars().all(|c| c.is_uppercase() || c == '_') {
        return name.to_lowercase();
    }
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn escape_terminal(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn body_is_epsilon(body: &RuleBody) -> bool {
    match body {
        RuleBody::Empty | RuleBody::SemanticAction(_) => true,
        RuleBody::Sequence(items) => items.iter().all(body_is_epsilon),
        RuleBody::Assignment(a) => body_is_epsilon(&a.value),
        _ => false,
    }
}

fn get_single_ref_target(body: &RuleBody) -> Option<String> {
    match body {
        RuleBody::RuleRef(name) => Some(name.clone()),
        RuleBody::Assignment(a) => get_single_ref_target(&a.value),
        _ => None,
    }
}

fn is_skip_lexical(name: &str) -> bool {
    const SKIP: &[&str] = &[
        "LINE_TERMINATOR",
        "LINE_TEXT",
        "WHITE_SPACE",
        "SINGLE_LINE_NOTE",
        "MULTILINE_NOTE",
        "REGULAR_COMMENT",
        "COMMENT_TEXT",
        "COMMENT_LINE_TEXT",
        "PREFIX_COMMENT",
        "BASIC_INITIAL_CHARACTER",
        "BASIC_NAME_CHARACTER",
        "ALPHABETIC_CHARACTER",
        "DECIMAL_DIGIT",
        "NAME_CHARACTER",
        "UNRESTRICTED_NAME_CHARACTER",
        "ESCAPE_SEQUENCE",
        "STRING_CHARACTER",
        "NAME",
        "BASIC_NAME",
        "SINGLE_QUOTE",
        "UNRESTRICTED_NAME",
        "DECIMAL_VALUE",
        "EXPONENTIAL_VALUE",
        "REAL_VALUE",
        "STRING_VALUE",
    ];
    SKIP.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("FooBar"), "foo_bar");
        assert_eq!(to_snake_case("FOO"), "foo");
    }

    #[test]
    fn test_body_is_epsilon() {
        assert!(body_is_epsilon(&RuleBody::Empty));
        assert!(!body_is_epsilon(&RuleBody::Terminal("x".to_string())));
    }
}
