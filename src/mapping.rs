use crate::ast::*;
use crate::naming::to_snake_case;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub version: String,
    pub generated_at: String,
    pub rules: HashMap<String, RuleMapping>,
    pub statistics: Statistics,
    pub ambiguity_resolutions: AmbiguityResolutions,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AmbiguityResolutions {
    pub definition_usage_conflicts: Vec<DefinitionUsageConflict>,
    pub empty_string_replacements: Vec<EmptyStringReplacement>,
    pub merged_rules: Vec<MergedRule>,
    pub additional_conflicts: Vec<AdditionalConflict>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefinitionUsageConflict {
    pub definition_rule: String,
    pub usage_rule: String,
    pub shared_prefix: String,
    pub mechanism: String,
    pub rationale: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyStringReplacement {
    pub original_rule: String,
    pub replacement_rule: String,
    pub original_body: String,
    pub replacement_body: String,
    pub semantic_impact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergedRule {
    pub merged_rules: Vec<String>,
    pub resulting_rule: String,
    pub rationale: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalConflict {
    pub rule_a: String,
    pub rule_b: String,
    pub conflict_type: String,
    pub rationale: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleMapping {
    pub kebnf_name: String,
    pub tree_sitter_name: String,
    pub source_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub produces_type: Option<String>,
    pub stripped_annotations: Vec<StrippedAnnotation>,
    pub needs_manual_review: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrippedAnnotation {
    pub kind: AnnotationKind,
    pub property: Option<String>,
    pub value: Option<String>,
    pub operator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationKind {
    TypeAnnotation,
    PropertyAssignment,
    PropertyAppend,
    BooleanFlag,
    CrossReference,
    NegatedCrossReference,
    SemanticAction,
    EmptySemanticAction,
    VariablePrefix,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Statistics {
    pub total_rules: usize,
    pub direct_conversion: usize,
    pub strip_and_convert: usize,
    pub best_effort: usize,
    pub manual_review: usize,
    pub by_annotation_type: HashMap<String, usize>,
}

pub fn generate(rules: &[Rule]) -> Result<String, Box<dyn std::error::Error>> {
    let mut rule_mappings = HashMap::new();
    let mut stats = Statistics {
        total_rules: rules.len(),
        ..Default::default()
    };

    for rule in rules {
        let stripped = collect_stripped_annotations(&rule.body);
        let needs_review = rule.needs_manual_review();

        if needs_review {
            stats.manual_review += 1;
        } else if stripped.iter().any(|a| {
            matches!(
                a.kind,
                AnnotationKind::SemanticAction | AnnotationKind::EmptySemanticAction
            )
        }) {
            stats.best_effort += 1;
        } else if !stripped.is_empty() {
            stats.strip_and_convert += 1;
        } else {
            stats.direct_conversion += 1;
        }

        for annotation in &stripped {
            let key = format!("{:?}", annotation.kind);
            *stats.by_annotation_type.entry(key).or_insert(0) += 1;
        }

        let mapping = RuleMapping {
            kebnf_name: rule.name.clone(),
            tree_sitter_name: to_snake_case(&rule.name),
            source_line: rule.source_line,
            produces_type: rule.produces_type.clone(),
            stripped_annotations: stripped,
            needs_manual_review: needs_review,
            review_reason: if needs_review {
                Some("Variable prefix detected - may need scoping disambiguation".to_string())
            } else {
                None
            },
        };

        rule_mappings.insert(rule.name.clone(), mapping);
    }

    let ambiguity_resolutions = generate_ambiguity_resolutions(rules);

    let mapping = Mapping {
        version: env!("CARGO_PKG_VERSION").to_string(),
        generated_at: chrono_lite_now(),
        rules: rule_mappings,
        statistics: stats,
        ambiguity_resolutions,
    };

    Ok(serde_json::to_string_pretty(&mapping)?)
}

pub fn compute_stats(rules: &[Rule]) -> Statistics {
    let mut stats = Statistics {
        total_rules: rules.len(),
        ..Default::default()
    };

    for rule in rules {
        let stripped = collect_stripped_annotations(&rule.body);
        let needs_review = rule.needs_manual_review();

        if needs_review {
            stats.manual_review += 1;
        } else if stripped.iter().any(|a| {
            matches!(
                a.kind,
                AnnotationKind::SemanticAction | AnnotationKind::EmptySemanticAction
            )
        }) {
            stats.best_effort += 1;
        } else if !stripped.is_empty() {
            stats.strip_and_convert += 1;
        } else {
            stats.direct_conversion += 1;
        }

        for annotation in &stripped {
            let key = format!("{:?}", annotation.kind);
            *stats.by_annotation_type.entry(key).or_insert(0) += 1;
        }
    }

    stats
}

fn collect_stripped_annotations(body: &RuleBody) -> Vec<StrippedAnnotation> {
    let mut annotations = Vec::new();
    collect_annotations_recursive(body, &mut annotations);
    annotations
}

fn collect_annotations_recursive(body: &RuleBody, annotations: &mut Vec<StrippedAnnotation>) {
    match body {
        RuleBody::Assignment(a) => {
            let kind = match a.operator {
                AssignmentOp::Set => AnnotationKind::PropertyAssignment,
                AssignmentOp::Append => AnnotationKind::PropertyAppend,
            };
            annotations.push(StrippedAnnotation {
                kind,
                property: Some(a.property.clone()),
                value: None,
                operator: Some(
                    if a.operator == AssignmentOp::Set {
                        "="
                    } else {
                        "+="
                    }
                    .to_string(),
                ),
            });
            if let Some(prefix) = &a.variable_prefix {
                annotations.push(StrippedAnnotation {
                    kind: AnnotationKind::VariablePrefix,
                    property: Some(format!("{}.{}", prefix, a.property)),
                    value: None,
                    operator: None,
                });
            }
            collect_annotations_recursive(&a.value, annotations);
        }

        RuleBody::BooleanFlag(f) => {
            annotations.push(StrippedAnnotation {
                kind: AnnotationKind::BooleanFlag,
                property: Some(f.property.clone()),
                value: Some(f.terminal.clone()),
                operator: Some("?=".to_string()),
            });
            if let Some(prefix) = &f.variable_prefix {
                annotations.push(StrippedAnnotation {
                    kind: AnnotationKind::VariablePrefix,
                    property: Some(format!("{}.{}", prefix, f.property)),
                    value: None,
                    operator: None,
                });
            }
        }

        RuleBody::CrossRef(c) => {
            annotations.push(StrippedAnnotation {
                kind: if c.negated {
                    AnnotationKind::NegatedCrossReference
                } else {
                    AnnotationKind::CrossReference
                },
                property: None,
                value: Some(c.rule_name.clone()),
                operator: None,
            });
        }

        RuleBody::SemanticAction(s) => {
            annotations.push(StrippedAnnotation {
                kind: if s.is_empty {
                    AnnotationKind::EmptySemanticAction
                } else {
                    AnnotationKind::SemanticAction
                },
                property: s.property.clone(),
                value: s.value.clone(),
                operator: None,
            });
        }

        RuleBody::Sequence(items) | RuleBody::Choice(items) => {
            for item in items {
                collect_annotations_recursive(item, annotations);
            }
        }

        RuleBody::Optional(inner)
        | RuleBody::Repeat(inner)
        | RuleBody::Repeat1(inner)
        | RuleBody::Group(inner) => {
            collect_annotations_recursive(inner, annotations);
        }

        RuleBody::Terminal(_) | RuleBody::RuleRef(_) | RuleBody::Empty => {}
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

fn generate_ambiguity_resolutions(rules: &[Rule]) -> AmbiguityResolutions {
    let mut resolutions = AmbiguityResolutions::default();
    let rule_names: std::collections::HashSet<_> = rules.iter().map(|r| r.name.as_str()).collect();

    for rule in rules {
        if rule.name.ends_with("Definition") {
            let usage_name = rule.name.replace("Definition", "Usage");
            if rule_names.contains(usage_name.as_str()) {
                let prefix = rule.name.trim_end_matches("Definition").to_lowercase();
                resolutions.definition_usage_conflicts.push(DefinitionUsageConflict {
                    definition_rule: rule.name.clone(),
                    usage_rule: usage_name,
                    shared_prefix: prefix,
                    mechanism: "GLR conflict array".to_string(),
                    rationale: "Inherent to SysML v2 design; parser cannot distinguish until 'def' keyword seen".to_string(),
                });
            }
        }
    }

    let empty_rules = [
        (
            "Identification",
            "('<' name '>')? name?",
            "choice(seq('<', name, '>', optional(name)), name)",
            "Anonymous elements require optional() at call sites",
        ),
        (
            "FeatureIdentification",
            "Same pattern",
            "Same replacement",
            "Same impact",
        ),
        (
            "RootNamespace",
            "NamespaceBodyElement*",
            "repeat1(namespace_body_element)",
            "Empty files handled by source_file",
        ),
        (
            "MemberPrefix",
            "visibility?",
            "visibility_indicator",
            "Visibility now required where used",
        ),
        (
            "TypePrefix",
            "'abstract'? metadata*",
            "choice with at least one",
            "At least abstract or metadata required",
        ),
        (
            "EmptyFeature",
            "{ }",
            "_empty_marker placeholder",
            "Semantic-only rule",
        ),
        (
            "EmptyMultiplicity",
            "{ }",
            "_empty_marker placeholder",
            "Semantic-only rule",
        ),
        (
            "EmptyUsage",
            "{ }",
            "_empty_marker placeholder",
            "Semantic-only rule",
        ),
        (
            "EmptyActionUsage",
            "{ }",
            "_empty_marker placeholder",
            "Semantic-only rule",
        ),
    ];

    for (rule, original, replacement, impact) in empty_rules {
        if rule_names.contains(rule) {
            resolutions
                .empty_string_replacements
                .push(EmptyStringReplacement {
                    original_rule: rule.to_string(),
                    replacement_rule: to_snake_case(rule),
                    original_body: original.to_string(),
                    replacement_body: replacement.to_string(),
                    semantic_impact: impact.to_string(),
                });
        }
    }

    let additional = [
        (
            "NamespaceBodyElement",
            "PackageBodyElement",
            "context_sensitive_body",
            "Member types overlap across body contexts",
        ),
        (
            "OwnedFeatureChaining",
            "ElementReferenceMember",
            "expression_ambiguity",
            "Feature chains vs element references",
        ),
        (
            "FeatureChain",
            "QualifiedName",
            "expression_ambiguity",
            "Dot-separated names ambiguous",
        ),
    ];

    for (a, b, conflict_type, rationale) in additional {
        if rule_names.contains(a) && rule_names.contains(b) {
            resolutions.additional_conflicts.push(AdditionalConflict {
                rule_a: a.to_string(),
                rule_b: b.to_string(),
                conflict_type: conflict_type.to_string(),
                rationale: rationale.to_string(),
            });
        }
    }

    resolutions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(name: &str) -> Rule {
        Rule {
            name: name.to_string(),
            produces_type: None,
            body: RuleBody::Empty,
            span: 0..0,
            source_line: 0,
        }
    }

    #[test]
    fn to_snake_case_matches_tree_sitter_emitter_for_all_caps_terminals() {
        // mapping.json's tree_sitter_name must agree with what the
        // tree-sitter emitter actually writes for lexical terminals.
        assert_eq!(to_snake_case("NAME"), "name");
        assert_eq!(to_snake_case("BASIC_NAME"), "basic_name");
        assert_eq!(to_snake_case("UNRESTRICTED_NAME"), "unrestricted_name");
        assert_eq!(to_snake_case("WHITE_SPACE"), "white_space");
        assert_eq!(to_snake_case("CONJUGATES"), "conjugates");
        assert_eq!(to_snake_case("SPECIALIZES"), "specializes");
    }

    #[test]
    fn generate_reports_terminal_mapping_without_mangling() {
        let rules = vec![rule("NAME"), rule("BASIC_NAME")];
        let json = generate(&rules).expect("generate should succeed");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["rules"]["NAME"]["tree_sitter_name"], "name");
        assert_eq!(
            value["rules"]["BASIC_NAME"]["tree_sitter_name"],
            "basic_name"
        );
    }

}
