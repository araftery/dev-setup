# claude-permissions-hook

A fast Rust binary that serves as a Claude Code PreToolUse hook. It encodes permission rules to auto-allow safe operations, hard-deny dangerous ones, and only prompt for genuinely destructive actions.

## Quick start

```bash
cd /etc/ig/dev-tools/claude-permissions-hook
make deploy
```

This builds the release binary, installs it to `~/.claude/hooks/claude-hook`, and removes the old shell scripts.

## settings.json configuration

Add this to the `hooks.PreToolUse` array in `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/claude-hook bash"
          }
        ]
      },
      {
        "matcher": "Read|Glob|Grep",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/claude-hook read"
          }
        ]
      },
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/claude-hook write"
          }
        ]
      },
      {
        "matcher": "WebFetch|WebSearch",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/claude-hook web"
          }
        ]
      }
    ]
  }
}
```

## Permission rules

### Hard deny: `rm -rf`
Any `rm` command with both `-r` and `-f` flags (in any combination, including `--recursive`) is denied outright.

### Ask: destructive operations
These prompt the user for confirmation:
- `rm` (any form), `mv`, `chmod`, `chown`
- `git rm`, `git rebase`, `git clean`
- `git push --force` / `-f`, `git reset --hard`, `git checkout .`

### Auto-allow: safe read-only & build commands
Safe commands like `ls`, `cat`, `grep`, `git status`, `git log`, `cargo test`, `npm test`, `eslint`, etc. are auto-allowed. See `bash_hook.rs` for the full whitelist.

Package runners (`npx`, `uvx`, `pnpx`, `bunx`) are treated as transparent wrappers — the inner command is evaluated against the safe list.

### Secrets protection
Any access to `.env`, `.env.*`, or `.dev.vars` files is denied across all hooks (Bash, Read, Glob, Grep, Edit, Write).

### Allowed directories (Read/Glob/Grep)
Files within CWD, `/Users/araftery/workspace`, or `/etc/ig` are auto-allowed for reading.

### Web tools
`WebFetch` and `WebSearch` are always auto-allowed.

## Architecture

Single binary dispatched by CLI argument:

```
claude-hook bash   # Bash tool
claude-hook read   # Read|Glob|Grep tools
claude-hook write  # Edit|Write tools
claude-hook web    # WebFetch|WebSearch tools
```

Output format:
```json
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow|deny|ask","permissionDecisionReason":"..."}}
```

Exit 0 with no output = abstain (fall through to Claude's default permissions).

## Known limitations

- **Simple string matching for compound commands**: `echo "rm -rf"` will false-positive as deny because the command is scanned as a flat string, not parsed as shell. This is conservative by design.
- **No subshell/heredoc parsing**: `$(...)`, backticks, and heredocs are scanned as flat strings.
- **cd tracking is basic**: Only tracks `cd <path>` across `&&`/`;` chains, not dynamic paths.

## Development

```bash
cargo test          # Run all 121 tests
cargo build         # Debug build
make deploy         # Test + release build + install
make clean          # Clean build artifacts
make uninstall      # Remove binary from ~/.claude/hooks/
```
