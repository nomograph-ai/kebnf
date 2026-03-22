use crate::ast::*;
use chumsky::prelude::*;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {:?}", self.message, self.span)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(source: &str, _source_name: &str) -> Result<Vec<Rule>, ParseError> {
    let tokens = lexer()
        .parse(source)
        .into_result()
        .map_err(|errs| ParseError {
            message: format!("Lexer errors: {:?}", errs),
            span: 0..source.len(),
        })?;

    let rule_chunks = split_into_rules(&tokens);
    let mut rules = Vec::new();

    for (idx, chunk) in rule_chunks.iter().enumerate() {
        if chunk.is_empty() {
            continue;
        }
        if std::env::var("DEBUG_CHUNKS").is_ok() {
            eprintln!("Chunk {}: {:?}", idx, chunk);
        }
        let rule = rule_parser()
            .parse(&chunk[..])
            .into_result()
            .map_err(|errs| ParseError {
                message: format!("Parser errors in chunk {}: {:?}", idx, errs),
                span: 0..source.len(),
            })?;
        rules.push(rule);
    }

    Ok(rules)
}

fn split_into_rules(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let rule_start_info = check_rule_start(tokens, i);
        if rule_start_info.is_some() && !current_chunk.is_empty() {
            chunks.push(std::mem::take(&mut current_chunk));
        }
        current_chunk.push(tokens[i].clone());

        if let Some(skip) = rule_start_info {
            for j in 1..skip {
                if i + j < tokens.len() {
                    current_chunk.push(tokens[i + j].clone());
                }
            }
            i += skip;
        } else {
            i += 1;
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

fn check_rule_start(tokens: &[Token], pos: usize) -> Option<usize> {
    if pos >= tokens.len() {
        return None;
    }

    match &tokens[pos] {
        Token::Ident(name) => {
            if !name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                return None;
            }

            if pos + 1 < tokens.len() && tokens[pos + 1] == Token::Equals {
                return Some(2);
            }

            if pos + 3 < tokens.len()
                && tokens[pos + 1] == Token::Colon
                && matches!(&tokens[pos + 2], Token::Ident(_))
                && tokens[pos + 3] == Token::Equals
            {
                return Some(4);
            }

            None
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Ident(String),
    Terminal(String),
    Equals,
    PlusEquals,
    QuestionEquals,
    Colon,
    ColonColon,
    Pipe,
    Star,
    Plus,
    Question,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Tilde,
    Dot,
    Semicolon,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{}", s),
            Token::Terminal(s) => write!(f, "'{}'", s),
            Token::Equals => write!(f, "="),
            Token::PlusEquals => write!(f, "+="),
            Token::QuestionEquals => write!(f, "?="),
            Token::Colon => write!(f, ":"),
            Token::ColonColon => write!(f, "::"),
            Token::Pipe => write!(f, "|"),
            Token::Star => write!(f, "*"),
            Token::Plus => write!(f, "+"),
            Token::Question => write!(f, "?"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Tilde => write!(f, "~"),
            Token::Dot => write!(f, "."),
            Token::Semicolon => write!(f, ";"),
        }
    }
}

fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<Token>, extra::Err<Rich<'a, char>>> {
    let ident = text::ident().map(|s: &str| Token::Ident(s.to_string()));

    let terminal = just('\'')
        .ignore_then(none_of("'").repeated().to_slice())
        .then_ignore(just('\''))
        .map(|s: &str| Token::Terminal(s.to_string()));

    let plus_equals = just("+=").to(Token::PlusEquals);
    let question_equals = just("?=").to(Token::QuestionEquals);
    let colon_colon = just("::").to(Token::ColonColon);
    let equals = just('=').to(Token::Equals);
    let colon = just(':').to(Token::Colon);
    let pipe = just('|').to(Token::Pipe);
    let star = just('*').to(Token::Star);
    let plus = just('+').to(Token::Plus);
    let question = just('?').to(Token::Question);
    let lparen = just('(').to(Token::LParen);
    let rparen = just(')').to(Token::RParen);
    let lbracket = just('[').to(Token::LBracket);
    let rbracket = just(']').to(Token::RBracket);
    let lbrace = just('{').to(Token::LBrace);
    let rbrace = just('}').to(Token::RBrace);
    let tilde = just('~').to(Token::Tilde);
    let dot = just('.').to(Token::Dot);
    let semicolon = just(';').to(Token::Semicolon);

    let line_comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .to(());

    let block_comment = just("/*")
        .then(any().and_is(just("*/").not()).repeated())
        .then(just("*/"))
        .to(());

    let comment = line_comment.or(block_comment).padded();

    let token = choice((
        plus_equals,
        question_equals,
        colon_colon,
        equals,
        colon,
        pipe,
        star,
        plus,
        question,
        lparen,
        rparen,
        lbracket,
        rbracket,
        lbrace,
        rbrace,
        tilde,
        dot,
        semicolon,
        terminal,
        ident,
    ));

    token
        .padded_by(comment.repeated())
        .padded()
        .repeated()
        .collect()
}

fn rule_parser<'a>() -> impl Parser<'a, &'a [Token], Rule, extra::Err<Rich<'a, Token>>> {
    let ident = select! { Token::Ident(s) => s.clone() };

    let type_annotation = just(Token::Colon).ignore_then(ident).or_not();

    ident
        .then(type_annotation)
        .then_ignore(just(Token::Equals))
        .then(rule_body_parser())
        .map(|((name, produces_type), body)| Rule {
            name,
            produces_type,
            body,
            span: 0..0,
            source_line: 0,
        })
}

fn rule_body_parser<'a>() -> impl Parser<'a, &'a [Token], RuleBody, extra::Err<Rich<'a, Token>>> {
    recursive(|body| {
        let ident = select! { Token::Ident(s) => s.clone() };
        let terminal = select! { Token::Terminal(s) => s.clone() };

        let terminal_rule = terminal.map(RuleBody::Terminal);

        let rule_ref = ident
            .then(
                choice((
                    just(Token::Colon),
                    just(Token::Equals),
                    just(Token::PlusEquals),
                    just(Token::QuestionEquals),
                ))
                .not()
                .rewind(),
            )
            .map(|(name, _)| RuleBody::RuleRef(name));

        let cross_ref = just(Token::Tilde)
            .or_not()
            .then_ignore(just(Token::LBracket))
            .then(ident)
            .then_ignore(just(Token::RBracket))
            .map(|(tilde, name)| {
                RuleBody::CrossRef(CrossRef {
                    rule_name: name,
                    negated: tilde.is_some(),
                })
            });

        let variable_prefix = ident.then_ignore(just(Token::Dot)).or_not();

        let boolean_flag = variable_prefix
            .clone()
            .then(ident)
            .then_ignore(just(Token::QuestionEquals))
            .then(terminal)
            .map(|((prefix, property), term)| {
                RuleBody::BooleanFlag(BooleanFlag {
                    property,
                    terminal: term,
                    variable_prefix: prefix,
                })
            });

        let assignment_op = choice((
            just(Token::PlusEquals).to(AssignmentOp::Append),
            just(Token::Equals).to(AssignmentOp::Set),
        ));

        let assignment_value = choice((
            cross_ref.clone(),
            just(Token::LParen)
                .ignore_then(body.clone())
                .then_ignore(just(Token::RParen))
                .map(|b| RuleBody::Group(Box::new(b))),
            rule_ref.clone(),
            terminal_rule,
        ))
        .boxed();

        let assignment = variable_prefix
            .then(ident)
            .then(assignment_op.clone())
            .then(assignment_value)
            .map(|(((prefix, property), operator), value)| {
                RuleBody::Assignment(Assignment {
                    property,
                    operator,
                    value: Box::new(value),
                    variable_prefix: prefix,
                })
            });

        let semantic_action_op = choice((
            just(Token::PlusEquals).to("+="),
            just(Token::Equals).to("="),
        ));

        let dotted_path = ident
            .then(
                just(Token::Dot)
                    .ignore_then(ident)
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(|(first, rest)| {
                let mut path = first;
                for part in rest {
                    path.push('.');
                    path.push_str(&part);
                }
                path
            });

        let semantic_action = just(Token::LBrace)
            .ignore_then(
                dotted_path
                    .clone()
                    .then(semantic_action_op)
                    .then(dotted_path.clone().or(terminal))
                    .map(|((prop, op), val)| SemanticAction {
                        property: Some(prop),
                        value: Some(format!("{}{}", op, val)),
                        is_empty: false,
                    })
                    .or(empty().map(|_| SemanticAction {
                        property: None,
                        value: None,
                        is_empty: true,
                    })),
            )
            .then_ignore(just(Token::RBrace))
            .map(RuleBody::SemanticAction);

        let group = just(Token::LParen)
            .ignore_then(body.clone())
            .then_ignore(just(Token::RParen))
            .map(|b| RuleBody::Group(Box::new(b)));

        let atom = choice((
            boolean_flag,
            assignment,
            semantic_action,
            cross_ref,
            terminal_rule,
            group,
            rule_ref,
        ));

        let postfix = atom
            .then(
                choice((
                    just(Token::Star).to('*'),
                    just(Token::Plus).to('+'),
                    just(Token::Question).to('?'),
                ))
                .or_not(),
            )
            .map(|(body, op)| match op {
                Some('*') => RuleBody::Repeat(Box::new(body)),
                Some('+') => RuleBody::Repeat1(Box::new(body)),
                Some('?') => RuleBody::Optional(Box::new(body)),
                _ => body,
            });

        let sequence = postfix
            .clone()
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .map(RuleBody::sequence);

        sequence
            .clone()
            .separated_by(just(Token::Pipe))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(RuleBody::choice)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rule() {
        let source = "Foo = 'bar'";
        let rules = parse(source, "test").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Foo");
    }

    #[test]
    fn test_rule_with_type() {
        let source = "Foo : Bar = 'baz'";
        let rules = parse(source, "test").unwrap();
        assert_eq!(rules[0].produces_type, Some("Bar".to_string()));
    }

    #[test]
    fn test_sequence() {
        let source = "Foo = 'a' 'b' 'c'";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::Sequence(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected sequence"),
        }
    }

    #[test]
    fn test_choice() {
        let source = "Foo = 'a' | 'b' | 'c'";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::Choice(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected choice"),
        }
    }

    #[test]
    fn test_optional() {
        let source = "Foo = 'a'?";
        let rules = parse(source, "test").unwrap();
        matches!(&rules[0].body, RuleBody::Optional(_));
    }

    #[test]
    fn test_repeat() {
        let source = "Foo = 'a'*";
        let rules = parse(source, "test").unwrap();
        matches!(&rules[0].body, RuleBody::Repeat(_));
    }

    #[test]
    fn test_repeat1() {
        let source = "Foo = 'a'+";
        let rules = parse(source, "test").unwrap();
        matches!(&rules[0].body, RuleBody::Repeat1(_));
    }

    #[test]
    fn test_boolean_flag() {
        let source = "Foo = isAbstract ?= 'abstract'";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::BooleanFlag(f) => {
                assert_eq!(f.property, "isAbstract");
                assert_eq!(f.terminal, "abstract");
            }
            _ => panic!("Expected boolean flag"),
        }
    }

    #[test]
    fn test_cross_ref() {
        let source = "Foo = [QualifiedName]";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::CrossRef(c) => {
                assert_eq!(c.rule_name, "QualifiedName");
                assert!(!c.negated);
            }
            _ => panic!("Expected cross ref"),
        }
    }

    #[test]
    fn test_negated_cross_ref() {
        let source = "Foo = ~[QualifiedName]";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::CrossRef(c) => {
                assert!(c.negated);
            }
            _ => panic!("Expected cross ref"),
        }
    }

    #[test]
    fn test_assignment() {
        let source = "Foo = name = Identifier";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::Assignment(a) => {
                assert_eq!(a.property, "name");
                assert_eq!(a.operator, AssignmentOp::Set);
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_append_assignment() {
        let source = "Foo = items += Element";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::Assignment(a) => {
                assert_eq!(a.operator, AssignmentOp::Append);
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_variable_prefix() {
        let source = "Foo = s.items += Element";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::Assignment(a) => {
                assert_eq!(a.variable_prefix, Some("s".to_string()));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_semantic_action() {
        let source = "Foo = { isPortion = true }";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::SemanticAction(s) => {
                assert_eq!(s.property, Some("isPortion".to_string()));
                assert_eq!(s.value, Some("=true".to_string()));
            }
            _ => panic!("Expected semantic action"),
        }
    }

    #[test]
    fn test_empty_semantic_action() {
        let source = "Foo = { }";
        let rules = parse(source, "test").unwrap();
        match &rules[0].body {
            RuleBody::SemanticAction(s) => {
                assert!(s.is_empty);
            }
            _ => panic!("Expected semantic action"),
        }
    }
}
