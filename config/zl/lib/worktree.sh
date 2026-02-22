# Git worktree management for zl projects
#
# Usage:
#   zwt <project> <wt#> [branch]          Create a new worktree
#   zwt -l <project>                      List worktrees for a project
#   zwt -r <project> <wt#>               Remove a worktree
#   zwt cycle <project> <wt#> <branch>   Remove old worktree, create fresh one from origin/main
#
# SOURCE_REPO in the project config points to the main git checkout.
# Multiple projects can share the same source repo (e.g. intelligems
# and cro-agent both use the intelligems repo).

# Copy node_modules from source worktree using CoW clones, then run pnpm install
_zwt_copy_node_modules() {
    local src="$1" dst="$2"
    local count=0
    while IFS= read -r rel; do
        local dir="${rel%/*}"
        [[ "$dir" != "$rel" ]] && mkdir -p "${dst}/${dir}"
        cp -Rc "${src}/${rel}" "${dst}/${rel}"
        ((count++))
    done < <(cd "$src" && find . -name "node_modules" -maxdepth 3 -not -path "*/node_modules/*/node_modules/*" -not -path "*/.git/*" | sed 's|^\./||')
    if [[ $count -gt 0 ]]; then
        echo "Cloned ${count} node_modules dir(s) from ${src} (CoW)"
        echo "Running pnpm install to reconcile..."
        (DIRENV_DIFF="" DIRENV_WATCHES="" HUSKY=0 cd "$dst" && pnpm install --frozen-lockfile 2>&1 | tail -3)
    fi
}

# Copy .env* files from source worktree to new worktree, preserving directory structure
_zwt_copy_env_files() {
    local src="$1" dst="$2"
    local count=0
    while IFS= read -r rel; do
        local dir="${rel%/*}"
        [[ "$dir" != "$rel" ]] && mkdir -p "${dst}/${dir}"
        cp "${src}/${rel}" "${dst}/${rel}"
        ((count++))
    done < <(cd "$src" && find . -name ".env*" -not -path "*/node_modules/*" -not -path "*/.git/*" | sed 's|^\./||')
    if [[ $count -gt 0 ]]; then
        echo "Copied ${count} .env file(s) from ${src}"
    fi
}

zwt() {
    # Parse leading flags / subcommands
    local mode="create"
    case "$1" in
        -l|--list)   mode="list";   shift ;;
        -r|--remove) mode="remove"; shift ;;
        cycle)       mode="cycle";  shift ;;
        -h|--help)
            echo "Usage:"
            echo "  zwt <project> <wt#> [branch]          Create worktree"
            echo "  zwt -l <project>                      List worktrees"
            echo "  zwt -r <project> <wt#>               Remove worktree"
            echo "  zwt cycle <project> <wt#> <branch>   Cycle: remove old, create fresh from origin/main"
            return 0
            ;;
    esac

    local project="$1"
    if [[ -z "$project" ]]; then
        echo "zwt: project name required"
        echo "Available: $(ls "${HOME}/.config/zl/projects/" | sed 's/\.sh$//' | tr '\n' ' ')"
        return 1
    fi

    local config_file="${HOME}/.config/zl/projects/${project}.sh"
    if [[ ! -f "$config_file" ]]; then
        echo "zwt: unknown project '$project'"
        return 1
    fi

    local WT_PATHS SOURCE_REPO
    source "$config_file"

    local source_path="${SOURCE_REPO}"
    if [[ -z "$source_path" ]]; then
        echo "zwt: SOURCE_REPO not set in ${config_file}"
        return 1
    fi
    if [[ ! -d "$source_path/.git" && ! -f "$source_path/.git" ]]; then
        echo "zwt: source ${source_path} is not a git repo"
        return 1
    fi

    case "$mode" in
        list)
            git -C "$source_path" worktree list
            ;;
        remove)
            local wt="$2"
            if [[ -z "$wt" ]]; then
                echo "zwt: worktree number required"
                return 1
            fi
            local wt_path="${WT_PATHS[$wt]}"
            if [[ -z "$wt_path" ]]; then
                echo "zwt: no worktree $wt configured for $project"
                return 1
            fi
            echo "Removing worktree at ${wt_path}..."
            git -C "$source_path" worktree remove "$wt_path"
            ;;
        create)
            local wt="$2"
            local branch="$3"
            if [[ -z "$wt" ]]; then
                echo "zwt: worktree number required"
                return 1
            fi
            local wt_path="${WT_PATHS[$wt]}"
            if [[ -z "$wt_path" ]]; then
                echo "zwt: no worktree $wt configured for $project"
                return 1
            fi
            if [[ -d "$wt_path" ]]; then
                echo "zwt: ${wt_path} already exists"
                return 1
            fi

            # Resolve base branch from wt1 (same as cycle)
            local wt1_path="${WT_PATHS[1]}"
            local base_branch
            base_branch=$(git -C "$wt1_path" branch --show-current 2>/dev/null)
            if [[ -z "$base_branch" ]]; then
                echo "zwt: wt1 at ${wt1_path} is in detached HEAD state"
                return 1
            fi

            if [[ -n "$branch" ]]; then
                # Check if branch already exists
                if git -C "$source_path" show-ref --verify --quiet "refs/heads/${branch}"; then
                    git -C "$source_path" worktree add "$wt_path" "$branch"
                else
                    git -C "$source_path" worktree add -b "$branch" "$wt_path" "$base_branch"
                fi
            else
                # Default: new branch named <project>-wt<N>
                local default_branch="${project}-wt${wt}"
                git -C "$source_path" worktree add -b "$default_branch" "$wt_path" "$base_branch"
            fi

            if [[ $? -eq 0 ]]; then
                _zwt_copy_env_files "${WT_PATHS[1]:-$source_path}" "$wt_path"
                _zwt_copy_node_modules "${WT_PATHS[1]:-$source_path}" "$wt_path"
                zl-vscode "$project" "$wt"
                echo "Created worktree at ${wt_path}"
                echo "Start session:  zl ${project} ${wt} [--dev]"
            fi
            ;;
        cycle)
            local wt="$2"
            local new_branch="$3"
            if [[ -z "$wt" || -z "$new_branch" ]]; then
                echo "Usage: zwt cycle <project> <wt#> <new-branch>"
                return 1
            fi
            local wt_path="${WT_PATHS[$wt]}"
            if [[ -z "$wt_path" ]]; then
                echo "zwt: no worktree $wt configured for $project"
                return 1
            fi

            # --- Resolve the base branch from wt1 (main worktree) ---
            local wt1_path="${WT_PATHS[1]}"
            local base_branch
            base_branch=$(git -C "$wt1_path" branch --show-current 2>/dev/null)
            if [[ -z "$base_branch" ]]; then
                echo "ABORT: wt1 at ${wt1_path} is in detached HEAD state"
                return 1
            fi
            echo "==> Base branch (from wt1): ${base_branch}"

            # --- Safety checks on the existing worktree ---
            if [[ -d "$wt_path" ]]; then
                local old_branch
                old_branch=$(git -C "$wt_path" branch --show-current 2>/dev/null)
                echo "==> Current branch in wt${wt}: ${old_branch:-detached HEAD}"

                # Check for uncommitted changes
                echo "==> Checking for uncommitted changes..."
                if ! git -C "$wt_path" diff --quiet 2>/dev/null || ! git -C "$wt_path" diff --cached --quiet 2>/dev/null; then
                    echo "ABORT: worktree has uncommitted changes"
                    echo "  cd ${wt_path} && git status"
                    return 1
                fi

                # Check for untracked files
                local untracked
                untracked=$(git -C "$wt_path" ls-files --others --exclude-standard 2>/dev/null)
                if [[ -n "$untracked" ]]; then
                    echo "ABORT: worktree has untracked files"
                    echo "  cd ${wt_path} && git status"
                    return 1
                fi
                echo "    Clean."

                # Check that base_branch contains all commits from old_branch
                if [[ -n "$old_branch" ]]; then
                    echo "==> Checking that '${base_branch}' contains '${old_branch}'..."
                    local behind
                    behind=$(git -C "$wt_path" rev-list --count "${base_branch}..${old_branch}" 2>/dev/null)
                    if [[ "$behind" -gt 0 ]]; then
                        echo "ABORT: '${base_branch}' is missing ${behind} commit(s) from '${old_branch}'"
                        echo "  Merge or rebase '${old_branch}' into '${base_branch}' first"
                        return 1
                    fi
                    echo "    Merged."
                fi

                # All clear — switch to new branch in-place
                echo "==> Switching to new branch '${new_branch}' from '${base_branch}'..."
                git -C "$wt_path" switch -C "$new_branch" "$base_branch" || return 1

                # Delete the old local branch if it was merged
                if [[ -n "$old_branch" ]]; then
                    if git -C "$source_path" branch -d "$old_branch" 2>/dev/null; then
                        echo "    Deleted local branch '${old_branch}'."
                    else
                        echo "    Kept local branch '${old_branch}' (git says not fully merged)."
                    fi
                fi
            else
                # No existing worktree — fall back to full create
                echo "==> No existing worktree at ${wt_path}, creating fresh..."
                git -C "$source_path" worktree add -b "$new_branch" "$wt_path" "$base_branch" || return 1
                _zwt_copy_env_files "${wt1_path}" "$wt_path"
                _zwt_copy_node_modules "${wt1_path}" "$wt_path"
                zl-vscode "$project" "$wt"
            fi

            echo "==> Done! Start session:  zl ${project} ${wt} [--dev]"
            ;;
    esac
}
