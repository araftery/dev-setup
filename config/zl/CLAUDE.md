# zl — Color-coded Zellij session + worktree manager

## Commands

| Command | Purpose |
|---------|---------|
| `zl <project> <wt#> [--dev]` | Open a Zellij session for a project worktree |
| `zwt <project> <wt#> [branch]` | Create a git worktree |
| `zwt -l <project>` | List worktrees for a project |
| `zwt -r <project> <wt#>` | Remove a worktree |
| `zwt cycle <project> <wt#> <branch>` | Remove old worktree, create fresh one from wt1's branch |
| `zl-vscode <project> <wt#>` | Sync VSCode title/status bar colors for one worktree |
| `zl-vscode-all` | Sync all worktrees that exist on disk |
| `zls` | Alias for `zellij list-sessions` |
| `za` | Alias for `zellij attach` |

## Day-to-day workflow

### Starting a session

```bash
zl watchlist 1 --dev    # Singleton dev session (attach if exists, create if not)
zl watchlist 1          # Ephemeral session (new every time, simple 2-pane layout)
```

`--dev` sessions are named `<project>-<wt#>-dev` and use the project's `DEV_LAYOUT` + `DEV_COMMANDS`. Non-dev sessions get a fresh layout named sequentially (`<project>-<wt#>-A`, `-B`, etc.).

Before Zellij starts, OSC escapes set the Ghostty tab name and background color (these don't pass through once Zellij is running).

### Creating a worktree

```bash
zwt intelligems 2 my-feature   # Create wt2 on branch "my-feature"
zwt intelligems 2              # Create wt2 on auto-named branch "intelligems-wt2"
```

### Cycling a worktree (branch rotation)

When you finish a feature branch and want a clean worktree for the next task:

```bash
zwt cycle intelligems 2 next-feature
```

This runs safety checks, then removes the old worktree and creates a fresh one:

1. Resolves the base branch from wt1's current branch (usually `main`)
2. Checks the old worktree is clean (no uncommitted/untracked files)
3. Checks the old branch is fully merged into the base branch
4. Removes the old worktree and deletes the local branch
5. Creates a new worktree on `<new-branch>` branching from the base

If any check fails, it aborts with a message explaining what to fix.

### Syncing VSCode colors

```bash
zl-vscode watchlist 1      # Update .vscode/settings.json for one worktree
zl-vscode-all              # Update all worktrees that exist on disk
```

Merges `workbench.colorCustomizations` (title bar + status bar) into the worktree's `.vscode/settings.json` using `jq`.

## File structure

```
~/.config/zl/
├── CLAUDE.md              # This file
├── zl.sh                  # Main zl() function
├── lib/
│   ├── layout.sh          # Zellij KDL layout generation
│   ├── vscode.sh          # VSCode settings.json color merging
│   └── worktree.sh        # zwt() function — worktree create/remove/cycle
└── projects/
    ├── intelligems.sh     # Per-project config
    ├── cro-agent.sh
    ├── recipes.sh
    └── watchlist.sh
```

Loaded in `.zshrc`:

```bash
source "${HOME}/.config/zl/zl.sh"
source "${HOME}/.config/zl/lib/layout.sh"
source "${HOME}/.config/zl/lib/vscode.sh"
source "${HOME}/.config/zl/lib/worktree.sh"
```

## Layouts

All layouts run `claude` in the left/main pane. Right-side panes are either plain terminals (empty command in `DEV_COMMANDS`) or run a specified command.

**Non-dev**: 50/50 vertical split — `claude` left, terminal right.

**Dev** (controlled by `DEV_LAYOUT`): If `DEV_COMMANDS` is empty, falls back to the non-dev layout (still a singleton session).

| Style | Panes | Description |
|-------|-------|-------------|
| `right-split` | 2 right | 65% claude left, 2 stacked right panes |
| `right-triple` | 3 right | 60% claude left, 3 stacked right panes |
| `bottom-split` | 2 bottom | 70% claude top, 2 side-by-side bottom panes |
| `top-split` | 2 top + 1 bottom | 70% top (65% claude + 35% shell), 30% bottom pane |
| `grid` | 3 other | 2x2 grid, claude top-left |

`DEV_COMMANDS` fills the non-claude panes in order. Empty strings become plain terminal panes.

## Project config format

Each project is a shell file in `~/.config/zl/projects/<name>.sh`:

```bash
PROJECT_NAME="myproject"
SOURCE_REPO="$HOME/workspace/myproject"                    # Main git checkout (for worktree operations)
WT_PATHS=([1]="$HOME/workspace/myproject" [2]="$HOME/workspace/myproject-2")
WT_BG=([1]="#1a2038" [2]="#1e284a")                        # Ghostty tab background (dark, ~#1e1e2e luminance)
WT_VSCODE_ACCENT=([1]="#2a4070" [2]="#345090")             # VSCode title/status bar (brighter)
DEV_LAYOUT="right-split"                                    # right-split | right-triple | bottom-split | grid
DEV_COMMANDS=("" "npm run dev")                            # Commands for right-side panes (empty = terminal)
```

**Key details:**
- `SOURCE_REPO` is the main git checkout used for `git worktree` operations. It can differ from `WT_PATHS[1]` — multiple projects can share one repo (e.g. `intelligems` and `cro-agent` both use `SOURCE_REPO="$HOME/workspace/intelligems"`)
- `WT_BG` colors should be close in luminance to catppuccin-mocha base (`#1e1e2e`) but hue-shifted. wt2 should be noticeably lighter/more saturated than wt1 (~+14 hex units)
- `WT_VSCODE_ACCENT` colors are brighter versions of the same hue
- `DEV_COMMANDS=()` (empty) means dev mode uses the same simple layout as non-dev
- Arrays are zsh associative-style: `([1]=... [2]=...)`

## Color palette

| Project | Hue | wt1 BG | wt2 BG |
|---------|-----|--------|--------|
| intelligems | blue | `#1a2038` | `#1e284a` |
| cro-agent | red | `#2a1a1e` | `#381e28` |
| recipes | green | `#1a2a1e` | `#20381e` |
| watchlist | purple | `#2a1e2a` | `#381e38` |

## Adding a new project

1. Create `~/.config/zl/projects/<name>.sh` with all config fields
2. Pick a hue not used by existing projects
3. Pick `WT_BG` wt1 color near `#1e1e2e` luminance, wt2 noticeably lighter
4. Pick `WT_VSCODE_ACCENT` as brighter versions of the same hues
5. Run `zl-vscode-all` to sync VSCode colors
6. Test: `zl <name> 1 --dev`
