#!/bin/bash

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // .tool_input.path // ""')
CWD=$(echo "$INPUT" | jq -r '.cwd // ""')

# Allow reads within CWD or standard allowed directories
if [[ "$FILE_PATH" == "$CWD"* ]] || \
   [[ "$FILE_PATH" =~ ^\./ ]] || \
   [[ "$FILE_PATH" == ~/Library/Caches* ]] || \
   [[ "$FILE_PATH" == /tmp* ]]; then
    echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"File within allowed directories"}}'
    exit 0
fi

# Fall back to standard permissions for reads outside CWD
exit 0
