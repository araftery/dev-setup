# Layout generation for zl sessions
# Generates Zellij KDL layout files to /tmp/

zl-layout-simple() {
    local layout_file="/tmp/zl-layout-simple.kdl"
    cat > "$layout_file" <<'KDL'
layout {
    pane split_direction="vertical" {
        pane size="50%"
        pane size="50%"
    }
}
KDL
    echo "$layout_file"
}

# Generate a dev layout based on style + commands
# Usage: zl-layout-dev <style> <cmd1> [cmd2] [cmd3]
#
# Styles:
#   right-split   — 65% editor left, 2 stacked right panes (default)
#   right-triple  — 60% editor left, 3 stacked right panes
#   bottom-split  — 70% editor top, 2 side-by-side bottom panes
#   grid          — 2x2 grid, editor top-left
zl-layout-dev() {
    local style="${1:-right-split}"
    shift
    local cmds=("$@")
    local layout_file="/tmp/zl-layout-dev.kdl"

    case "$style" in
        right-split)
            local c1="${cmds[0]:-echo 'no command configured'}"
            local c2="${cmds[1]:-echo 'no command configured'}"
            cat > "$layout_file" <<KDL
layout {
    pane split_direction="vertical" {
        pane size="65%"
        pane split_direction="horizontal" size="35%" {
            pane size="50%" {
                command "bash"
                args "-c" "${c1}"
            }
            pane size="50%" {
                command "bash"
                args "-c" "${c2}"
            }
        }
    }
}
KDL
            ;;
        right-triple)
            local c1="${cmds[0]:-echo 'no command configured'}"
            local c2="${cmds[1]:-echo 'no command configured'}"
            local c3="${cmds[2]:-echo 'no command configured'}"
            cat > "$layout_file" <<KDL
layout {
    pane split_direction="vertical" {
        pane size="60%"
        pane split_direction="horizontal" size="40%" {
            pane size="34%" {
                command "bash"
                args "-c" "${c1}"
            }
            pane size="33%" {
                command "bash"
                args "-c" "${c2}"
            }
            pane size="33%" {
                command "bash"
                args "-c" "${c3}"
            }
        }
    }
}
KDL
            ;;
        bottom-split)
            local c1="${cmds[0]:-echo 'no command configured'}"
            local c2="${cmds[1]:-echo 'no command configured'}"
            cat > "$layout_file" <<KDL
layout {
    pane split_direction="horizontal" {
        pane size="70%"
        pane split_direction="vertical" size="30%" {
            pane size="50%" {
                command "bash"
                args "-c" "${c1}"
            }
            pane size="50%" {
                command "bash"
                args "-c" "${c2}"
            }
        }
    }
}
KDL
            ;;
        grid)
            local c1="${cmds[0]:-echo 'no command configured'}"
            local c2="${cmds[1]:-echo 'no command configured'}"
            local c3="${cmds[2]:-echo 'no command configured'}"
            cat > "$layout_file" <<KDL
layout {
    pane split_direction="horizontal" {
        pane split_direction="vertical" size="50%" {
            pane size="50%"
            pane size="50%" {
                command "bash"
                args "-c" "${c1}"
            }
        }
        pane split_direction="vertical" size="50%" {
            pane size="50%" {
                command "bash"
                args "-c" "${c2}"
            }
            pane size="50%" {
                command "bash"
                args "-c" "${c3}"
            }
        }
    }
}
KDL
            ;;
        *)
            echo "zl: unknown layout style '$style'" >&2
            return 1
            ;;
    esac

    echo "$layout_file"
}
