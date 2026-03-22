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

identification
    : (('<' NAME '>'))? (NAME)?
    ; // : Element

relationshipBody
    : ';'
    | '{' relationshipOwnedElement* '}'
    ; // : Relationship

relationshipOwnedElement
    : ownedRelatedElement
    | annotatingElement
    ; // : Relationship

ownedRelatedElement
    : nonFeatureElement
    | featureElement
    ; // : Element

dependency
    : (prefixMetadataAnnotation)* 'dependency' ((identification? 'from'))? qualifiedName ((',' qualifiedName))* 'to' qualifiedName ((',' qualifiedName))* relationshipBody
    ;

annotation
    : qualifiedName
    ;

ownedAnnotation
    : annotatingElement
    ; // : Annotation

annotatingElement
    : comment
    | documentation
    | textualRepresentation
    | metadataFeature
    ;

comment
    : (('comment' identification (('about' annotation ((',' annotation))*))?))? (('locale' STRING_VALUE))? REGULAR_COMMENT
    ;

documentation
    : 'doc' identification (('locale' STRING_VALUE))? REGULAR_COMMENT
    ;

textualRepresentation
    : (('rep' identification))? 'language' STRING_VALUE REGULAR_COMMENT
    ;

rootNamespace
    : namespaceBodyElement*
    ; // : Namespace

namespace
    : (prefixMetadataMember)* namespaceDeclaration namespaceBody
    ;

namespaceDeclaration
    : 'namespace' identification
    ; // : Namespace

namespaceBody
    : ';'
    | '{' namespaceBodyElement* '}'
    ; // : Namespace

namespaceBodyElement
    : namespaceMember
    | aliasMember
    | import_
    ; // : Namespace

memberPrefix
    : (visibilityIndicator)?
    ; // : Membership

visibilityIndicator
    : 'public'
    | 'private'
    | 'protected'
    ; // : VisibilityKind

namespaceMember
    : nonFeatureMember
    | namespaceFeatureMember
    ; // : OwningMembership

nonFeatureMember
    : memberPrefix memberElement
    ; // : OwningMembership

namespaceFeatureMember
    : memberPrefix featureElement
    ; // : OwningMembership

aliasMember
    : memberPrefix 'alias' (('<' NAME '>'))? (NAME)? 'for' qualifiedName relationshipBody
    ; // : Membership

qualifiedName
    : (('$' '::'))? ((NAME '::'))* NAME
    ;

import_
    : visibilityIndicator 'import' ('all'?)? importDeclaration relationshipBody
    ; // renamed from 'import' (ANTLR4 reserved word)

membershipImport
    : qualifiedName (('::' '**'?))?
    ;

filterPackageMember
    : '[' ownedExpression ']'
    ; // : ElementFilterMembership

memberElement
    : annotatingElement
    | nonFeatureElement
    ; // : Element

nonFeatureElement
    : dependency
    | namespace
    | type
    | classifier
    | dataType
    | class
    | structure
    | metaclass
    | association
    | associationStructure
    | interaction
    | behavior
    | function
    | predicate
    | multiplicity
    | package
    | libraryPackage
    | specialization
    | conjugation
    | subclassification
    | disjoining
    | featureInverting
    | featureTyping
    | subsetting
    | redefinition
    | typeFeaturing
    ; // : Element

featureElement
    : feature
    | step
    | expression
    | booleanExpression
    | invariant
    | connector
    | bindingConnector
    | succession
    | flow
    | successionFlow
    ; // : Feature

type
    : typePrefix 'type' typeDeclaration typeBody
    ;

typePrefix
    : ('abstract'?)? (prefixMetadataMember)*
    ; // : Type

typeDeclaration
    : ('all'?)? identification (multiplicityRange)? ((specializationPart
    | conjugationPart))+ typeRelationshipPart*
    ; // : Type

specializationPart
    : SPECIALIZES generalType ((',' generalType))*
    ; // : Type

conjugationPart
    : CONJUGATES ownedConjugation
    ; // : Type

typeRelationshipPart
    : disjoiningPart
    | unioningPart
    | intersectingPart
    | differencingPart
    ; // : Type

disjoiningPart
    : 'disjoint' 'from' ownedDisjoining ((',' ownedDisjoining))*
    ; // : Type

unioningPart
    : 'unions' unioning ((',' unioning))*
    ; // : Type

intersectingPart
    : 'intersects' intersecting ((',' intersecting))*
    ; // : Type

differencingPart
    : 'differences' differencing ((',' differencing))*
    ; // : Type

typeBody
    : ';'
    | '{' typeBodyElement* '}'
    ; // : Type

typeBodyElement
    : nonFeatureMember
    | featureMember
    | aliasMember
    | import_
    ; // : Type

specialization
    : (('specialization' identification))? 'subtype' specificType SPECIALIZES generalType relationshipBody
    ;

ownedSpecialization
    : generalType
    ; // : Specialization

specificType
    : qualifiedName
    | featureChain
    ; // : Specialization

generalType
    : qualifiedName
    | featureChain
    ; // : Specialization

conjugation
    : (('conjugation' identification))? 'conjugate' (qualifiedName
    | featureChain) CONJUGATES (qualifiedName
    | featureChain) relationshipBody
    ;

ownedConjugation
    : qualifiedName
    | featureChain
    ; // : Conjugation

disjoining
    : (('disjoining' identification))? 'disjoint' (qualifiedName
    | featureChain) 'from' (qualifiedName
    | featureChain) relationshipBody
    ;

ownedDisjoining
    : qualifiedName
    | featureChain
    ; // : Disjoining

unioning
    : qualifiedName
    | featureChain
    ;

intersecting
    : qualifiedName
    | featureChain
    ;

differencing
    : qualifiedName
    | featureChain
    ;

featureMember
    : typeFeatureMember
    | ownedFeatureMember
    ; // : OwningMembership

typeFeatureMember
    : memberPrefix 'member' featureElement
    ; // : OwningMembership

ownedFeatureMember
    : memberPrefix featureElement
    ; // : FeatureMembership

classifier
    : typePrefix 'classifier' classifierDeclaration typeBody
    ;

classifierDeclaration
    : ('all'?)? identification (multiplicityRange)? ((superclassingPart
    | conjugationPart))? typeRelationshipPart*
    ; // : Classifier

superclassingPart
    : SPECIALIZES ownedSubclassification ((',' ownedSubclassification))*
    ; // : Classifier

subclassification
    : (('specialization' identification))? 'subclassifier' qualifiedName SPECIALIZES qualifiedName relationshipBody
    ;

ownedSubclassification
    : qualifiedName
    ; // : Subclassification

feature
    : (featurePrefix ('feature'
    | prefixMetadataMember) featureDeclaration?
    | (endFeaturePrefix
    | basicFeaturePrefix) featureDeclaration) featureValue? typeBody
    ;

endFeaturePrefix
    : ('const'?)? 'end'?
    ; // : Feature

basicFeaturePrefix
    : (featureDirection)? ('derived'?)? ('abstract'?)? (('composite'?
    | 'portion'?))? (('var'?
    | 'const'?))?
    ; // : Feature

featurePrefix
    : (endFeaturePrefix (ownedCrossFeature)?
    | basicFeaturePrefix) (prefixMetadataMember)*
    ;

ownedCrossFeatureMember
    : ownedCrossFeature
    ; // : OwningMembership

ownedCrossFeature
    : basicFeaturePrefix featureDeclaration
    ; // : Feature

featureDirection
    : 'in'
    | 'out'
    | 'inout'
    ; // : FeatureDirectionKind

featureDeclaration
    : ('all'?)? (featureIdentification ((featureSpecializationPart
    | conjugationPart))?
    | featureSpecializationPart
    | conjugationPart) featureRelationshipPart*
    ; // : Feature

featureIdentification
    : '<' NAME '>' (NAME)?
    | NAME
    ; // : Feature

featureRelationshipPart
    : typeRelationshipPart
    | chainingPart
    | invertingPart
    | typeFeaturingPart
    ; // : Feature

chainingPart
    : 'chains' (ownedFeatureChaining
    | featureChain)
    ; // : Feature

invertingPart
    : 'inverse' 'of' ownedFeatureInverting
    ; // : Feature

typeFeaturingPart
    : 'featured' 'by' ownedTypeFeaturing ((',' ownedTypeFeaturing))*
    ; // : Feature

featureSpecializationPart
    : featureSpecialization+ multiplicityPart? featureSpecialization*
    | multiplicityPart featureSpecialization*
    ; // : Feature

multiplicityPart
    : multiplicityRange
    | (multiplicityRange)? ('ordered'? ('nonunique')?
    | 'nonunique' ('ordered'?)?)
    ; // : Feature

featureSpecialization
    : typings
    | subsettings
    | references
    | crosses
    | redefinitions
    ; // : Feature

typings
    : typedBy ((',' generalType))*
    ; // : Feature

typedBy
    : TYPED_BY generalType
    ; // : Feature

subsettings
    : subsets ((',' generalType))*
    ; // : Feature

subsets
    : SUBSETS generalType
    ; // : Feature

references
    : REFERENCES generalType
    ; // : Feature

crosses
    : CROSSES generalType
    ; // : Feature

redefinitions
    : redefines ((',' generalType))*
    ; // : Feature

redefines
    : REDEFINES generalType
    ; // : Feature

featureTyping
    : (('specialization' identification))? 'typing' qualifiedName TYPED_BY generalType relationshipBody
    ;

ownedFeatureTyping
    : generalType
    ; // : FeatureTyping

subsetting
    : (('specialization' identification))? 'subset' specificType SUBSETS generalType relationshipBody
    ;

ownedSubsetting
    : generalType
    ; // : Subsetting

ownedReferenceSubsetting
    : generalType
    ; // : ReferenceSubsetting

ownedCrossSubsetting
    : generalType
    ; // : CrossSubsetting

redefinition
    : (('specialization' identification))? 'redefinition' specificType REDEFINES generalType relationshipBody
    ;

ownedRedefinition
    : generalType
    ; // : Redefinition

ownedFeatureChain
    : featureChain
    ; // : Feature

featureChain
    : ownedFeatureChaining (('.' ownedFeatureChaining))+
    ; // : Feature

ownedFeatureChaining
    : qualifiedName
    ; // : FeatureChaining

featureInverting
    : (('inverting' identification?))? 'inverse' (qualifiedName
    | featureChain) 'of' (qualifiedName
    | featureChain) relationshipBody
    ;

ownedFeatureInverting
    : qualifiedName
    | featureChain
    ; // : FeatureInverting

typeFeaturing
    : 'featuring' ((identification 'of'))? qualifiedName 'by' qualifiedName relationshipBody
    ;

ownedTypeFeaturing
    : qualifiedName
    ; // : TypeFeaturing

dataType
    : typePrefix 'datatype' classifierDeclaration typeBody
    ;

class
    : typePrefix 'class' classifierDeclaration typeBody
    ;

structure
    : typePrefix 'struct' classifierDeclaration typeBody
    ;

association
    : typePrefix 'assoc' classifierDeclaration typeBody
    ;

associationStructure
    : typePrefix 'assoc' 'struct' classifierDeclaration typeBody
    ;

connector
    : featurePrefix 'connector' (featureDeclaration? featureValue?
    | connectorDeclaration) typeBody
    ;

connectorDeclaration
    : binaryConnectorDeclaration
    | naryConnectorDeclaration
    ; // : Connector

binaryConnectorDeclaration
    : ((featureDeclaration? 'from'
    | 'all'? 'from'?))? connectorEnd 'to' connectorEnd
    ; // : Connector

naryConnectorDeclaration
    : featureDeclaration? '(' connectorEnd ',' connectorEnd ((',' connectorEnd))* ')'
    ; // : Connector

connectorEndMember
    : connectorEnd
    ; // : EndFeatureMembership

connectorEnd
    : (multiplicityRange)? ((NAME REFERENCES))? generalType
    ; // : Feature

ownedCrossMultiplicityMember
    : multiplicityRange
    ; // : OwningMembership

ownedCrossMultiplicity
    : multiplicityRange
    ; // : Feature

bindingConnector
    : featurePrefix 'binding' bindingConnectorDeclaration typeBody
    ;

bindingConnectorDeclaration
    : featureDeclaration (('of' connectorEnd '=' connectorEnd))?
    | ('all'?)? (('of'? connectorEnd '=' connectorEnd))?
    ; // : BindingConnector

succession
    : featurePrefix 'succession' successionDeclaration typeBody
    ;

successionDeclaration
    : featureDeclaration (('first' connectorEnd 'then' connectorEnd))?
    | ('all'?)? (('first'? connectorEnd 'then' connectorEnd))?
    ; // : Succession

behavior
    : typePrefix 'behavior' classifierDeclaration typeBody
    ;

step
    : featurePrefix 'step' featureDeclaration featureValue? typeBody
    ;

function
    : typePrefix 'function' classifierDeclaration functionBody
    ;

functionBody
    : ';'
    | '{' functionBodyPart '}'
    ; // : Type

functionBodyPart
    : ((typeBodyElement
    | returnFeatureMember))* (resultExpressionMember)?
    ; // : Type

returnFeatureMember
    : memberPrefix 'return' featureElement
    ; // : ReturnParameterMembership

resultExpressionMember
    : memberPrefix ownedExpression
    ; // : ResultExpressionMembership

expression
    : featurePrefix 'expr' featureDeclaration featureValue? functionBody
    ;

predicate
    : typePrefix 'predicate' classifierDeclaration functionBody
    ;

booleanExpression
    : featurePrefix 'bool' featureDeclaration featureValue? functionBody
    ;

invariant
    : featurePrefix 'inv' (('true'
    | 'false'?))? featureDeclaration featureValue? functionBody
    ;

ownedExpressionReferenceMember
    : ownedExpression
    ; // : FeatureMembership

ownedExpressionReference
    : ownedExpression
    ; // : FeatureReferenceExpression

ownedExpressionMember
    : ownedExpression
    ; // : FeatureMembership

conditionalExpression
    : 'if' ownedExpression '?' ownedExpression 'else' ownedExpression
    ; // : OperatorExpression

conditionalBinaryOperator
    : '??'
    | 'or'
    | 'and'
    | 'implies'
    ;

binaryOperator
    : '|'
    | '&'
    | 'xor'
    | '..'
    | '=='
    | '!='
    | '==='
    | '!=='
    | '<'
    | '>'
    | '<='
    | '>='
    | '+'
    | '-'
    | '*'
    | '/'
    | '%'
    | '^'
    | '**'
    ;

unaryOperatorExpression
    : unaryOperator ownedExpression
    ; // : OperatorExpression

unaryOperator
    : '+'
    | '-'
    | '~'
    | 'not'
    ;

classificationTestOperator
    : 'istype'
    | 'hastype'
    | '@'
    ;

castOperator
    : 'as'
    ;

metaclassificationExpression
    : elementReferenceMember (classificationTestOperator referenceTyping
    | metaCastOperator referenceTyping)
    ; // : OperatorExpression

argumentMember
    : ownedExpression
    ; // : ParameterMembership

argument
    : ownedExpression
    ; // : Feature

argumentValue
    : ownedExpression
    ; // : FeatureValue

argumentExpressionMember
    : ownedExpression
    ; // : FeatureMembership

argumentExpression
    : ownedExpression
    ; // : Feature

argumentExpressionValue
    : ownedExpression
    ; // : FeatureValue

metadataArgumentMember
    : elementReferenceMember
    ; // : ParameterMembership

metadataArgument
    : elementReferenceMember
    ; // : Feature

metadataValue
    : elementReferenceMember
    ; // : FeatureValue

metadataReference
    : elementReferenceMember
    ; // : MetadataAccessExpression

metaclassificationTestOperator
    : '@@'
    ;

metaCastOperator
    : 'meta'
    ;

extentExpression
    : 'all' referenceTyping
    ; // : OperatorExpression

typeReferenceMember
    : referenceTyping
    ; // : ParameterMembership

typeResultMember
    : referenceTyping
    ; // : ResultParameterMembership

typeReference
    : referenceTyping
    ; // : Feature

referenceTyping
    : qualifiedName
    ; // : FeatureTyping

primaryArgumentValue
    : primaryExpression
    ; // : FeatureValue

primaryArgument
    : primaryExpression
    ; // : Feature

primaryArgumentMember
    : primaryExpression
    ; // : ParameterMembership

nonFeatureChainPrimaryExpression
    : bracketExpression
    | indexExpression
    | sequenceExpression
    | selectExpression
    | collectExpression
    | functionOperationExpression
    | baseExpression
    ; // : Expression

nonFeatureChainPrimaryArgumentValue
    : nonFeatureChainPrimaryExpression
    ; // : FeatureValue

nonFeatureChainPrimaryArgument
    : nonFeatureChainPrimaryExpression
    ; // : Feature

nonFeatureChainPrimaryArgumentMember
    : primaryExpression
    ; // : ParameterMembership

bracketExpression
    : primaryExpression '[' sequenceExpressionList ']'
    ; // : OperatorExpression

indexExpression
    : primaryExpression '#' '(' sequenceExpressionList ')'
    ;

sequenceExpression
    : '(' sequenceExpressionList ')'
    ; // : Expression

sequenceExpressionList
    : ownedExpression ','?
    | sequenceOperatorExpression
    ; // : Expression

sequenceOperatorExpression
    : ownedExpression ',' sequenceExpressionList
    ; // : OperatorExpression

sequenceExpressionListMember
    : sequenceExpressionList
    ; // : FeatureMembership

featureChainExpression
    : primaryExpression '.' featureChainMember
    ;

collectExpression
    : primaryExpression '.' expressionBody
    ;

selectExpression
    : primaryExpression '.?' expressionBody
    ;

functionOperationExpression
    : primaryExpression '->' invocationTypeMember (expressionBody
    | referenceTyping
    | argumentList)
    ; // : InvocationExpression

bodyArgumentMember
    : expressionBody
    ; // : ParameterMembership

bodyArgument
    : expressionBody
    ; // : Feature

bodyArgumentValue
    : expressionBody
    ; // : FeatureValue

functionReferenceArgumentMember
    : referenceTyping
    ; // : ParameterMembership

functionReferenceArgument
    : referenceTyping
    ; // : Feature

functionReferenceArgumentValue
    : referenceTyping
    ; // : FeatureValue

functionReferenceExpression
    : referenceTyping
    ; // : FeatureReferenceExpression

functionReferenceMember
    : referenceTyping
    ; // : FeatureMembership

functionReference
    : referenceTyping
    ; // : Expression

featureChainMember
    : featureReference
    | featureChain
    ; // : Membership

ownedFeatureChainMember
    : featureChain
    ; // : OwningMembership

baseExpression
    : nullExpression
    | literalExpression
    | featureReferenceExpression
    | metadataAccessExpression
    | invocationExpression
    | constructorExpression
    | expressionBody
    ; // : Expression

nullExpression
    : 'null'
    | '(' ')'
    ; // : NullExpression

featureReferenceExpression
    : featureReference
    ; // : FeatureReferenceExpression

featureReferenceMember
    : featureReference
    ; // : Membership

featureReference
    : qualifiedName
    ; // : Feature

metadataAccessExpression
    : elementReferenceMember '.' 'metadata'
    ;

elementReferenceMember
    : qualifiedName
    ; // : Membership

invocationExpression
    : instantiatedTypeMember argumentList
    ; // : InvocationExpression

constructorExpression
    : 'new' instantiatedTypeMember argumentList
    ;

constructorResultMember
    : argumentList
    ; // : ReturnParameterMembership

constructorResult
    : argumentList
    ; // : Feature

instantiatedTypeMember
    : instantiatedTypeReference
    | featureChain
    ; // : Membership

instantiatedTypeReference
    : qualifiedName
    ; // : Type

argumentList
    : '(' ((positionalArgumentList
    | namedArgumentList))? ')'
    ; // : Feature

positionalArgumentList
    : ownedExpression ((',' ownedExpression))*
    ; // : Feature

namedArgumentList
    : namedArgument ((',' namedArgument))*
    ; // : Feature

namedArgumentMember
    : namedArgument
    ; // : FeatureMembership

namedArgument
    : parameterRedefinition '=' ownedExpression
    ; // : Feature

parameterRedefinition
    : qualifiedName
    ; // : Redefinition

bodyExpression
    : expressionBody
    ; // : FeatureReferenceExpression

expressionBodyMember
    : expressionBody
    ; // : FeatureMembership

expressionBody
    : '{' functionBodyPart '}'
    ; // : Expression

literalExpression
    : booleanValue
    | STRING_VALUE
    | DECIMAL_VALUE
    | realValue
    | literalInfinity
    ;

literalBoolean
    : booleanValue
    ;

booleanValue
    : 'true'
    | 'false'
    ; // : Boolean

literalString
    : STRING_VALUE
    ;

literalInteger
    : DECIMAL_VALUE
    ;

literalReal
    : realValue
    ;

realValue
    : DECIMAL_VALUE? '.' (DECIMAL_VALUE
    | EXPONENTIAL_VALUE)
    | EXPONENTIAL_VALUE
    ; // : Real

literalInfinity
    : '*'
    ;

interaction
    : typePrefix 'interaction' classifierDeclaration typeBody
    ;

flow
    : featurePrefix 'flow' flowDeclaration typeBody
    ;

successionFlow
    : featurePrefix 'succession' 'flow' flowDeclaration typeBody
    ;

flowDeclaration
    : featureDeclaration featureValue? (('of' payloadFeature))? (('from' flowEnd 'to' flowEnd))?
    | ('all'?)? flowEnd 'to' flowEnd
    ; // : Flow

payloadFeatureMember
    : payloadFeature
    ; // : FeatureMembership

payloadFeature
    : identification payloadFeatureSpecializationPart featureValue?
    | identification featureValue
    | generalType (multiplicityRange)?
    | multiplicityRange (generalType)?
    ;

payloadFeatureSpecializationPart
    : featureSpecialization+ multiplicityPart? featureSpecialization*
    | multiplicityPart featureSpecialization+
    ; // : Feature

flowEndMember
    : flowEnd
    ; // : EndFeatureMembership

flowEnd
    : ((generalType '.'))? flowFeatureRedefinition
    ;

flowFeatureMember
    : flowFeatureRedefinition
    ; // : FeatureMembership

flowFeature
    : flowFeatureRedefinition
    ; // : Feature

flowFeatureRedefinition
    : qualifiedName
    ; // : Redefinition

valuePart
    : featureValue
    ; // : Feature

featureValue
    : ('='
    | ':='?
    | 'default'? (('='
    | ':='?))?) ownedExpression
    ;

multiplicity
    : multiplicitySubset
    | multiplicityRange
    ;

multiplicitySubset
    : 'multiplicity' identification subsets typeBody
    ; // : Multiplicity

multiplicityRange
    : 'multiplicity' identification multiplicityBounds typeBody
    ;

ownedMultiplicity
    : multiplicityBounds
    ; // : OwningMembership

ownedMultiplicityRange
    : multiplicityBounds
    ; // : MultiplicityRange

multiplicityBounds
    : '[' ((multiplicityExpressionMember '..'))? multiplicityExpressionMember ']'
    ; // : MultiplicityRange

multiplicityExpressionMember
    : (literalExpression
    | featureReferenceExpression)
    ; // : OwningMembership

metaclass
    : typePrefix 'metaclass' classifierDeclaration typeBody
    ;

prefixMetadataAnnotation
    : '#' generalType
    ; // : Annotation

prefixMetadataMember
    : '#' generalType
    ; // : OwningMembership

prefixMetadataFeature
    : generalType
    ; // : MetadataFeature

metadataFeature
    : (prefixMetadataMember)* ('@'
    | 'metadata') metadataFeatureDeclaration (('about' annotation ((',' annotation))*))? metadataBody
    ;

metadataFeatureDeclaration
    : ((identification (':'
    | 'typed' 'by')))? generalType
    ; // : MetadataFeature

metadataBody
    : ';'
    | '{' (metadataBodyElement)* '}'
    ; // : Feature

metadataBodyElement
    : nonFeatureMember
    | metadataBodyFeature
    | aliasMember
    | import_
    ; // : Membership

metadataBodyFeatureMember
    : metadataBodyFeature
    ; // : FeatureMembership

metadataBodyFeature
    : 'feature'? ((':>>'
    | 'redefines'))? generalType featureSpecializationPart? featureValue? metadataBody
    ; // : Feature

package
    : (prefixMetadataMember)* packageDeclaration packageBody
    ;

libraryPackage
    : ('standard'?) 'library' (prefixMetadataMember)* packageDeclaration packageBody
    ;

packageDeclaration
    : 'package' identification
    ; // : Package

packageBody
    : ';'
    | '{' ((namespaceBodyElement
    | elementFilterMember))* '}'
    ; // : Package

elementFilterMember
    : memberPrefix 'filter' ownedExpression ';'
    ; // : ElementFilterMembership

dependencyDeclaration
    : ((identification 'from'))? qualifiedName ((',' qualifiedName))* 'to' qualifiedName ((',' qualifiedName))*
    ;

annotatingMember
    : annotatingElement
    ; // : OwningMembership

packageBodyElement
    : packageMember
    | elementFilterMember
    | aliasMember
    | import_
    ; // : Package

packageMember
    : memberPrefix (definitionElement
    | usageElement)
    ; // : OwningMembership

definitionElement
    : package
    | libraryPackage
    | annotatingElement
    | dependency
    | attributeDefinition
    | enumerationDefinition
    | occurrenceDefinition
    | individualDefinition
    | itemDefinition
    | partDefinition
    | connectionDefinition
    | flowDefinition
    | interfaceDefinition
    | portDefinition
    | actionDefinition
    | calculationDefinition
    | stateDefinition
    | constraintDefinition
    | requirementDefinition
    | concernDefinition
    | caseDefinition
    | analysisCaseDefinition
    | verificationCaseDefinition
    | useCaseDefinition
    | viewDefinition
    | viewpointDefinition
    | renderingDefinition
    | metadataDefinition
    | extendedDefinition
    ; // : Element

usageElement
    : nonOccurrenceUsageElement
    | occurrenceUsageElement
    ; // : Usage

basicDefinitionPrefix
    : 'abstract'?
    | 'variation'?
    ;

definitionExtensionKeyword
    : prefixMetadataMember
    ; // : Definition

definitionPrefix
    : basicDefinitionPrefix? prefixMetadataMember*
    ; // : Definition

definition
    : definitionDeclaration definitionBody
    ;

definitionDeclaration
    : identification subclassificationPart?
    ; // : Definition

definitionBody
    : ';'
    | '{' definitionBodyItem* '}'
    ; // : Type

definitionBodyItem
    : definitionMember
    | variantUsageMember
    | nonOccurrenceUsageMember
    | (sourceSuccessionMember)? occurrenceUsageMember
    | aliasMember
    | import_
    ; // : Type

definitionMember
    : memberPrefix definitionElement
    ; // : OwningMembership

variantUsageMember
    : memberPrefix 'variant' variantUsageElement
    ; // : VariantMembership

nonOccurrenceUsageMember
    : memberPrefix nonOccurrenceUsageElement
    ; // : FeatureMembership

occurrenceUsageMember
    : memberPrefix occurrenceUsageElement
    ; // : FeatureMembership

structureUsageMember
    : memberPrefix structureUsageElement
    ; // : FeatureMembership

behaviorUsageMember
    : memberPrefix behaviorUsageElement
    ; // : FeatureMembership

refPrefix
    : (featureDirection)? ('derived'?)? (('abstract'?
    | 'variation'?))? ('constant'?)?
    ; // : Usage

basicUsagePrefix
    : refPrefix ('ref'?)?
    ; // : Usage

endUsagePrefix
    : 'end'? (ownedCrossFeature)?
    ; // : Usage

usageExtensionKeyword
    : prefixMetadataMember
    ; // : Usage

unextendedUsagePrefix
    : endUsagePrefix
    | basicUsagePrefix
    ; // : Usage

usagePrefix
    : unextendedUsagePrefix prefixMetadataMember*
    ; // : Usage

usage
    : usageDeclaration usageCompletion
    ;

usageDeclaration
    : identification featureSpecializationPart?
    ; // : Usage

usageCompletion
    : featureValue? definitionBody
    ; // : Usage

usageBody
    : definitionBody
    ; // : Usage

defaultReferenceUsage
    : refPrefix usage
    ; // : ReferenceUsage

referenceUsage
    : (endUsagePrefix
    | refPrefix) 'ref' usage
    ;

variantReference
    : generalType featureSpecialization* definitionBody
    ; // : ReferenceUsage

nonOccurrenceUsageElement
    : defaultReferenceUsage
    | referenceUsage
    | attributeUsage
    | enumerationUsage
    | bindingConnectorAsUsage
    | successionAsUsage
    | extendedUsage
    ; // : Usage

occurrenceUsageElement
    : structureUsageElement
    | behaviorUsageElement
    ; // : Usage

structureUsageElement
    : occurrenceUsage
    | individualUsage
    | portionUsage
    | eventOccurrenceUsage
    | itemUsage
    | partUsage
    | viewUsage
    | renderingUsage
    | portUsage
    | connectionUsage
    | interfaceUsage
    | allocationUsage
    | message
    | flowUsage
    | successionFlowUsage
    ; // : Usage

behaviorUsageElement
    : actionUsage
    | calculationUsage
    | stateUsage
    | constraintUsage
    | requirementUsage
    | concernUsage
    | caseUsage
    | analysisCaseUsage
    | verificationCaseUsage
    | useCaseUsage
    | viewpointUsage
    | performActionUsage
    | exhibitStateUsage
    | includeUseCaseUsage
    | assertConstraintUsage
    | satisfyRequirementUsage
    ; // : Usage

variantUsageElement
    : variantReference
    | referenceUsage
    | attributeUsage
    | bindingConnectorAsUsage
    | successionAsUsage
    | occurrenceUsage
    | individualUsage
    | portionUsage
    | eventOccurrenceUsage
    | itemUsage
    | partUsage
    | viewUsage
    | renderingUsage
    | portUsage
    | connectionUsage
    | interfaceUsage
    | allocationUsage
    | message
    | flowUsage
    | successionFlowUsage
    | behaviorUsageElement
    ; // : Usage

subclassificationPart
    : SPECIALIZES ownedSubclassification ((',' ownedSubclassification))*
    ; // : Classifier

attributeDefinition
    : definitionPrefix 'attribute' 'def' definition
    ; // : AttributeDefinition

attributeUsage
    : usagePrefix 'attribute' usage
    ; // : AttributeUsage

enumerationDefinition
    : prefixMetadataMember* 'enum' 'def' definitionDeclaration enumerationBody
    ;

enumerationBody
    : ';'
    | '{' ((annotatingElement
    | enumerationUsageMember))* '}'
    ; // : EnumerationDefinition

enumerationUsageMember
    : memberPrefix enumeratedValue
    ; // : VariantMembership

enumeratedValue
    : 'enum'? usage
    ; // : EnumerationUsage

enumerationUsage
    : usagePrefix 'enum' usage
    ; // : EnumerationUsage

occurrenceDefinitionPrefix
    : basicDefinitionPrefix? ('individual'?)? prefixMetadataMember*
    ; // : OccurrenceDefinition

occurrenceDefinition
    : occurrenceDefinitionPrefix 'occurrence' 'def' definition
    ;

individualDefinition
    : basicDefinitionPrefix? 'individual'? prefixMetadataMember* 'def' definition
    ; // : OccurrenceDefinition

occurrenceUsagePrefix
    : basicUsagePrefix ('individual'?)? (portionKind)? prefixMetadataMember*
    ; // : OccurrenceUsage

occurrenceUsage
    : occurrenceUsagePrefix 'occurrence' usage
    ;

individualUsage
    : basicUsagePrefix 'individual'? prefixMetadataMember* usage
    ; // : OccurrenceUsage

portionUsage
    : basicUsagePrefix ('individual'?)? portionKind prefixMetadataMember* usage
    ; // : OccurrenceUsage

portionKind
    : 'snapshot'
    | 'timeslice'
    ;

eventOccurrenceUsage
    : occurrenceUsagePrefix 'event' (generalType featureSpecializationPart?
    | 'occurrence' usageDeclaration?) usageCompletion
    ;

sourceSuccessionMember
    : 'then' sourceEnd
    ; // : FeatureMembership

sourceSuccession
    : sourceEnd
    ; // : SuccessionAsUsage

sourceEndMember
    : sourceEnd
    ; // : EndFeatureMembership

sourceEnd
    : (multiplicityRange)?
    ; // : ReferenceUsage

itemDefinition
    : occurrenceDefinitionPrefix 'item' 'def' definition
    ;

itemUsage
    : occurrenceUsagePrefix 'item' usage
    ;

partDefinition
    : occurrenceDefinitionPrefix 'part' 'def' definition
    ;

partUsage
    : occurrenceUsagePrefix 'part' usage
    ;

portDefinition
    : definitionPrefix 'port' 'def' definition
    ;

portUsage
    : occurrenceUsagePrefix 'port' usage
    ;

conjugatedPortTyping
    : '~' qualifiedName
    ; // : ConjugatedPortTyping

connectionDefinition
    : occurrenceDefinitionPrefix 'connection' 'def' definition
    ;

connectionUsage
    : occurrenceUsagePrefix ('connection' usageDeclaration featureValue? (('connect' connectorPart))?
    | 'connect' connectorPart) definitionBody
    ;

connectorPart
    : binaryConnectorPart
    | naryConnectorPart
    ; // : ConnectionUsage

binaryConnectorPart
    : connectorEnd 'to' connectorEnd
    ; // : ConnectionUsage

naryConnectorPart
    : '(' connectorEnd ',' connectorEnd ((',' connectorEnd))* ')'
    ; // : ConnectionUsage

bindingConnectorAsUsage
    : usagePrefix (('binding' usageDeclaration))? 'bind' connectorEnd '=' connectorEnd definitionBody
    ;

successionAsUsage
    : usagePrefix (('succession' usageDeclaration))? 'first' connectorEnd 'then' connectorEnd definitionBody
    ;

interfaceDefinition
    : occurrenceDefinitionPrefix 'interface' 'def' definitionDeclaration interfaceBody
    ;

interfaceBody
    : ';'
    | '{' interfaceBodyItem* '}'
    ; // : Type

interfaceBodyItem
    : definitionMember
    | variantUsageMember
    | interfaceNonOccurrenceUsageMember
    | (sourceSuccessionMember)? interfaceOccurrenceUsageMember
    | aliasMember
    | import_
    ; // : Type

interfaceNonOccurrenceUsageMember
    : memberPrefix interfaceNonOccurrenceUsageElement
    ; // : FeatureMembership

interfaceNonOccurrenceUsageElement
    : referenceUsage
    | attributeUsage
    | enumerationUsage
    | bindingConnectorAsUsage
    | successionAsUsage
    ; // : Usage

interfaceOccurrenceUsageMember
    : memberPrefix interfaceOccurrenceUsageElement
    ; // : FeatureMembership

interfaceOccurrenceUsageElement
    : defaultInterfaceEnd
    | structureUsageElement
    | behaviorUsageElement
    ; // : Usage

defaultInterfaceEnd
    : 'end'? usage
    ; // : PortUsage

interfaceUsage
    : occurrenceUsagePrefix 'interface' interfaceUsageDeclaration interfaceBody
    ;

interfaceUsageDeclaration
    : usageDeclaration featureValue? (('connect' interfacePart))?
    | interfacePart
    ; // : InterfaceUsage

interfacePart
    : binaryInterfacePart
    | naryInterfacePart
    ; // : InterfaceUsage

binaryInterfacePart
    : interfaceEnd 'to' interfaceEnd
    ; // : InterfaceUsage

naryInterfacePart
    : '(' interfaceEnd ',' interfaceEnd ((',' interfaceEnd))* ')'
    ; // : InterfaceUsage

interfaceEndMember
    : interfaceEnd
    ; // : EndFeatureMembership

interfaceEnd
    : (multiplicityRange)? ((NAME REFERENCES))? generalType
    ; // : PortUsage

allocationDefinition
    : occurrenceDefinitionPrefix 'allocation' 'def' definition
    ;

allocationUsage
    : occurrenceUsagePrefix allocationUsageDeclaration definitionBody
    ;

allocationUsageDeclaration
    : 'allocation' usageDeclaration (('allocate' connectorPart))?
    | 'allocate' connectorPart
    ; // : AllocationUsage

flowDefinition
    : occurrenceDefinitionPrefix 'flow' 'def' definition
    ;

message
    : occurrenceUsagePrefix 'message' messageDeclaration definitionBody
    ; // : FlowUsage

messageDeclaration
    : usageDeclaration featureValue? (('of' payloadFeature))? (('from' generalType 'to' generalType))?
    | generalType 'to' generalType
    ; // : FlowUsage

messageEventMember
    : generalType
    ; // : ParameterMembership

messageEvent
    : generalType
    ; // : EventOccurrenceUsage

flowUsage
    : occurrenceUsagePrefix 'flow' flowDeclaration definitionBody
    ;

successionFlowUsage
    : occurrenceUsagePrefix 'succession' 'flow' flowDeclaration definitionBody
    ;

flowPayloadFeatureMember
    : payloadFeature
    ; // : FeatureMembership

flowPayloadFeature
    : payloadFeature
    ; // : PayloadFeature

flowEndSubsetting
    : qualifiedName
    | featureChainPrefix
    ; // : ReferenceSubsetting

featureChainPrefix
    : ((ownedFeatureChaining '.'))+ ownedFeatureChaining '.'
    ; // : Feature

actionDefinition
    : occurrenceDefinitionPrefix 'action' 'def' definitionDeclaration actionBody
    ;

actionBody
    : ';'
    | '{' actionBodyItem* '}'
    ; // : Type

actionBodyItem
    : nonBehaviorBodyItem
    | initialNodeMember (actionTargetSuccessionMember)*
    | (sourceSuccessionMember)? actionBehaviorMember (actionTargetSuccessionMember)*
    | guardedSuccessionMember
    ; // : Type

nonBehaviorBodyItem
    : import_
    | aliasMember
    | definitionMember
    | variantUsageMember
    | nonOccurrenceUsageMember
    | (sourceSuccessionMember)? structureUsageMember
    ;

actionBehaviorMember
    : behaviorUsageMember
    | actionNodeMember
    ; // : FeatureMembership

initialNodeMember
    : memberPrefix 'first' qualifiedName relationshipBody
    ; // : FeatureMembership

actionNodeMember
    : memberPrefix actionNode
    ; // : FeatureMembership

actionTargetSuccessionMember
    : memberPrefix actionTargetSuccession
    ; // : FeatureMembership

guardedSuccessionMember
    : memberPrefix guardedSuccession
    ; // : FeatureMembership

actionUsage
    : occurrenceUsagePrefix 'action' actionUsageDeclaration actionBody
    ;

actionUsageDeclaration
    : usageDeclaration featureValue?
    ; // : ActionUsage

performActionUsage
    : occurrenceUsagePrefix 'perform' performActionUsageDeclaration actionBody
    ;

performActionUsageDeclaration
    : (generalType featureSpecializationPart?
    | 'action' usageDeclaration) featureValue?
    ; // : PerformActionUsage

actionNode
    : controlNode
    | sendNode
    | acceptNode
    | assignmentNode
    | terminateNode
    | ifNode
    | whileLoopNode
    | forLoopNode
    ; // : ActionUsage

actionNodeUsageDeclaration
    : 'action' usageDeclaration?
    ; // : ActionUsage

actionNodePrefix
    : occurrenceUsagePrefix actionNodeUsageDeclaration?
    ; // : ActionUsage

controlNode
    : mergeNode
    | decisionNode
    | joinNode
    | forkNode
    ;

controlNodePrefix
    : refPrefix ('individual'?)? (portionKind)? prefixMetadataMember*
    ; // : OccurrenceUsage

mergeNode
    : controlNodePrefix 'merge'? usageDeclaration actionBody
    ;

decisionNode
    : controlNodePrefix 'decide'? usageDeclaration actionBody
    ;

joinNode
    : controlNodePrefix 'join'? usageDeclaration actionBody
    ;

forkNode
    : controlNodePrefix 'fork'? usageDeclaration actionBody
    ;

acceptNode
    : occurrenceUsagePrefix acceptNodeDeclaration actionBody
    ; // : AcceptActionUsage

acceptNodeDeclaration
    : actionNodeUsageDeclaration? 'accept' acceptParameterPart
    ; // : AcceptActionUsage

acceptParameterPart
    : payloadParameter (('via' ownedExpression))?
    ; // : AcceptActionUsage

payloadParameterMember
    : payloadParameter
    ; // : ParameterMembership

payloadParameter
    : payloadFeature
    | identification payloadFeatureSpecializationPart? triggerExpression
    ; // : ReferenceUsage

triggerValuePart
    : triggerExpression
    ; // : Feature

triggerFeatureValue
    : triggerExpression
    ; // : FeatureValue

triggerExpression
    : ('at'
    | 'after') ownedExpression
    | 'when' ownedExpression
    ; // : TriggerInvocationExpression

sendNode
    : occurrenceUsagePrefix actionUsageDeclaration? 'send' ((ownedExpression senderReceiverPart?
    | senderReceiverPart))? actionBody
    ; // : SendActionUsage

sendNodeDeclaration
    : actionNodeUsageDeclaration? 'send' ownedExpression senderReceiverPart?
    ; // : SendActionUsage

senderReceiverPart
    : 'via' ownedExpression (('to' ownedExpression))?
    | 'to' ownedExpression
    ; // : SendActionUsage

nodeParameterMember
    : ownedExpression
    ; // : ParameterMembership

nodeParameter
    : ownedExpression
    ; // : ReferenceUsage

featureBinding
    : ownedExpression
    ; // : FeatureValue

assignmentNode
    : occurrenceUsagePrefix assignmentNodeDeclaration actionBody
    ; // : AssignmentActionUsage

assignmentNodeDeclaration
    : (actionNodeUsageDeclaration)? 'assign' assignmentTargetParameter featureChainMember ':=' ownedExpression
    ; // : ActionUsage

assignmentTargetMember
    : assignmentTargetParameter
    ; // : ParameterMembership

assignmentTargetParameter
    : ((nonFeatureChainPrimaryExpression '.'))?
    ; // : ReferenceUsage

assignmentTargetBinding
    : nonFeatureChainPrimaryExpression
    ; // : FeatureValue

terminateNode
    : occurrenceUsagePrefix actionNodeUsageDeclaration? 'terminate' (ownedExpression)? actionBody
    ; // : TerminateActionUsage

ifNode
    : actionNodePrefix 'if' ownedExpression actionBodyParameter (('else' (actionBodyParameter
    | ifNode)))?
    ; // : IfActionUsage

expressionParameterMember
    : ownedExpression
    ; // : ParameterMembership

actionBodyParameterMember
    : actionBodyParameter
    ; // : ParameterMembership

actionBodyParameter
    : (('action' usageDeclaration?))? '{' actionBodyItem* '}'
    ; // : ActionUsage

ifNodeParameterMember
    : ifNode
    ; // : ParameterMembership

whileLoopNode
    : actionNodePrefix ('while' ownedExpression
    | 'loop') actionBodyParameter (('until' ownedExpression ';'))?
    ; // : WhileLoopActionUsage

forLoopNode
    : actionNodePrefix 'for' usageDeclaration 'in' ownedExpression actionBodyParameter
    ; // : ForLoopActionUsage

forVariableDeclarationMember
    : usageDeclaration
    ; // : FeatureMembership

forVariableDeclaration
    : usageDeclaration
    ; // : ReferenceUsage

actionTargetSuccession
    : (targetSuccession
    | guardedTargetSuccession
    | defaultTargetSuccession) definitionBody
    ; // : Usage

targetSuccession
    : sourceEnd 'then' connectorEnd
    ; // : SuccessionAsUsage

guardedTargetSuccession
    : guardExpressionMember 'then' transitionSuccession
    ; // : TransitionUsage

defaultTargetSuccession
    : 'else' transitionSuccession
    ; // : TransitionUsage

guardedSuccession
    : (('succession' usageDeclaration))? 'first' featureChainMember guardExpressionMember 'then' transitionSuccession definitionBody
    ; // : TransitionUsage

stateDefinition
    : occurrenceDefinitionPrefix 'state' 'def' definitionDeclaration stateDefBody
    ;

stateDefBody
    : ';'
    | ('parallel'?)? '{' stateBodyItem* '}'
    ; // : StateDefinition

stateBodyItem
    : nonBehaviorBodyItem
    | (sourceSuccessionMember)? behaviorUsageMember (targetTransitionUsageMember)*
    | transitionUsageMember
    | entryActionMember (entryTransitionMember)*
    | doActionMember
    | exitActionMember
    ; // : Type

entryActionMember
    : memberPrefix 'entry' stateActionUsage
    ; // : StateSubactionMembership

doActionMember
    : memberPrefix 'do' stateActionUsage
    ; // : StateSubactionMembership

exitActionMember
    : memberPrefix 'exit' stateActionUsage
    ; // : StateSubactionMembership

entryTransitionMember
    : memberPrefix (guardedTargetSuccession
    | 'then' targetSuccession) ';'
    ; // : FeatureMembership

stateActionUsage
    : ';'
    | statePerformActionUsage
    | stateAcceptActionUsage
    | stateSendActionUsage
    | stateAssignmentActionUsage
    ; // : ActionUsage

statePerformActionUsage
    : performActionUsageDeclaration actionBody
    ; // : PerformActionUsage

stateAcceptActionUsage
    : acceptNodeDeclaration actionBody
    ; // : AcceptActionUsage

stateSendActionUsage
    : sendNodeDeclaration actionBody
    ; // : SendActionUsage

stateAssignmentActionUsage
    : assignmentNodeDeclaration actionBody
    ; // : AssignmentActionUsage

transitionUsageMember
    : memberPrefix transitionUsage
    ; // : FeatureMembership

targetTransitionUsageMember
    : memberPrefix targetTransitionUsage
    ; // : FeatureMembership

stateUsage
    : occurrenceUsagePrefix 'state' actionUsageDeclaration stateUsageBody
    ;

stateUsageBody
    : ';'
    | ('parallel'?)? '{' stateBodyItem* '}'
    ; // : StateUsage

exhibitStateUsage
    : occurrenceUsagePrefix 'exhibit' (generalType featureSpecializationPart?
    | 'state' usageDeclaration) featureValue? stateUsageBody
    ;

transitionUsage
    : 'transition' ((usageDeclaration 'first'))? featureChainMember (triggerActionMember)? (guardExpressionMember)? (effectBehaviorMember)? 'then' transitionSuccession actionBody
    ;

targetTransitionUsage
    : (('transition' (triggerActionMember)? (guardExpressionMember)? (effectBehaviorMember)?
    | triggerActionMember (guardExpressionMember)? (effectBehaviorMember)?
    | guardExpressionMember (effectBehaviorMember)?))? 'then' transitionSuccession actionBody
    ; // : TransitionUsage

triggerActionMember
    : 'accept' acceptParameterPart
    ; // : TransitionFeatureMembership

triggerAction
    : acceptParameterPart
    ; // : AcceptActionUsage

guardExpressionMember
    : 'if' ownedExpression
    ; // : TransitionFeatureMembership

effectBehaviorMember
    : 'do' effectBehaviorUsage
    ; // : TransitionFeatureMembership

effectBehaviorUsage
    : transitionPerformActionUsage
    | transitionAcceptActionUsage
    | transitionSendActionUsage
    | transitionAssignmentActionUsage
    ; // : ActionUsage

transitionPerformActionUsage
    : performActionUsageDeclaration (('{' actionBodyItem* '}'))?
    ; // : PerformActionUsage

transitionAcceptActionUsage
    : acceptNodeDeclaration (('{' actionBodyItem* '}'))?
    ; // : AcceptActionUsage

transitionSendActionUsage
    : sendNodeDeclaration (('{' actionBodyItem* '}'))?
    ; // : SendActionUsage

transitionAssignmentActionUsage
    : assignmentNodeDeclaration (('{' actionBodyItem* '}'))?
    ; // : AssignmentActionUsage

transitionSuccessionMember
    : transitionSuccession
    ; // : OwningMembership

transitionSuccession
    : connectorEnd
    ; // : Succession

calculationDefinition
    : occurrenceDefinitionPrefix 'calc' 'def' definitionDeclaration calculationBody
    ;

calculationUsage
    : occurrenceUsagePrefix 'calc' actionUsageDeclaration calculationBody
    ; // : CalculationUsage

calculationBody
    : ';'
    | '{' calculationBodyPart '}'
    ; // : Type

calculationBodyPart
    : calculationBodyItem* (resultExpressionMember)?
    ; // : Type

calculationBodyItem
    : actionBodyItem
    | returnParameterMember
    ; // : Type

returnParameterMember
    : memberPrefix? 'return' usageElement
    ; // : ReturnParameterMembership

constraintDefinition
    : occurrenceDefinitionPrefix 'constraint' 'def' definitionDeclaration calculationBody
    ;

constraintUsage
    : occurrenceUsagePrefix 'constraint' constraintUsageDeclaration calculationBody
    ;

assertConstraintUsage
    : occurrenceUsagePrefix 'assert' ('not'?)? (generalType featureSpecializationPart?
    | 'constraint' constraintUsageDeclaration) calculationBody
    ;

constraintUsageDeclaration
    : usageDeclaration featureValue?
    ; // : ConstraintUsage

requirementDefinition
    : occurrenceDefinitionPrefix 'requirement' 'def' definitionDeclaration requirementBody
    ;

requirementBody
    : ';'
    | '{' requirementBodyItem* '}'
    ; // : Type

requirementBodyItem
    : definitionBodyItem
    | subjectMember
    | requirementConstraintMember
    | framedConcernMember
    | requirementVerificationMember
    | actorMember
    | stakeholderMember
    ; // : Type

subjectMember
    : memberPrefix subjectUsage
    ; // : SubjectMembership

subjectUsage
    : 'subject' prefixMetadataMember* usage
    ; // : ReferenceUsage

requirementConstraintMember
    : memberPrefix? requirementKind requirementConstraintUsage
    ; // : RequirementConstraintMembership

requirementKind
    : 'assume'
    | 'require'
    ; // : RequirementConstraintMembership

requirementConstraintUsage
    : generalType featureSpecializationPart? requirementBody
    | (prefixMetadataMember* 'constraint'
    | prefixMetadataMember+) constraintUsageDeclaration calculationBody
    ; // : ConstraintUsage

framedConcernMember
    : memberPrefix? 'frame' framedConcernUsage
    ; // : FramedConcernMembership

framedConcernUsage
    : generalType featureSpecializationPart? calculationBody
    | (prefixMetadataMember* 'concern'
    | prefixMetadataMember+) calculationUsageDeclaration calculationBody
    ; // : ConcernUsage

actorMember
    : memberPrefix actorUsage
    ; // : ActorMembership

actorUsage
    : 'actor' prefixMetadataMember* usage
    ; // : PartUsage

stakeholderMember
    : memberPrefix stakeholderUsage
    ; // : StakeholderMembership

stakeholderUsage
    : 'stakeholder' prefixMetadataMember* usage
    ; // : PartUsage

requirementUsage
    : occurrenceUsagePrefix 'requirement' constraintUsageDeclaration requirementBody
    ;

satisfyRequirementUsage
    : occurrenceUsagePrefix 'assert' ('not'?) 'satisfy' (generalType featureSpecializationPart?
    | 'requirement' usageDeclaration) featureValue? (('by' featureChainMember))? requirementBody
    ;

satisfactionSubjectMember
    : featureChainMember
    ; // : SubjectMembership

satisfactionParameter
    : featureChainMember
    ; // : ReferenceUsage

satisfactionFeatureValue
    : featureChainMember
    ; // : FeatureValue

satisfactionReferenceExpression
    : featureChainMember
    ; // : FeatureReferenceExpression

concernDefinition
    : occurrenceDefinitionPrefix 'concern' 'def' definitionDeclaration requirementBody
    ;

concernUsage
    : occurrenceUsagePrefix 'concern' constraintUsageDeclaration requirementBody
    ;

caseDefinition
    : occurrenceDefinitionPrefix 'case' 'def' definitionDeclaration caseBody
    ;

caseUsage
    : occurrenceUsagePrefix 'case' constraintUsageDeclaration caseBody
    ;

caseBody
    : ';'
    | '{' caseBodyItem* (resultExpressionMember)? '}'
    ; // : Type

caseBodyItem
    : actionBodyItem
    | subjectMember
    | actorMember
    | objectiveMember
    ; // : Type

objectiveMember
    : memberPrefix 'objective' objectiveRequirementUsage
    ; // : ObjectiveMembership

objectiveRequirementUsage
    : prefixMetadataMember* constraintUsageDeclaration requirementBody
    ; // : RequirementUsage

analysisCaseDefinition
    : occurrenceDefinitionPrefix 'analysis' 'def' definitionDeclaration caseBody
    ;

analysisCaseUsage
    : occurrenceUsagePrefix 'analysis' constraintUsageDeclaration caseBody
    ;

verificationCaseDefinition
    : occurrenceDefinitionPrefix 'verification' 'def' definitionDeclaration caseBody
    ;

verificationCaseUsage
    : occurrenceUsagePrefix 'verification' constraintUsageDeclaration caseBody
    ;

requirementVerificationMember
    : memberPrefix 'verify' requirementVerificationUsage
    ; // : RequirementVerificationMembership

requirementVerificationUsage
    : generalType featureSpecialization* requirementBody
    | (prefixMetadataMember* 'requirement'
    | prefixMetadataMember+) constraintUsageDeclaration requirementBody
    ; // : RequirementUsage

useCaseDefinition
    : occurrenceDefinitionPrefix 'use' 'case' 'def' definitionDeclaration caseBody
    ;

useCaseUsage
    : occurrenceUsagePrefix 'use' 'case' constraintUsageDeclaration caseBody
    ;

includeUseCaseUsage
    : occurrenceUsagePrefix 'include' (generalType featureSpecializationPart?
    | 'use' 'case' usageDeclaration) featureValue? caseBody
    ;

viewDefinition
    : occurrenceDefinitionPrefix 'view' 'def' definitionDeclaration viewDefinitionBody
    ;

viewDefinitionBody
    : ';'
    | '{' viewDefinitionBodyItem* '}'
    ; // : ViewDefinition

viewDefinitionBodyItem
    : definitionBodyItem
    | elementFilterMember
    | viewRenderingMember
    ; // : ViewDefinition

viewRenderingMember
    : memberPrefix 'render' viewRenderingUsage
    ; // : ViewRenderingMembership

viewRenderingUsage
    : generalType featureSpecializationPart? definitionBody
    | (prefixMetadataMember* 'rendering'
    | prefixMetadataMember+) usage
    ; // : RenderingUsage

viewUsage
    : occurrenceUsagePrefix 'view' usageDeclaration? featureValue? viewBody
    ;

viewBody
    : ';'
    | '{' viewBodyItem* '}'
    ; // : ViewUsage

viewBodyItem
    : definitionBodyItem
    | elementFilterMember
    | viewRenderingMember
    | expose
    ; // : ViewUsage

expose
    : 'expose' (membershipImport
    | namespaceImport) relationshipBody
    ;

membershipExpose
    : membershipImport
    ;

namespaceExpose
    : namespaceImport
    ;

viewpointDefinition
    : occurrenceDefinitionPrefix 'viewpoint' 'def' definitionDeclaration requirementBody
    ;

viewpointUsage
    : occurrenceUsagePrefix 'viewpoint' constraintUsageDeclaration requirementBody
    ;

renderingDefinition
    : occurrenceDefinitionPrefix 'rendering' 'def' definition
    ;

renderingUsage
    : occurrenceUsagePrefix 'rendering' usage
    ;

metadataDefinition
    : ('abstract'?)? prefixMetadataMember* 'metadata' 'def' definition
    ;

prefixMetadataUsage
    : generalType
    ; // : MetadataUsage

metadataUsage
    : prefixMetadataMember* ('@'
    | 'metadata') metadataUsageDeclaration (('about' annotation ((',' annotation))*))? metadataBody
    ;

metadataUsageDeclaration
    : ((identification (':'
    | 'typed' 'by')))? generalType
    ; // : MetadataUsage

metadataBodyUsageMember
    : metadataBodyUsage
    ; // : FeatureMembership

metadataBodyUsage
    : 'ref'? ((':>>'
    | 'redefines'))? generalType featureSpecializationPart? featureValue? metadataBody
    ; // : ReferenceUsage

extendedDefinition
    : basicDefinitionPrefix? prefixMetadataMember+ 'def' definition
    ; // : Definition

extendedUsage
    : unextendedUsagePrefix prefixMetadataMember+ usage
    ; // : Usage

// ─── Lexer rules (from KeBNF) ────────────────────────────

RESERVED_KEYWORD
    : 'about'
    | 'abstract'
    | 'alias'
    | 'all'
    | 'and'
    | 'as'
    | 'assoc'
    | 'behavior'
    | 'binding'
    | 'bool'
    | 'by'
    | 'chains'
    | 'class'
    | 'classifier'
    | 'comment'
    | 'composite'
    | 'conjugate'
    | 'conjugates'
    | 'conjugation'
    | 'connector'
    | 'const'
    | 'crosses'
    | 'datatype'
    | 'default'
    | 'dependency'
    | 'derived'
    | 'differences'
    | 'disjoining'
    | 'disjoint'
    | 'doc'
    | 'else'
    | 'end'
    | 'expr'
    | 'false'
    | 'feature'
    | 'featured'
    | 'featuring'
    | 'filter'
    | 'first'
    | 'flow'
    | 'for'
    | 'from'
    | 'function'
    | 'hastype'
    | 'if'
    | 'implies'
    | 'import'
    | 'in'
    | 'inout'
    | 'interaction'
    | 'intersects'
    | 'inv'
    | 'inverse'
    | 'inverting'
    | 'istype'
    | 'language'
    | 'library'
    | 'locale'
    | 'member'
    | 'meta'
    | 'metaclass'
    | 'metadata'
    | 'multiplicity'
    | 'namespace'
    | 'nonunique'
    | 'not'
    | 'null'
    | 'of'
    | 'or'
    | 'ordered'
    | 'out'
    | 'package'
    | 'portion'
    | 'predicate'
    | 'private'
    | 'protected'
    | 'public'
    | 'redefines'
    | 'redefinition'
    | 'references'
    | 'rep'
    | 'return'
    | 'specialization'
    | 'specializes'
    | 'standard'
    | 'step'
    | 'struct'
    | 'subclassifier'
    | 'subset'
    | 'subsets'
    | 'subtype'
    | 'succession'
    | 'then'
    | 'to'
    | 'true'
    | 'type'
    | 'typed'
    | 'typing'
    | 'unions'
    | 'var'
    | 'xor'
    ;

RESERVED_SYMBOL
    : '~'
    | '}'
    | '|'
    | '{'
    | '^'
    | ']'
    | '['
    | '@'
    | '??'
    | '?'
    | '>='
    | '>'
    | '=>'
    | '==='
    | '=='
    | '='
    | '<='
    | '<'
    | ';'
    | ':>>'
    | ':>'
    | ':='
    | '::>'
    | '::'
    | ':'
    | '/'
    | '.?'
    | '..'
    | '.'
    | '->'
    | '-'
    | ','
    | '+'
    | '**'
    | '*'
    | ')'
    | '('
    | '&'
    | '%'
    | '$'
    | '#'
    | '!=='
    | '!='
    ;

TYPED_BY
    : ':'
    | 'typed' 'by'
    ;

SPECIALIZES
    : ':>'
    | 'specializes'
    ;

SUBSETS
    : ':>'
    | 'subsets'
    ;

REFERENCES
    : '::>'
    | 'references'
    ;

CROSSES
    : '=>'
    | 'crosses'
    ;

REDEFINES
    : ':>>'
    | 'redefines'
    ;

CONJUGATES
    : '~'
    | 'conjugates'
    ;

DEFINED_BY
    : ':'
    | 'defined' 'by'
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

calculationUsageDeclaration
    : NAME // stub: CalculationUsageDeclaration not defined in KeBNF source
    ;

filterPackageImport
    : NAME // stub: FilterPackageImport not defined in KeBNF source
    ;

invocationTypeMember
    : NAME // stub: InvocationTypeMember not defined in KeBNF source
    ;

