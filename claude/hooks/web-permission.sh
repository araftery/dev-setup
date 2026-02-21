#!/bin/bash

# Read hook input from stdin
INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // ""')
URL=$(echo "$INPUT" | jq -r '.tool_input.url // ""')
QUERY=$(echo "$INPUT" | jq -r '.tool_input.query // ""')

# ===== WebSearch Handling =====
if [[ "$TOOL_NAME" == "WebSearch" ]]; then
    # Option 1: Auto-allow all web searches
    echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"WebSearch auto-approved"}}'
    exit 0

    # Option 2: Block sensitive queries (uncomment to use)
    # BLOCKED_KEYWORDS=("password" "secret" "credential" "hack")
    # for keyword in "${BLOCKED_KEYWORDS[@]}"; do
    #     if [[ "$QUERY" =~ $keyword ]]; then
    #         echo "Search query contains blocked keyword: $keyword" >&2
    #         exit 2
    #     fi
    # done
fi

# ===== WebFetch Handling =====
if [[ "$TOOL_NAME" == "WebFetch" ]]; then
  echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"WebFtech auto-approved"}}'
  exit 0
fi

# Fall back to standard Claude permissions
exit 0
