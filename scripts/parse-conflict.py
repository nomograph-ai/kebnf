#!/usr/bin/env python3
"""Parse tree-sitter generate conflict diagnostics into structured data.

Usage:
    tree-sitter generate 2>&1 | python3 parse-conflict.py

Output: JSON with conflict details, or empty object if no conflict.
"""

import sys
import re
import json


def parse_conflict(text: str) -> dict | None:
    """Parse a tree-sitter conflict diagnostic into structured data."""

    if "Unresolved conflict" not in text:
        return None

    result = {
        "raw": text.strip(),
        "symbol_sequence": "",
        "interpretations": [],
        "suggested_resolutions": [],
        "rule_names": set(),
    }

    lines = text.strip().split("\n")

    for i, line in enumerate(lines):
        stripped = line.strip()

        # Symbol sequence line (contains the bullet •)
        if "•" in stripped and not stripped.startswith(("1:", "2:", "3:")):
            result["symbol_sequence"] = stripped

        # Interpretation lines
        if re.match(r"^\d+:", stripped):
            result["interpretations"].append(stripped)
            # Extract rule names from parenthesized groups
            for match in re.finditer(r"\((\w+)", stripped):
                result["rule_names"].add(match.group(1))
            # Extract rule names from bare identifiers
            for match in re.finditer(r"\b([a-z_]\w+)\b", stripped):
                result["rule_names"].add(match.group(1))

        # Resolution suggestions
        if stripped.startswith(("Specify a", "Add a conflict")):
            result["suggested_resolutions"].append(stripped)
            # Extract rule names from backtick-quoted names
            for match in re.finditer(r"`(\w+)`", stripped):
                result["rule_names"].add(match.group(1))

    # Clean up rule names -- remove common noise words
    noise = {"for", "in", "or", "and", "the", "these", "rules", "left", "right"}
    result["rule_names"] = sorted(result["rule_names"] - noise)

    return result


def extract_rule(grammar_text: str, rule_name: str) -> str | None:
    """Extract a single rule definition from grammar.js."""
    # Match: rule_name: $ => ... (handles multi-line rules)
    pattern = rf"^\s+{re.escape(rule_name)}:\s*\$\s*=>"
    lines = grammar_text.split("\n")

    start = None
    depth = 0
    for i, line in enumerate(lines):
        if start is None:
            if re.match(pattern, line):
                start = i
                depth = line.count("(") - line.count(")")
        else:
            depth += line.count("(") - line.count(")")
            if depth <= 0 and line.strip().endswith(","):
                return "\n".join(lines[start : i + 1])

    if start is not None:
        # Fallback: return next 20 lines
        return "\n".join(lines[start : start + 20])

    return None


def main():
    text = sys.stdin.read()

    conflict = parse_conflict(text)
    if conflict is None:
        if "Error" not in text:
            # No error at all -- tree-sitter generate succeeded
            print(json.dumps({"success": True}))
        else:
            print(json.dumps({"success": False, "error": text.strip()}))
        return

    # If grammar.js path provided as arg, extract relevant rules
    grammar_text = None
    reference_text = None
    if len(sys.argv) > 1:
        with open(sys.argv[1]) as f:
            grammar_text = f.read()
    if len(sys.argv) > 2:
        with open(sys.argv[2]) as f:
            reference_text = f.read()

    output = {
        "success": False,
        "conflict": {
            "symbol_sequence": conflict["symbol_sequence"],
            "interpretations": conflict["interpretations"],
            "suggested_resolutions": conflict["suggested_resolutions"],
            "rule_names": conflict["rule_names"],
        },
    }

    if grammar_text:
        output["rules"] = {}
        for name in conflict["rule_names"]:
            rule = extract_rule(grammar_text, name)
            if rule:
                output["rules"][name] = rule

    if reference_text:
        output["reference_rules"] = {}
        for name in conflict["rule_names"]:
            # Try snake_case version too
            rule = extract_rule(reference_text, name)
            if rule:
                output["reference_rules"][name] = rule

    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
