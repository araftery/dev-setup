---
name: zl
description: Manage Zellij sessions and worktrees. Use when the user asks about sessions, worktrees, project configs, or wants to run zl/zwt commands.
user_invocable: true
---

# zl — Zellij session & worktree manager

Help the user manage their Zellij sessions and git worktrees using the `zl` system.

## Reference

Read `~/.config/zl/CLAUDE.md` for the full reference (commands, config format, workflow).

## What you can do

### List projects and their config

Read project configs from `~/.config/zl/projects/*.sh` to answer questions about the current setup (paths, colors, layouts, commands).

### Run commands

Use Bash to run:

- `zl <project> <wt#> [--dev]` — start a session (note: this launches Zellij interactively, so warn the user it will take over the terminal)
- `zwt <project> <wt#> [branch]` — create a worktree
- `zwt -l <project>` — list worktrees
- `zwt -r <project> <wt#>` — remove a worktree
- `zwt cycle <project> <wt#> <branch>` — cycle a worktree (remove old, create fresh)
- `zl-vscode <project> <wt#>` — sync VSCode colors
- `zl-vscode-all` — sync all VSCode colors

Important: these are shell functions, not scripts. They must be run in a login shell. Use `zsh -ic '<command>'` to invoke them from Bash tool.

### Help add/edit projects

When creating a new project config:

1. Ask the user for the project name, repo path, and number of worktrees
2. Pick `WT_BG` colors near `#1e1e2e` luminance with a unique hue (check existing configs to avoid duplicates)
3. Pick `WT_VSCODE_ACCENT` as slightly brighter versions of the same hue
4. Write the config to `~/.config/zl/projects/<name>.sh`
5. Run `zsh -ic 'zl-vscode-all'` to sync VSCode colors

### Guide worktree cycling

When the user wants to cycle a worktree:

1. Confirm the project, worktree number, and new branch name
2. Run `zsh -ic 'zwt cycle <project> <wt#> <branch>'`
3. If it aborts, explain the safety check that failed and what to fix
4. On success, suggest `zl <project> <wt#> --dev` to start a session

## If the user just says `/zl` with no further context

List all configured projects by reading `~/.config/zl/projects/*.sh` and display a summary table showing: project name, source repo, worktree paths, dev layout, and dev commands.
