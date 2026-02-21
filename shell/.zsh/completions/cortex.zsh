# cortex-cli-completion-version: 1.0.12+183057.950e63f0ed1c
# Cortex CLI completion setup - sourced from .zshrc
fpath+=~/.zsh/completions
if (( ${+_comps} )); then
  # compinit already ran - register directly
  autoload -Uz _cortex 2>/dev/null && _comps[cortex]=_cortex
elif (( ${+functions[compdef]} )); then
  # compdef exists but compinit hasn't run yet - queue registration
  autoload -Uz _cortex 2>/dev/null && compdef _cortex cortex
fi
