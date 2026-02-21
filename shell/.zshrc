eval "$(/opt/homebrew/bin/brew shellenv)"
export PATH="/Applications/Sublime Text.app/Contents/SharedSupport/bin:$PATH"

eval "$(starship init zsh)"
eval "$(direnv hook zsh)"
. $HOMEBREW_PREFIX/etc/profile.d/z.sh
. "$HOME/.local/bin/env"
. "$HOME/.cargo/env"
export PATH="/opt/homebrew/bin:$PATH" >> ~/.zshrc

autoload -U up-line-or-beginning-search
autoload -U down-line-or-beginning-search
zle -N up-line-or-beginning-search
zle -N down-line-or-beginning-search
bindkey "^[[A" up-line-or-beginning-search # Up
bindkey "^[[B" down-line-or-beginning-search # Down

export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion

# proto
export PROTO_HOME="$HOME/.proto";
export PATH="$PROTO_HOME/shims:$PROTO_HOME/bin:$PATH";alias uuid='f() { for i in $(seq ${1:-1}); do uuidgen | tr "[:upper:]" "[:lower:]"; done }; f'
alias timestamp="python -c 'import time; print(int(time.time() * 1000))'"
alias sublime="/Applications/Sublime\ Text.app/Contents/SharedSupport/bin/subl"
export NODE_OPTIONS=--max-old-space-size=16384
export FORCE_COLOR=1
# The following lines have been added by Docker Desktop to enable Docker CLI completions.
fpath=(/Users/araftery/.docker/completions $fpath)
autoload -Uz compinit
compinit
# End of Docker CLI completions

alias nb="cd /etc/ig/intelli-notebooks && uv run jupyter notebook"
alias uuid='f() { for i in $(seq ${1:-1}); do uuidgen | tr "[:upper:]" "[:lower:]"; done }; f'

export PATH="$(npm config get prefix)/bin:$PATH"


# pnpm
export PNPM_HOME="/Users/araftery/Library/pnpm"
case ":$PATH:" in
  *":$PNPM_HOME:"*) ;;
  *) export PATH="$PNPM_HOME:$PATH" ;;
esac
# pnpm end

# The next line updates PATH for the Google Cloud SDK.
if [ -f '/Users/araftery/Downloads/google-cloud-sdk/path.zsh.inc' ]; then . '/Users/araftery/Downloads/google-cloud-sdk/path.zsh.inc'; fi

# The next line enables shell command completion for gcloud.
if [ -f '/Users/araftery/Downloads/google-cloud-sdk/completion.zsh.inc' ]; then . '/Users/araftery/Downloads/google-cloud-sdk/completion.zsh.inc'; fi

js() { pbpaste | jless; }

# Added by Antigravity
export PATH="/Users/araftery/.antigravity/antigravity/bin:$PATH"
export SST_WORKER_POOL_SIZE=20
export SST_WORKER_IDLE_TIMEOUT=90000
export SST_BUILD_CONCURRENCY=8

# Entire CLI shell completion
autoload -Uz compinit && compinit && source <(entire completion zsh)

# peon-ping quick controls
alias peon="bash ~/.claude/hooks/peon-ping/peon.sh"
[ -f ~/.claude/hooks/peon-ping/completions.bash ] && source ~/.claude/hooks/peon-ping/completions.bash

# Cortex CLI completion (disable via /settings in cortex)
[[ -s ~/.zsh/completions/cortex.zsh ]] && source ~/.zsh/completions/cortex.zsh


# === zl session manager ===
source "${HOME}/.config/zl/zl.sh"
source "${HOME}/.config/zl/lib/layout.sh"
source "${HOME}/.config/zl/lib/vscode.sh"
source "${HOME}/.config/zl/lib/worktree.sh"
alias zls="zellij list-sessions"
alias za="zellij attach"