#!/usr/bin/env bash

input=$(cat)

model=$(echo "$input" | jq -r '.model.display_name')
used_pct=$(echo "$input" | jq -r '.context_window.used_percentage // empty')
total_cost=$(echo "$input" | jq -r '.total_cost_usd // empty')

# Build progress bar (10 chars wide)
if [ -n "$used_pct" ]; then
  filled=$(echo "$used_pct" | awk '{printf "%d", ($1 / 10 + 0.5)}')
  [ "$filled" -gt 10 ] && filled=10
  empty=$((10 - filled))
  bar=""
  [ "$filled" -gt 0 ] && bar=$(printf '%0.s█' $(seq 1 $filled))
  [ "$empty" -gt 0 ] && bar="${bar}$(printf '%0.s░' $(seq 1 $empty))"
  ctx_display="${bar} ${used_pct}%"
else
  ctx_display="░░░░░░░░░░ --%"
fi

# Dim color for separators
DIM="\033[2m"
RESET="\033[0m"

# Format cost
cost_segment=""
if [ -n "$total_cost" ] && [ "$total_cost" != "null" ] && [ "$total_cost" != "0" ]; then
  cost_segment=" ${DIM}│${RESET} \$$(printf '%.2f' "$total_cost")"
fi

printf "%s ${DIM}│${RESET} %s%s" "$model" "$ctx_display" "$cost_segment"
