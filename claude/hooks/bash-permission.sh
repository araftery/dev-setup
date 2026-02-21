#!/bin/bash

# Read hook input from stdin
INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // ""')
CWD=$(echo "$INPUT" | jq -r '.cwd // ""')

# Rule 1: Block rm -rf and rm -f
if [[ "$COMMAND" =~ rm[[:space:]]+-[a-zA-Z]*f ]]; then
    echo "Destructive rm -f or rm -rf commands are not allowed" >&2
    exit 2
fi

# Rule 2: Auto-allow safe read-only commands
if [[ "$COMMAND" =~ ^(cat|ls|grep|rg|curl|wget|fetch|find)[[:space:]] ]] || \
   [[ "$COMMAND" =~ ^(cat|ls|grep|rg|curl|wget|fetch|find)$ ]]; then

    # For file system reads (cat, ls, grep, rg), verify path is within CWD
    if [[ "$COMMAND" =~ ^(cat|ls|grep|rg)[[:space:]] ]]; then
        # Extract file path (simplified - adjust for your needs)
        FILE_PATH=$(echo "$COMMAND" | awk '{print $NF}')

        # Allow if path starts with ./ or is within CWD
        if [[ "$FILE_PATH" =~ ^\. ]] || [[ "$FILE_PATH" != /* ]]; then
            echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"Safe read-only command in CWD"}}'
            exit 0
        fi
    else
        # curl, wget, fetch - always allow
        echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"Safe read-only network command"}}'
        exit 0
    fi
fi

# Rule 3: Fall back to standard Claude permissions
exit 0
