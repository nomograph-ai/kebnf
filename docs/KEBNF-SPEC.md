# KEBNF Specification Reference

Complete documentation of the KEBNF (KerML Extended BNF) syntax as used in OMG SysML v2 and KerML specifications.

## Overview

KEBNF extends standard EBNF with metamodel-aware annotations that bind grammar productions to the SysML/KerML abstract syntax model. These annotations are meaningful to tools that build ASTs conforming to the metamodel but have no direct equivalent in pure parsing frameworks like tree-sitter.

## Lexical Elements

### Comments

```ebnf
// Single-line comment (to end of line)

/* Multi-line
   comment */
```

### Terminals

```ebnf
'keyword'           // Keyword literal
"string"            // String literal (rarely used, prefer single quotes)
```

### Names

```ebnf
RuleName            // PascalCase rule reference
SYMBOL_NAME         // UPPER_CASE typically for token aliases
```

## Rule Syntax

### Basic Rule Definition

```ebnf
RuleName = Body
```

### Rule with Type Annotation

```ebnf
RuleName : MetamodelType = Body
```

The `: MetamodelType` declares that this rule produces an instance of the specified metamodel class. This is semantic information for AST construction, not parsing.

**Example:**
```ebnf
PartDeclaration : PartDefinition =
    'part' 'def' Identification
```

## Body Elements

### Sequences

```ebnf
A B C               // Match A, then B, then C
```

### Choices (Alternatives)

```ebnf
A | B | C           // Match A or B or C
```

### Grouping

```ebnf
( A B )             // Group A B as a unit
```

### Optionals

```ebnf
A?                  // Zero or one A
( A B )?            // Zero or one (A followed by B)
```

### Repetition

```ebnf
A*                  // Zero or more A
A+                  // One or more A
( A B )*            // Zero or more (A followed by B)
```

## Property Assignments

KEBNF uses property assignments to bind parsed elements to metamodel properties.

### Simple Assignment

```ebnf
propertyName = Element
```

Assigns the parsed `Element` to the property `propertyName`.

**Example:**
```ebnf
declaredName = NAME
```

### Collection Append

```ebnf
propertyName += Element
```

Appends the parsed `Element` to the collection property `propertyName`.

**Example:**
```ebnf
ownedRelationship += PrefixMetadataMember
```

### Boolean Flag

```ebnf
propertyName ?= 'terminal'
```

Sets the boolean property `propertyName` to `true` if the terminal is present.

**Example:**
```ebnf
isAbstract ?= 'abstract'
isOrdered ?= 'ordered'
```

## Cross-References

### Basic Cross-Reference

```ebnf
propertyName = [QualifiedName]
```

The `[QualifiedName]` notation indicates that:
1. Parse text as a `QualifiedName`
2. Resolve it to find the referenced element
3. Assign the resolved element (not the name) to the property

**Example:**
```ebnf
memberElement = [QualifiedName]
superclassifier = [QualifiedName]
```

### Negated Cross-Reference

```ebnf
propertyName = ~[QualifiedName]
```

Special resolution semantics: prepend `~` to the final segment of the qualified name before resolution.

**Example:**
```ebnf
ConjugatedPortTyping : ConjugatedPortTyping =
    '~' originalPortDefinition = ~[QualifiedName]
```

Resolution algorithm for `~A::B::C`:
1. Extract final segment: `C`
2. Prepend `~`: `~C`
3. Resolve as: `A::B::C::'~C'`

## Semantic Actions

### Property Setting Action

```ebnf
{ propertyName = value }
```

Unconditionally sets a property to a value. This is purely semantic (affects AST construction, not parsing).

**Examples:**
```ebnf
{ isPortion = true }
{ isVariable = true }
{ isUnique = false }
```

### Combined with Parsing

```ebnf
isConstant ?= 'const' { isVariable = true }
```

If `'const'` is parsed, set both `isConstant = true` AND `isVariable = true`.

### Empty Semantic Block

```ebnf
RuleName : Type = { }
```

Creates an empty instance of the type. No parsing occurs.

**Example:**
```ebnf
EmptyFeature : Feature = { }
EmptyMultiplicity : Multiplicity = { }
```

## Variable Prefixes

### Scoped Property Assignment

```ebnf
s.propertyName += Element
e.propertyName += Element
```

Variable prefixes (`s.`, `e.`, etc.) indicate which context/scope the property belongs to. These appear when a rule can be used in multiple contexts that need different property bindings.

**Example:**
```ebnf
SuccessionDeclaration : Succession =
    FeatureDeclaration
    ( 'first' ownedRelationship += ConnectorEndMember
      'then'  ownedRelationship += ConnectorEndMember )?
  | ( s.isSufficient ?= 'all' )?
    ( 'first'? ownedRelationship += ConnectorEndMember
      'then'   ownedRelationship += ConnectorEndMember )?
```

The `s.` prefix indicates "on the Succession being built" vs the general context.

## Symbol Aliases

### Token Alternatives

```ebnf
SYMBOL_NAME = 'symbol' | 'keyword' 'sequence'
```

Defines alternative syntactic forms for the same semantic concept.

**Examples:**
```ebnf
TYPED_BY    = ':'   | 'typed' 'by'
SPECIALIZES = ':>'  | 'specializes'
SUBSETS     = ':>'  | 'subsets'
REFERENCES  = '::>' | 'references'
CROSSES     = '=>'  | 'crosses'
REDEFINES   = ':>>' | 'redefines'
CONJUGATES  = '~'   | 'conjugates'
DEFINED_BY  = ':'   | 'defined' 'by'
```

Note: `SPECIALIZES` and `SUBSETS` share the same symbol `:>` - context determines meaning.

## Inline Notes

```ebnf
// (See Note 1)
```

Reference to explanatory notes in the specification text. These provide semantic clarification not expressible in the grammar.

## Complete Pattern Examples

### Simple Definition

```ebnf
Package =
    ( ownedRelationship += PrefixMetadataMember )*
    PackageDeclaration PackageBody
```

### Definition with Type and Properties

```ebnf
Identification : Element =
    ( '<' declaredShortName = NAME '>' )?
    ( declaredName = NAME )?
```

### Complex with Alternatives and Actions

```ebnf
FeatureValue =
    ( '='
    | isInitial ?= ':='
    | isDefault ?= 'default' ( '=' | isInitial ?= ':=' )?
    )
    ownedRelatedElement += OwnedExpression
```

### Rule with Multiple Property Contexts

```ebnf
PositionalArgumentList : Feature =
    e.ownedRelationship += ArgumentMember
    ( ',' e.ownedRelationship += ArgumentMember )*
```

### Empty Production

```ebnf
EmptyFeature : Feature =
    { }
```

## Tree-sitter Mapping Summary

| KEBNF Construct | Tree-sitter Equivalent | Notes |
|-----------------|----------------------|-------|
| `A B C` | `seq(A, B, C)` | Direct |
| `A \| B` | `choice(A, B)` | Direct |
| `A?` | `optional(A)` | Direct |
| `A*` | `repeat(A)` | Direct |
| `A+` | `repeat1(A)` | Direct |
| `'keyword'` | `'keyword'` | Direct |
| `: Type` | (strip) | Document in mapping |
| `prop = X` | `X` | Strip assignment |
| `prop += X` | `X` | Strip assignment |
| `prop ?= 'x'` | `optional('x')` | Convert to optional |
| `[Name]` | `$.name` | Parse as rule |
| `~[Name]` | `$.name` | Parse as rule, document resolution |
| `{ prop = val }` | (strip or empty seq) | Best-effort, document |
| `{ }` | `seq()` or blank | Empty production |
| `s.prop` | `prop` | Strip prefix, flag for review |
| `SYMBOL = ...` | Inline expansion | Expand at use sites |

## Source Files

The authoritative KEBNF sources are:

- **KerML**: `github.com/Systems-Modeling/SysML-v2-Release/tree/master/bnf/KerML-textual-bnf.kebnf`
- **SysML**: `github.com/Systems-Modeling/SysML-v2-Release/tree/master/bnf/SysML-textual-bnf.kebnf`

Note: Files contain manual corrections by HP de Koning as noted in their headers.
