# Claude Permissions Hook

## What this project is

A Rust binary (`claude-hook`) that serves as a Claude Code PreToolUse hook. It replaces the old bash scripts (`bash-permission.sh`, `read-permission.sh`, `web-permission.sh`) with a single compiled binary for speed and correctness.

Installed to `~/.claude/hooks/claude-hook`. Called by Claude Code via `settings.json` hooks.

## Project structure

```
src/
├── main.rs          # stdin JSON parsing, CLI dispatch (bash|read|write|web)
├── types.rs         # HookInput, HookOutput, Decision enum
├── paths.rs         # Path normalization, allowed-dir checks, secrets detection
├── bash_hook.rs     # Bash tool: rm -rf deny, destructive ask, safe command allow
├── read_hook.rs     # Read/Glob/Grep: secrets deny, allowed-dir allow
├── write_hook.rs    # Edit/Write: secrets deny, else abstain
└── web_hook.rs      # WebFetch/WebSearch: always allow
```

## Key design decisions

- **Decision enum**: `Allow`, `Deny`, `Ask`, `Abstain`. Abstain = no output, exit 0 = fall through to Claude's defaults.
- **Compound command splitting**: Splits on `&&`, `||`, `;`, `|`. Not a full shell parser — conservative by design.
- **cd tracking**: Maintains `current_dir` across `&&`/`;` segments to resolve relative paths.
- **npx/uvx unwrapping**: Strips the runner and its flags, evaluates the inner command against the safe list.
- **Path normalization**: Manual component-based resolution (no `canonicalize`) for predictability.
- **Secrets files**: `.env`, `.env.*`, `.dev.vars` — checked by basename.

## How to modify permission rules

- **Add a safe command**: Add to `SAFE_COMMANDS` in `bash_hook.rs`
- **Add a safe build tool**: Add to `SAFE_BUILD_COMMANDS` in `bash_hook.rs`
- **Add an allowed directory**: Add to `ALLOWED_DIRS` in `paths.rs`
- **Add a destructive command**: Add a check in `is_destructive()` in `bash_hook.rs`

## Build & deploy

```bash
make deploy   # cargo test && cargo build --release && install to ~/.claude/hooks/
```

Dependencies: `serde`, `serde_json` only.

## Testing

121 tests cover all hooks. Run with `cargo test`. Tests use in-module `#[cfg(test)]` blocks — no separate test files.

## Important: settings.json hook format

The binary is called with a single argument that selects the hook type. The settings.json `hooks.PreToolUse` array must have entries with matchers for `Bash`, `Read|Glob|Grep`, `Edit|Write`, and `WebFetch|WebSearch`, each calling `~/.claude/hooks/claude-hook <type>`.
