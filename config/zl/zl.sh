# zl — Color-coded Zellij session manager
# Usage: zl <project> <wt#> [--dev]
#
# --dev: singleton session per worktree (attach if exists, create if not)
# no flag: always creates a new session with a simple 2-pane layout

zl() {
    local project="$1"
    local wt="${2:-1}"
    local dev_mode=0

    if [[ -z "$project" ]]; then
        echo "Usage: zl <project> <wt#> [--dev]"
        echo ""
        echo "Available projects:"
        for f in "${HOME}/.config/zl/projects"/*.sh; do
            echo "  $(basename "$f" .sh)"
        done
        return 1
    fi

    # Check for --dev flag
    shift 2 2>/dev/null
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --dev) dev_mode=1 ;;
            *) echo "zl: unknown flag '$1'"; return 1 ;;
        esac
        shift
    done

    local config_file="${HOME}/.config/zl/projects/${project}.sh"
    if [[ ! -f "$config_file" ]]; then
        echo "zl: unknown project '$project'"
        echo "Available: $(ls "${HOME}/.config/zl/projects/" | sed 's/\.sh$//' | tr '\n' ' ')"
        return 1
    fi

    # Source project config
    source "$config_file"

    local wt_path="${WT_PATHS[$wt]}"
    local bg_color="${WT_BG[$wt]}"

    if [[ -z "$wt_path" ]]; then
        echo "zl: no worktree $wt configured for $project"
        return 1
    fi

    if [[ ! -d "$wt_path" ]]; then
        echo "zl: directory does not exist: $wt_path"
        return 1
    fi

    # Set Ghostty tab title (OSC 2) — must happen before Zellij
    local tab_label="${project}-${wt}"
    [[ "$dev_mode" -eq 1 ]] && tab_label="${tab_label}-dev"
    printf '\e]2;%s\e\\' "$tab_label"

    # Set terminal background (OSC 11) — must happen before Zellij
    if [[ -n "$bg_color" ]]; then
        printf '\e]11;%s\e\\' "$bg_color"
    fi

    # cd into worktree
    cd "$wt_path" || return 1

    if [[ "$dev_mode" -eq 1 ]]; then
        # Dev sessions are singletons — one per worktree
        local session_name="${project}-${wt}-dev"

        if zellij list-sessions -s 2>/dev/null | grep -qx "$session_name"; then
            zellij attach "$session_name"
        else
            local style="${DEV_LAYOUT:-right-split}"
            local layout_file
            layout_file=$(zl-layout-dev "$style" "${DEV_COMMANDS[@]}")
            zellij --new-session-with-layout "$layout_file" -s "$session_name"
        fi
    else
        # Non-dev: always create a new session with a unique sequential name
        local existing
        existing=$(zellij list-sessions -s 2>/dev/null)
        local prefix="${project}-${wt}-"
        local letter session_name
        for letter in {A..Z}; do
            session_name="${prefix}${letter}"
            if ! echo "$existing" | grep -qx "$session_name"; then
                break
            fi
        done

        local layout_file
        layout_file=$(zl-layout-simple)
        zellij --new-session-with-layout "$layout_file" -s "$session_name"
    fi
}
