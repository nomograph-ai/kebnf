# Tree-sitter Emitter: Journey from Mechanical to Structural

## Research Finding

Mechanical 1:1 conversion of KeBNF grammar rules to tree-sitter produces a
grammar that **generates** (valid parser.c) but does not **parse** in practical
time. The gap between generation and parsing is structural, not quantitative.

## The Experiment

### Phase 1: Naive Conversion (baseline)
- 546 rules, 0 prec calls, 0 conflict declarations
- tree-sitter generate: **hangs** (>3 minutes, no output)
- Root cause: combinatorial LR table explosion from 546 conflicting rules

### Phase 2: Structural Reductions
Applied 6 transformations to reduce rule count and add disambiguation:
1. Wrapper rule inlining (inline_map) -- resolve A->B->C chains
2. Epsilon rule elimination -- drop semantic-only rules
3. Body member flattening -- merge context-sensitive rules into _body_member
4. Prefix rule unification -- merge 14 prefix rules into unified_prefix
5. Expression precedence tower -- 18 prec.left/prec.right calls
6. Can-match-empty handling -- hardcoded non-empty replacements

Result: 478 rules, tree-sitter generate **succeeds** in <1 second.
But parsing times out on even a 1-line file.

### Phase 3: Conflict Resolution (automated)
Used automated loop to add conflict declarations:
- Parsed tree-sitter diagnostics programmatically
- Added exact conflict sets from error messages
- Resolved 51 conflicts automatically

Result: 167 conflict declarations, 244K-line parser.c.
Parsing still times out.

### Phase 4: Category Rules + prec.left
Added _definition (26 types) and _usage (34 types) category rules.
Replaced 41 self-conflicts with prec.left(0, ...) wrapping.

Result: 190 conflicts, 162 prec.left calls, 401K-line parser.c.
Parsing still times out.

### Phase 5: Prefix Inlining (reverted)
Attempted inlining unified_prefix content into each rule body.
Reduced conflict declarations but created 100+-rule mega-conflicts.
**Made parsing worse, not better.** Reverted.

## Analysis: Why Mechanical Conversion Fails

### The Quantitative Gap
| Metric | Generated | Hand-tuned | Gap |
|--------|-----------|------------|-----|
| Rules | 480 | 222 | 2.2x |
| Conflicts | 190 | 94 | 2.0x |
| prec calls | 162 | 22 | 7.4x |
| Multi-way conflicts (3+) | 84 | ~10 | 8.4x |
| Parse speed | timeout | instant | ∞ |

### The Structural Gap
The quantitative gap understates the problem. The issue is not "too many
conflicts" but "wrong kind of conflicts."

**Generated grammar conflicts** are between rules that share prefixes
because KeBNF factors out prefix keywords into shared rules:
```
OccurrenceDefinitionPrefix = BasicDefinitionPrefix? ('individual' ...)?
PartDefinition = OccurrenceDefinitionPrefix 'part' 'def' ...
ActionDefinition = OccurrenceDefinitionPrefix 'action' 'def' ...
```
When tree-sitter sees `abstract`, it cannot tell if it's the start of a
PartDefinition, ActionDefinition, or any of 26 other definition types
until it sees the keyword (`part`, `action`, etc.) many tokens later.

**Hand-tuned grammar** inlines the prefix into each rule:
```
part_definition: $ => seq(
  repeat($.prefix_metadata),
  optional($.visibility),
  optional('abstract'),
  'part', 'def', ...
)
```
The keyword appears early in each rule. tree-sitter sees `part` and
immediately knows which rule to use. The conflicts are between rules
that genuinely share syntax (definition/usage pairs), not between rules
that share factored-out prefixes.

### The Key Insight
KeBNF's grammar structure serves **metamodel binding** (factored prefixes
map to metamodel properties). tree-sitter's grammar structure serves
**parsing efficiency** (inlined keywords enable early disambiguation).
These are fundamentally different design goals. Mechanical conversion
preserves the metamodel structure but destroys the parsing structure.

## Decision: Pattern-Based Emission

Based on this analysis, we pivot from AST-walking emission to
pattern-based emission for the tree-sitter backend:

1. **Recognize definition/usage patterns** in the KeBNF AST
2. **Emit hand-tuned-style rules** with inlined prefixes and keywords
3. **Drop intermediate rules** (prefix, declaration, body member wrappers)
4. **Use category rules** (_definition, _usage) for body member contexts

This is not "copying the hand-tuned grammar." It is learning the
structural patterns that make tree-sitter grammars efficient and
applying them during emission. The KeBNF AST provides the rule content;
the emission patterns provide the structure.

## Implications for the Paper

This journey demonstrates that:
1. Grammar conversion is not a solved problem -- mechanical approaches fail
2. The failure mode is subtle: generation succeeds but parsing doesn't
3. The fix requires understanding the target parser's disambiguation strategy
4. This understanding can be encoded as emission patterns (deterministic)
   or discovered by an LLM repair agent (the research question)
5. The structural gap (2.2x rules, 8.4x multi-way conflicts) is measurable
   and could serve as a quality metric for grammar conversion tools

## Phase 6: Pattern-Based Emission (attempted, reverted)

Attempted to emit definition/usage rules in the hand-tuned pattern style
(inlined prefix + keyword + body) while keeping the rest of the grammar
using unified_prefix. The two systems conflicted: pattern-based rules
have `optional('ref')` in their prefix, and unified_prefix also contains
`'ref'`. tree-sitter can't tell which system is consuming the token.

**Key finding:** Pattern-based emission requires a COMPLETE rewrite of the
emitter, not a partial overlay. Every rule must handle its own prefix, or
no rule should. Mixing the two approaches creates worse conflicts than
either approach alone.

**The path forward:** A complete pattern-based emitter that:
1. Emits every definition/usage with inlined prefix keywords
2. Emits every non-definition/non-usage rule without unified_prefix
3. Uses _body_member as choice(_definition, _usage, ...other members...)
4. Has no shared prefix rule at all

This is ~4 hours of focused work. The current emitter (which generates
but doesn't parse fast) is the right baseline for CI validation. The
pattern-based rewrite is the next major milestone.

## Phase 7: Pattern-Based Emitter (SUCCEEDED)

Complete rewrite of tree-sitter emitter using pattern-based emission.
Every definition/usage rule has inlined prefix keywords. No shared
prefix rule. All conflicts are 2-way.

**Results:**
- 473 rules, 198 conflicts (all 2-way)
- 176K line parser.c (down from 401K)
- **Parsing works: 0.23ms for a simple file** (was: timeout)
- `part def V;` parses correctly as part_definition

**What made the difference:**
The pattern-based approach eliminates the shared prefix rule entirely.
Each definition/usage rule starts with its own prefix keywords followed
by its disambiguating keyword. tree-sitter sees the keyword early and
knows which rule to use. No mega-conflicts, no exponential GLR exploration.

**Remaining work:**
- `package` keyword not recognized (needs to be in _member or source_file)
- Braces `{ }` not handled in source_file context
- Parse coverage on test corpus not yet measured
- Many rules still have parse errors (body structure, specialization, etc.)

## Phase 8: Parse Quality Iteration (SUCCEEDED)

After the pattern-based emitter breakthrough, iterative fixes to improve
parse quality:

1. Added package/library_package rules to source_file
2. Fixed typing rules (typed_by, typings, subsettings, redefinitions)
   to include the type reference after the keyword
3. Added feature_direction (in/out/inout) to usage prefix
4. Added feature_value to usage pattern for value assignments

**Final metrics:**
- 478 rules, 199 conflicts (all 2-way), 313K line parser.c
- Parse speed: 0.15ms for 35-line file (4290 bytes/ms)
- Corpus coverage: 178/192 test snippets (92.7%)
- Zero errors on 55-line multi-construct test file

**Failing cases (14/192):**
- Shorthand attributes (hand-tuned grammar invention, not in KeBNF)
- Succession with typing keyword
- A few edge cases in actions, calculations, connections, constraints,
  definitions, expressions, flows, metadata, packages, requirements, states

**Comparison to hand-tuned grammar:**
| Metric | Generated | Hand-tuned |
|--------|-----------|------------|
| Rules | 478 | 222 |
| Conflicts | 199 | 94 |
| parser.c | 313K | 478K |
| Parse speed | 0.15ms | ~0.1ms |
| Corpus coverage | 92.7% | 100% |

The generated grammar is now practical for real use. Parse speed is
comparable to the hand-tuned grammar. The 7.3% coverage gap is from
edge cases and hand-tuned grammar inventions.

## Corrected Metrics (Phase 8 continued)

**CORRECTION: Earlier 92.7% claim was based on corpus FILES (15), not
individual test SNIPPETS (192). The actual corpus coverage is 36%.**

**Accurate corpus results (192 test snippets):**
- definitions: 15/20 (75%)
- usages: 22/35 (63%)
- views: 10/24 (42%)
- expressions: 8/23 (35%)
- packages: 5/9 (56%)
- metadata: 3/8 (38%)
- calculations: 2/7 (29%)
- constraints: 1/7 (14%)
- actions: 1/20 (5%)
- states: 1/11 (9%)
- connections: 0/10 (0%)
- flows: 0/5 (0%)
- requirements: 0/11 (0%)
- attributes: 0/1 (0%)
- successions: 0/1 (0%)
- **Total: 68/192 (35.4%)**

**What parses correctly:**
- All 25 definition types with basic structure
- Usages with types, multiplicity, and value assignments
- Packages with nested definitions
- Basic expressions
- Import statements

**What doesn't parse (body-level constructs):**
- Action flow keywords: first, then, if, while, for, loop, send, perform, assign
- Connection keywords: connect, bind, allocate, flow, end
- Constraint keywords: assert, require, assume
- Requirement keywords: satisfy, verify, subject, actor, stakeholder
- State keywords: entry, exit, do, transition, accept
- View keywords: expose, render, filter, frame, satisfy
- Expression features: units [kg], feature chains a.b.c, function calls f(x)
- Documentation: doc, comment about

**Path to 100%:** Each category needs its own body-level rules added to
the pattern-based emitter. This is ~10-15 categories x ~5-10 rules each
= 50-150 additional hardcoded rules. Mechanical but time-consuming.

## Phase 9: Iterative Quality Improvement

**Corpus coverage trajectory:**
- Phase 7 (pattern-based): 36% baseline
- Expression rewrite: 53%
- Generic feature: 72%
- Body statements: 78%
- Batch fixes: 83%

**Final metrics (Phase 9):**
- 192 test snippets, 159 pass (82.8%)
- 2 categories at 100%: attributes, calculations
- 7 categories above 80%: definitions (95%), expressions (96%),
  states (91%), packages (89%), constraints (86%), requirements (82%),
  actions (80%)
- Parse speed: 0.15ms (unchanged)
- Both backends CI-validated

**Remaining 33 failures by category:**
- Actions (4): first action, succession with multiplicity, variation perform, loop
- Connections (4): end with occurrence, n-ary connect, @metadata, allocate ::>
- Views (6): expose/filter/satisfy in view bodies (prec issue)
- Metadata (4): annotated element, about elements, @access, nested blocks
- Usages (6): multiplicity+specialization, assert, snapshot, allocation, event
- Other (9): comment about, assume/require #goal, state parallel, succession typing
