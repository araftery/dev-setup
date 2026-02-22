# Layout generation for zl sessions
# Generates Zellij KDL layout files to /tmp/
#
# All layouts run `claude` in the left/main pane.
# Right-side panes are either plain terminals (empty command) or run a command.

# Helpers: emit tab-bar / status-bar plugin panes
_zl_bar_top() {
    echo '    pane size=1 borderless=true {'
    echo '        plugin location="zellij:tab-bar"'
    echo '    }'
}
_zl_bar_bottom() {
    echo '    pane size=2 borderless=true {'
    echo '        plugin location="zellij:status-bar"'
    echo '    }'
}

# Helper: emit a KDL pane block
# Usage: _zl_pane <size> [command]
_zl_pane() {
    local size="$1"
    local cmd="$2"
    if [[ -n "$cmd" ]]; then
        echo "            pane size=\"${size}\" {"
        echo "                command \"zsh\""
        echo "                args \"-c\" \"${cmd}; exec zsh\""
        echo "            }"
    else
        echo "            pane size=\"${size}\""
    fi
}

zl-layout-simple() {
    local layout_file="/tmp/zl-layout-simple.kdl"
    cat > "$layout_file" <<'KDL'
layout {
    pane size=1 borderless=true {
        plugin location="zellij:tab-bar"
    }
    pane split_direction="vertical" {
        pane size="50%" {
            command "zsh"
            args "-c" "claude; exec zsh"
        }
        pane size="50%"
    }
    pane size=2 borderless=true {
        plugin location="zellij:status-bar"
    }
}
KDL
    echo "$layout_file"
}

# Generate a dev layout based on style + commands
# Usage: zl-layout-dev <style> [cmd1] [cmd2] [cmd3]
#
# Commands fill the right-side panes in order.
# An empty string means a plain terminal pane.
#
# Styles:
#   right-split   — 65% claude left, 2 stacked right panes (default)
#   right-triple  — 60% claude left, 3 stacked right panes
#   bottom-split  — 70% claude top, 2 side-by-side bottom panes
#   top-split     — 70% top (65% claude + 35% shell), 30% bottom pane
#   grid          — 2x2 grid, claude top-left
zl-layout-dev() {
    local style="${1:-right-split}"
    shift
    local -a cmds=("$@")
    local layout_file="/tmp/zl-layout-dev.kdl"

    case "$style" in
        right-split)
            {
                echo 'layout {'
                _zl_bar_top
                echo '    pane split_direction="vertical" {'
                echo '        pane size="50%" {'
                echo '            command "zsh"'
                echo '            args "-c" "claude; exec zsh"'
                echo '        }'
                echo '        pane split_direction="horizontal" size="50%" {'
                _zl_pane "50%" "${cmds[1]}"
                _zl_pane "50%" "${cmds[2]}"
                echo '        }'
                echo '    }'
                _zl_bar_bottom
                echo '}'
            } > "$layout_file"
            ;;
        right-triple)
            {
                echo 'layout {'
                _zl_bar_top
                echo '    pane split_direction="vertical" {'
                echo '        pane size="60%" {'
                echo '            command "zsh"'
                echo '            args "-c" "claude; exec zsh"'
                echo '        }'
                echo '        pane split_direction="horizontal" size="40%" {'
                _zl_pane "34%" "${cmds[1]}"
                _zl_pane "33%" "${cmds[2]}"
                _zl_pane "33%" "${cmds[3]}"
                echo '        }'
                echo '    }'
                _zl_bar_bottom
                echo '}'
            } > "$layout_file"
            ;;
        bottom-split)
            {
                echo 'layout {'
                _zl_bar_top
                echo '    pane split_direction="horizontal" {'
                echo '        pane size="70%" {'
                echo '            command "zsh"'
                echo '            args "-c" "claude; exec zsh"'
                echo '        }'
                echo '        pane split_direction="vertical" size="30%" {'
                _zl_pane "50%" "${cmds[1]}"
                _zl_pane "50%" "${cmds[2]}"
                echo '        }'
                echo '    }'
                _zl_bar_bottom
                echo '}'
            } > "$layout_file"
            ;;
        top-split)
            {
                echo 'layout {'
                _zl_bar_top
                echo '    pane split_direction="horizontal" {'
                echo '        pane split_direction="vertical" size="70%" {'
                echo '            pane size="65%" {'
                echo '                command "zsh"'
                echo '                args "-c" "claude; exec zsh"'
                echo '            }'
                _zl_pane "35%" "${cmds[1]}"
                echo '        }'
                _zl_pane "30%" "${cmds[2]}"
                echo '    }'
                _zl_bar_bottom
                echo '}'
            } > "$layout_file"
            ;;
        grid)
            {
                echo 'layout {'
                _zl_bar_top
                echo '    pane split_direction="horizontal" {'
                echo '        pane split_direction="vertical" size="50%" {'
                echo '            pane size="50%" {'
                echo '                command "zsh"'
                echo '                args "-c" "claude; exec zsh"'
                echo '            }'
                _zl_pane "50%" "${cmds[1]}"
                echo '        }'
                echo '        pane split_direction="vertical" size="50%" {'
                _zl_pane "50%" "${cmds[2]}"
                _zl_pane "50%" "${cmds[3]}"
                echo '        }'
                echo '    }'
                _zl_bar_bottom
                echo '}'
            } > "$layout_file"
            ;;
        *)
            echo "zl: unknown layout style '$style'" >&2
            return 1
            ;;
    esac

    echo "$layout_file"
}
