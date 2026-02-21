#!/bin/bash
# Backup personal dev setup files to this git repo
# Run periodically, then commit & push to keep a remote backup

set -e

BACKUP_DIR="$(cd "$(dirname "$0")" && pwd)"
HOME_DIR="$HOME"

echo "Backing up dev setup to $BACKUP_DIR ..."

# --- Shell configs ---
mkdir -p "$BACKUP_DIR/shell"
for f in .zshrc .zprofile .zshenv; do
  [ -f "$HOME_DIR/$f" ] && cp "$HOME_DIR/$f" "$BACKUP_DIR/shell/$f"
done

# Zsh completions
if [ -d "$HOME_DIR/.zsh" ]; then
  mkdir -p "$BACKUP_DIR/shell/.zsh"
  cp -R "$HOME_DIR/.zsh/." "$BACKUP_DIR/shell/.zsh/"
fi

# --- Git config ---
mkdir -p "$BACKUP_DIR/git"
[ -f "$HOME_DIR/.gitconfig" ] && cp "$HOME_DIR/.gitconfig" "$BACKUP_DIR/git/.gitconfig"
[ -f "$HOME_DIR/.gitignore_global" ] && cp "$HOME_DIR/.gitignore_global" "$BACKUP_DIR/git/.gitignore_global"

# --- Claude Code ---
CLAUDE_DIR="$HOME_DIR/.claude"
mkdir -p "$BACKUP_DIR/claude/hooks"
mkdir -p "$BACKUP_DIR/claude/skills"
mkdir -p "$BACKUP_DIR/claude/projects"

# Top-level config files
for f in CLAUDE.md settings.json statusline-command.sh keybindings.json; do
  [ -f "$CLAUDE_DIR/$f" ] && cp "$CLAUDE_DIR/$f" "$BACKUP_DIR/claude/$f"
done

# Hooks
if [ -d "$CLAUDE_DIR/hooks" ]; then
  cp -R "$CLAUDE_DIR/hooks/." "$BACKUP_DIR/claude/hooks/"
fi

# Skills
if [ -d "$CLAUDE_DIR/skills" ]; then
  cp -R "$CLAUDE_DIR/skills/." "$BACKUP_DIR/claude/skills/"
fi

# Per-project CLAUDE.md and settings files (skip large caches/logs)
if [ -d "$CLAUDE_DIR/projects" ]; then
  for proj_dir in "$CLAUDE_DIR/projects"/*/; do
    proj_name="$(basename "$proj_dir")"
    mkdir -p "$BACKUP_DIR/claude/projects/$proj_name"
    for f in CLAUDE.md settings.json settings.local.json; do
      [ -f "$proj_dir/$f" ] && cp "$proj_dir/$f" "$BACKUP_DIR/claude/projects/$proj_name/$f"
    done
  done
fi

# Plugins config (not caches)
mkdir -p "$BACKUP_DIR/claude/plugins"
for f in config.json installed_plugins.json blocklist.json; do
  [ -f "$CLAUDE_DIR/plugins/$f" ] && cp "$CLAUDE_DIR/plugins/$f" "$BACKUP_DIR/claude/plugins/$f"
done

# --- ~/.config selections ---
CONFIG_DIR="$HOME_DIR/.config"

# starship
[ -f "$CONFIG_DIR/starship.toml" ] && cp "$CONFIG_DIR/starship.toml" "$BACKUP_DIR/config-starship.toml"

# Directory-based configs
for dir in zl zs zellij iterm2 ghostty; do
  if [ -d "$CONFIG_DIR/$dir" ]; then
    mkdir -p "$BACKUP_DIR/config/$dir"
    cp -R "$CONFIG_DIR/$dir/." "$BACKUP_DIR/config/$dir/"
  fi
done

# --- Ghostty (macOS Application Support location) ---
GHOSTTY_APP_SUPPORT="$HOME_DIR/Library/Application Support/com.mitchellh.ghostty"
if [ -d "$GHOSTTY_APP_SUPPORT" ]; then
  mkdir -p "$BACKUP_DIR/config/ghostty"
  cp -R "$GHOSTTY_APP_SUPPORT/." "$BACKUP_DIR/config/ghostty/"
fi

echo "Done. Files copied to $BACKUP_DIR"

# If --push flag is passed, commit and push
if [ "${1:-}" = "--push" ]; then
  cd "$BACKUP_DIR"
  git add -A
  if git diff --cached --quiet; then
    echo "No changes to commit."
  else
    git commit -m "backup $(date +%Y-%m-%d)"
    git push
    echo "Committed and pushed."
  fi
else
  echo ""
  echo "To commit and push:"
  echo "  cd $BACKUP_DIR && git add -A && git commit -m 'backup $(date +%Y-%m-%d)' && git push"
  echo ""
  echo "Or run: $0 --push"
fi
