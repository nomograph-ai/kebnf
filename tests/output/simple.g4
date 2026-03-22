// Sysml — ANTLR4 combined grammar
//
// Generated from OMG KeBNF specifications by kebnf.
// Source: https://gitlab.com/nomograph/kebnf
//
// This grammar was mechanically converted from the KerML and SysML v2
// KeBNF specifications. Metamodel annotations (type bindings, property
// assignments, cross-references) have been stripped. A mapping file
// preserving semantic traceability can be generated with --mapping.
//
// Known limitations:
//   - Expression and import rules were restructured to eliminate mutual
//     left recursion (ANTLR4 requires direct left recursion only).
//   - Rules named 'import' in KeBNF are emitted as 'import_' because
//     'import' is an ANTLR4 reserved word.
//   - Some KeBNF rules are purely semantic (e.g., EmptyFeature) and
//     have been omitted. References to them are dropped.
//   - Symbol alias tokens (SPECIALIZES, SUBSETS, etc.) have overlapping
//     alternatives that produce ANTLR4 warnings. This is inherent to
//     the SysML v2 design (both symbolic and keyword forms are valid).
//

grammar Sysml;

// ─── Expressions ─────────────────────────────────────────
// Merged from mutually recursive KeBNF rules into directly
// left-recursive forms that ANTLR4's LL(*) parser accepts.
// Original rules: OwnedExpression, BinaryOperatorExpression,
// ConditionalBinaryOperatorExpression, ClassificationExpression.

ownedExpression
    : conditionalExpression
    | ownedExpression conditionalBinaryOperator argumentExpressionMember
    | ownedExpression binaryOperator ownedExpression
    | unaryOperatorExpression
    | ownedExpression classificationTestOperator typeReferenceMember
    | ownedExpression castOperator typeResultMember
    | classificationTestOperator typeReferenceMember
    | castOperator typeResultMember
    | metaclassificationExpression
    | extentExpression
    | primaryExpression
    ;

// Merged from PrimaryExpression, BracketExpression, IndexExpression,
// SelectExpression, CollectExpression, FunctionOperationExpression,
// FeatureChainExpression.

primaryExpression
    : literalExpression
    | invocationExpression
    | bodyExpression
    | metadataAccessExpression
    | nullExpression
    | '(' ownedExpression ')'
    | primaryExpression '#' '(' ownedExpression ')'
    | primaryExpression '[' ownedExpression ']'
    | primaryExpression '.' ownedExpression
    | primaryExpression '.?' ownedExpression
    | primaryExpression '.' bodyExpression
    | primaryExpression '.?' bodyExpression
    | primaryExpression '.' NAME
    ;

// ─── Imports ─────────────────────────────────────────────
// Merged from ImportDeclaration, NamespaceImport, FilterPackage
// to break mutual left recursion.

importDeclaration
    : membershipImport
    | qualifiedName '::' '*' ('::' '**'?)?
    | importDeclaration filterPackageMember+
    ;

namespaceImport
    : qualifiedName '::' '*' ('::' '**'?)?
    | importDeclaration filterPackageMember+
    ;

filterPackage
    : importDeclaration filterPackageMember+
    ;

// ─── Parser rules ────────────────────────────────────────

package
    : 'package' identification? packageBody
    ;

identification
    : (('<' IDENTIFIER '>'))? IDENTIFIER?
    ; // : Element

packageBody
    : ';'
    | '{' packageBodyElement* '}'
    ;

packageBodyElement
    : partDefinition
    | partUsage
    ;

partDefinition
    : 'abstract'? 'part' 'def' identification?
    ;

partUsage
    : 'part' identification? typePart
    ;

typePart
    : ':' IDENTIFIER
    ;

typeName
    : IDENTIFIER
    ;

name
    : IDENTIFIER
    ;

// ─── Built-in lexer rules ───────────────────────────────
// These replace the KeBNF lexical grammar with ANTLR4-native
// patterns for whitespace, comments, names, and literals.

WS
    : [ \t\r\n]+ -> skip
    ;

SINGLE_LINE_COMMENT
    : '//' ~[\r\n]* -> channel(HIDDEN)
    ;

MULTI_LINE_COMMENT
    : '/*' .*? '*/' -> channel(HIDDEN)
    ;

NAME
    : BASIC_NAME | UNRESTRICTED_NAME
    ;

fragment BASIC_NAME
    : [a-zA-Z_] [a-zA-Z0-9_]*
    ;

fragment UNRESTRICTED_NAME
    : '\'' (~['\\] | '\\' .)* '\''
    ;

STRING_VALUE
    : '"' (~["\\] | '\\' .)* '"'
    ;

DECIMAL_VALUE
    : [0-9]+
    ;

REAL_VALUE
    : [0-9]+ '.' [0-9]* ([eE] [+-]? [0-9]+)?
    | '.' [0-9]+ ([eE] [+-]? [0-9]+)?
    ;

EXPONENTIAL_VALUE
    : [0-9]+ [eE] [+-]? [0-9]+
    ;

// ─── Stub rules ─────────────────────────────────────────
// These rules are referenced in the KeBNF source but not defined.
// They may be defined in a different spec file or in the metamodel.

IDENTIFIER
    : NAME // stub: IDENTIFIER not defined in KeBNF source
    ;

