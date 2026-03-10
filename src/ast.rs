use serde::{Deserialize, Serialize};
use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub produces_type: Option<String>,
    pub body: RuleBody,
    pub span: Span,
    pub source_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleBody {
    Sequence(Vec<RuleBody>),
    Choice(Vec<RuleBody>),
    Optional(Box<RuleBody>),
    Repeat(Box<RuleBody>),
    Repeat1(Box<RuleBody>),
    Group(Box<RuleBody>),
    Terminal(String),
    RuleRef(String),
    CrossRef(CrossRef),
    Assignment(Assignment),
    BooleanFlag(BooleanFlag),
    SemanticAction(SemanticAction),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRef {
    pub rule_name: String,
    pub negated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub property: String,
    pub operator: AssignmentOp,
    pub value: Box<RuleBody>,
    pub variable_prefix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentOp {
    Set,
    Append,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanFlag {
    pub property: String,
    pub terminal: String,
    pub variable_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAction {
    pub property: Option<String>,
    pub value: Option<String>,
    pub is_empty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolAlias {
    pub name: String,
    pub alternatives: Vec<Vec<String>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grammar {
    pub rules: Vec<Rule>,
    pub symbol_aliases: Vec<SymbolAlias>,
}

impl RuleBody {
    pub fn is_empty(&self) -> bool {
        matches!(self, RuleBody::Empty)
    }

    pub fn sequence(items: Vec<RuleBody>) -> Self {
        match items.len() {
            0 => RuleBody::Empty,
            1 => items.into_iter().next().unwrap(),
            _ => RuleBody::Sequence(items),
        }
    }

    pub fn choice(items: Vec<RuleBody>) -> Self {
        match items.len() {
            0 => RuleBody::Empty,
            1 => items.into_iter().next().unwrap(),
            _ => RuleBody::Choice(items),
        }
    }
}

impl Rule {
    pub fn needs_manual_review(&self) -> bool {
        has_variable_prefix(&self.body)
    }
}

fn has_variable_prefix(body: &RuleBody) -> bool {
    match body {
        RuleBody::Assignment(a) => a.variable_prefix.is_some(),
        RuleBody::BooleanFlag(f) => f.variable_prefix.is_some(),
        RuleBody::Sequence(items) | RuleBody::Choice(items) => {
            items.iter().any(has_variable_prefix)
        }
        RuleBody::Optional(inner)
        | RuleBody::Repeat(inner)
        | RuleBody::Repeat1(inner)
        | RuleBody::Group(inner) => has_variable_prefix(inner),
        _ => false,
    }
}
