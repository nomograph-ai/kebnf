#!/usr/bin/env bash
# repair-loop.sh — LLM-driven grammar repair loop for tree-sitter
#
# Usage:
#   ./scripts/repair-loop.sh [--step|--auto|--dry-run] [--max-iterations N]
#
# Requires: tree-sitter CLI, python3, jq
# The LLM integration is a placeholder — replace call_llm() with your
# preferred API (OpenAI, Anthropic, GitLab Duo, etc.)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
WORK_DIR="${WORK_DIR:-/tmp/ts-repair}"
REFERENCE_GRAMMAR="${REFERENCE_GRAMMAR:-$HOME/gitlab.com/nomograph/tree-sitter-sysml/grammar.js}"
PROMPT_FILE="$REPO_DIR/prompts/repair-agent.md"
MAX_ITERATIONS="${MAX_ITERATIONS:-100}"
MODE="${1:---step}"  # --step, --auto, --dry-run

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[repair]${NC} $*"; }
ok()  { echo -e "${GREEN}[  ok  ]${NC} $*"; }
err() { echo -e "${RED}[error ]${NC} $*"; }
warn(){ echo -e "${YELLOW}[ warn ]${NC} $*"; }

# --- Setup ---

setup_workspace() {
    log "Setting up workspace at $WORK_DIR"
    mkdir -p "$WORK_DIR"

    # Generate fresh grammar.js from kebnf
    log "Generating grammar.js from KeBNF specs..."
    (cd "$REPO_DIR" && cargo run --release -- \
        tests/kebnf/KerML-textual-bnf.kebnf \
        tests/kebnf/SysML-textual-bnf.kebnf \
        --format tree-sitter \
        -o "$WORK_DIR/grammar.js" 2>/dev/null)

    # tree-sitter config
    cat > "$WORK_DIR/tree-sitter.json" << 'EOF'
{
  "metadata": {"version": "0.0.1", "links": {}},
  "grammars": [{"name": "sysml", "path": "."}]
}
EOF
    cat > "$WORK_DIR/package.json" << 'EOF'
{"name": "tree-sitter-sysml-repair", "version": "0.0.1"}
EOF

    # Initialize repair log
    echo "[]" > "$WORK_DIR/repair-log.json"
    ok "Workspace ready"
}

# --- Core loop ---

try_generate() {
    cd "$WORK_DIR"
    tree-sitter generate 2>&1
}

parse_conflict() {
    python3 "$SCRIPT_DIR/parse-conflict.py" \
        "$WORK_DIR/grammar.js" \
        "$REFERENCE_GRAMMAR" \
        2>/dev/null
}

build_llm_prompt() {
    local conflict_json="$1"
    local iteration="$2"
    local history

    history=$(cat "$WORK_DIR/repair-log.json")

    cat << PROMPT_EOF
$(cat "$PROMPT_FILE")

## Current State

Iteration: $iteration
Repairs so far: $(echo "$history" | jq length)

## Conflict Diagnostic

$(echo "$conflict_json" | jq -r '.conflict.symbol_sequence // "none"')

Interpretations:
$(echo "$conflict_json" | jq -r '.conflict.interpretations[]? // "none"')

Suggested resolutions:
$(echo "$conflict_json" | jq -r '.conflict.suggested_resolutions[]? // "none"')

## Conflicting Rules (from generated grammar)

$(echo "$conflict_json" | jq -r '.rules // {} | to_entries[] | "### \(.key)\n```javascript\n\(.value)\n```\n"')

## Reference Rules (from hand-tuned grammar)

$(echo "$conflict_json" | jq -r '.reference_rules // {} | to_entries[] | "### \(.key)\n```javascript\n\(.value)\n```\n"')

## Repair History (last 5)

$(echo "$history" | jq -r '.[-5:][] | "- [\(.strategy)] \(.reasoning)"')

Respond with a JSON object only. No markdown fences, no explanation outside the JSON.
PROMPT_EOF
}

call_llm() {
    # PLACEHOLDER: Replace with your LLM API call.
    # Input: prompt on stdin
    # Output: JSON repair instruction on stdout
    #
    # Example with curl + Anthropic API:
    # curl -s https://api.anthropic.com/v1/messages \
    #   -H "x-api-key: $ANTHROPIC_API_KEY" \
    #   -H "content-type: application/json" \
    #   -d "$(jq -n --arg prompt "$(cat)" '{
    #     model: "claude-sonnet-4-20250514",
    #     max_tokens: 1024,
    #     messages: [{role: "user", content: $prompt}]
    #   }')" | jq -r '.content[0].text'

    err "LLM integration not configured. Set up call_llm() in repair-loop.sh"
    err "Prompt saved to $WORK_DIR/last-prompt.txt"
    cat > "$WORK_DIR/last-prompt.txt"
    return 1
}

apply_repair() {
    local repair_json="$1"
    local target_rule old_text new_text edit_type

    edit_type=$(echo "$repair_json" | jq -r '.edit.type')
    target_rule=$(echo "$repair_json" | jq -r '.edit.target_rule')
    old_text=$(echo "$repair_json" | jq -r '.edit.old_text')
    new_text=$(echo "$repair_json" | jq -r '.edit.new_text')

    case "$edit_type" in
        replace)
            if ! grep -qF "$old_text" "$WORK_DIR/grammar.js"; then
                err "old_text not found in grammar.js for rule $target_rule"
                return 1
            fi
            # Use python for safe string replacement
            python3 -c "
import sys
with open('$WORK_DIR/grammar.js') as f:
    content = f.read()
old = '''$old_text'''
new = '''$new_text'''
if old not in content:
    sys.exit(1)
content = content.replace(old, new, 1)
with open('$WORK_DIR/grammar.js', 'w') as f:
    f.write(content)
"
            ;;
        *)
            err "Unsupported edit type: $edit_type"
            return 1
            ;;
    esac

    ok "Applied $edit_type to $target_rule"
}

record_repair() {
    local iteration="$1"
    local repair_json="$2"
    local result="$3"

    local entry
    entry=$(echo "$repair_json" | jq --arg i "$iteration" --arg r "$result" '{
        iteration: ($i | tonumber),
        strategy: .strategy,
        reasoning: .reasoning,
        target_rule: .edit.target_rule,
        result: $r
    }')

    local log
    log=$(cat "$WORK_DIR/repair-log.json")
    echo "$log" | jq ". + [$entry]" > "$WORK_DIR/repair-log.json"
}

# --- Main ---

main() {
    setup_workspace

    for i in $(seq 1 "$MAX_ITERATIONS"); do
        log "=== Iteration $i ==="

        # Try to generate
        local output
        output=$(try_generate 2>&1 || true)

        # Parse the result
        local conflict_json
        conflict_json=$(echo "$output" | parse_conflict)

        # Check if we succeeded
        if echo "$conflict_json" | jq -e '.success == true' > /dev/null 2>&1; then
            ok "tree-sitter generate SUCCEEDED after $((i - 1)) repairs!"
            ok "Repaired grammar at: $WORK_DIR/grammar.js"
            ok "Repair log at: $WORK_DIR/repair-log.json"
            return 0
        fi

        # Show the conflict
        log "Conflict: $(echo "$conflict_json" | jq -r '.conflict.symbol_sequence')"
        log "Rules: $(echo "$conflict_json" | jq -r '.conflict.rule_names | join(", ")')"

        if [ "$MODE" = "--dry-run" ]; then
            local prompt
            prompt=$(build_llm_prompt "$conflict_json" "$i")
            echo "$prompt" > "$WORK_DIR/prompt-$i.txt"
            log "Prompt saved to $WORK_DIR/prompt-$i.txt"
            continue
        fi

        # Build prompt and call LLM
        local prompt repair_json
        prompt=$(build_llm_prompt "$conflict_json" "$i")

        if ! repair_json=$(echo "$prompt" | call_llm); then
            warn "LLM call failed at iteration $i"
            if [ "$MODE" = "--step" ]; then
                log "Fix manually, then press Enter to continue (or Ctrl-C to stop)"
                read -r
                continue
            else
                return 1
            fi
        fi

        # Validate the repair JSON
        if ! echo "$repair_json" | jq -e '.edit.old_text' > /dev/null 2>&1; then
            err "Invalid repair JSON from LLM"
            echo "$repair_json" > "$WORK_DIR/bad-response-$i.json"
            continue
        fi

        log "Strategy: $(echo "$repair_json" | jq -r '.strategy')"
        log "Reasoning: $(echo "$repair_json" | jq -r '.reasoning')"

        if [ "$MODE" = "--step" ]; then
            echo "$repair_json" | jq '.edit'
            log "Apply this fix? [y/N/e(dit)]"
            read -r answer
            case "$answer" in
                y|Y) ;;
                e|E)
                    log "Edit $WORK_DIR/grammar.js manually, then press Enter"
                    read -r
                    record_repair "$i" "$repair_json" "manual_edit"
                    continue
                    ;;
                *)
                    log "Skipped"
                    record_repair "$i" "$repair_json" "skipped"
                    continue
                    ;;
            esac
        fi

        # Apply the repair
        if apply_repair "$repair_json"; then
            record_repair "$i" "$repair_json" "applied"
        else
            record_repair "$i" "$repair_json" "failed"
            warn "Repair failed to apply"
        fi
    done

    err "Hit max iterations ($MAX_ITERATIONS) without resolving all conflicts"
    return 1
}

main "$@"
