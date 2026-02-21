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

`--dev` sessions are named `<project>-<wt#>-dev` and use the project's `DEV_LAYOUT` + `DEV_COMMANDS`. Non-dev sessions always get a fresh 2-pane layout.

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
3. Checks the old branch is pushed to remote
4. Checks the old branch is fully merged into the base branch
5. Removes the old worktree and deletes the local branch
6. Creates a new worktree on `<new-branch>` branching from the base

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

## Project config format

Each project is a shell file in `~/.config/zl/projects/<name>.sh`:

```bash
PROJECT_NAME="myproject"
SOURCE_REPO="$HOME/workspace/myproject"                    # Main git checkout (for worktree operations)
WT_PATHS=([1]="$HOME/workspace/myproject" [2]="$HOME/workspace/myproject-2")
WT_BG=([1]="#1a1e2e" [2]="#1e2230")                        # Ghostty tab background (dark, ~#1e1e2e luminance)
WT_VSCODE_ACCENT=([1]="#2a3a5c" [2]="#2e4060")             # VSCode title/status bar (slightly brighter)
DEV_LAYOUT="right-split"                                    # right-split | right-triple | bottom-split | grid
DEV_COMMANDS=("npm run dev" "npm run logs")                # Commands for --dev layout panes
```

**Key details:**
- `SOURCE_REPO` is the main git checkout used for `git worktree` operations. It can differ from `WT_PATHS[1]` — multiple projects can share one repo (e.g. `intelligems` and `cro-agent` both use `SOURCE_REPO="$HOME/workspace/intelligems"`)
- `WT_BG` colors should be very close in luminance to catppuccin-mocha base (`#1e1e2e`) but hue-shifted
- `WT_VSCODE_ACCENT` colors are slightly brighter versions of the same hue
- Arrays are zsh associative-style: `([1]=... [2]=...)`

## Dev layouts

| Style | Panes | Description |
|-------|-------|-------------|
| `right-split` | 2 cmd | 65% editor left, 2 stacked right panes |
| `right-triple` | 3 cmd | 60% editor left, 3 stacked right panes |
| `bottom-split` | 2 cmd | 70% editor top, 2 side-by-side bottom panes |
| `grid` | 3 cmd | 2x2 grid, editor top-left |

`DEV_COMMANDS` fills the non-editor panes in order.

## Adding a new project

1. Create `~/.config/zl/projects/<name>.sh` with all config fields
2. Pick `WT_BG` colors close to `#1e1e2e` luminance with a unique hue
3. Pick `WT_VSCODE_ACCENT` as slightly brighter versions
4. Run `zl-vscode-all` to sync VSCode colors
5. Test: `zl <name> 1 --dev`
