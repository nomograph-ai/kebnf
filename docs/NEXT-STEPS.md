# Next Steps: Hybrid Approach Implementation

## Current State (2026-02-10)

### What Works
- KEBNF parser: 640/640 rules parsed successfully
- Empty-string rules: 23 rules replaced with non-empty alternatives
- Ambiguity registry: All resolutions documented for public review
- Mapping output: Full semantic traceability preserved

### What Doesn't Work
- **335+ LR conflicts** when generating tree-sitter parser
- Direct KEBNF → tree-sitter conversion produces unusable grammar

### Root Cause
KEBNF is designed for **metamodel binding**, not **LR parsing**:
- Context-sensitive bodies (different members valid in different contexts)
- Overlapping expression patterns
- Syntactically identical rules with different semantic types

## Proposed Solution: Hybrid Approach

### Rationale
| Approach | Conflicts | Spec Fidelity | Maintenance |
|----------|-----------|---------------|-------------|
| Pure spec-driven | 335+ | High | Re-run converter |
| Restructure for LR | ~28 | Low | Manual updates |
| **Hybrid** | ~50-100 | Medium | Mixed |

The hybrid approach balances traceability with practicality.

### Implementation Plan

#### Phase 1: Identify High-Conflict Patterns
Analyze the 335 conflicts to find patterns:
1. **Context-sensitive bodies** (~100+ conflicts)
   - `NamespaceBodyElement` vs `PackageBodyElement` vs `DefinitionBodyItem`
   - Brute-force solution: Single `_usage_member` rule
   
2. **Expression ambiguities** (~50+ conflicts)
   - Feature chains vs qualified names
   - Brute-force solution: Precedence declarations
   
3. **Metadata/prefix patterns** (~50+ conflicts)
   - `prefix_metadata_member` appears in many contexts
   - Brute-force solution: Structural refactoring

#### Phase 2: Apply Brute-Force Patterns
For each high-conflict pattern:
1. Study how brute-force grammar resolves it
2. Apply same pattern to spec-driven grammar
3. Document deviation in `AMBIGUITY-RESOLUTIONS.md`
4. Update mapping to track semantic information lost

#### Phase 3: Validate
1. Run `tree-sitter generate` - should succeed with ~50-100 conflicts
2. Test against training files - compare coverage to brute-force
3. Document any parsing differences

### Key Files

| File | Purpose |
|------|---------|
| `src/emitter.rs` | Grammar generation logic |
| `docs/AMBIGUITY-RESOLUTIONS.md` | Resolution registry |
| `docs/ambiguity-resolutions.json` | Machine-readable resolutions |
| `tests/output/combined_grammar.js` | Generated grammar |
| `tests/output/combined_mapping.json` | Semantic mapping |

### Reference: Brute-Force Patterns

Location: `gitlab.com/nomograph/tree-sitter-sysml` (grammar.js)

Key patterns to study:
```javascript
// Single body member rule (lines ~200-250)
_usage_member: ($) => choice(
  $._usage,
  $.connect_statement,
  // ... all member types unified
)

// Expression precedence (lines ~800-900)
_expression: ($) => choice(
  prec.left(1, $.conditional_expression),
  prec.left(2, $.null_coalescing_expression),
  // ... precedence tower
)

// Conflicts array (lines ~15-45)
conflicts: $ => [
  // Only 28 conflicts needed
]
```

### Success Criteria

1. `tree-sitter generate` succeeds
2. Conflict count: 50-100 (not 335+)
3. Training file coverage: ≥95% (brute-force achieves 100%)
4. All deviations documented in resolution registry
5. Mapping preserves semantic traceability

## Session Resumption

To continue this work:

```bash
cd kebnf-to-tree-sitter

# Check current state
cargo run -- tests/kebnf/KerML-textual-bnf.kebnf tests/kebnf/SysML-textual-bnf.kebnf \
  -o tests/output/combined_grammar.js --stats

# Test tree-sitter generation
cp tests/output/combined_grammar.js /tmp/ts-test/grammar.js
cd /tmp/ts-test && npx tree-sitter generate

# Compare with brute-force
diff <(grep "conflicts:" grammar.js -A50) \
     <(grep "conflicts:" path/to/tree-sitter-sysml/grammar.js -A50)
```

## Related Documents

- `README.md` - Project context and design philosophy
- `docs/KEBNF-SPEC.md` - Complete KEBNF syntax reference
- `docs/CONFLICT-PATTERNS.md` - Conflict resolution patterns
- `docs/AMBIGUITY-RESOLUTIONS.md` - Resolution registry
