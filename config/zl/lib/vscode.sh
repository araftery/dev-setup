# VSCode color integration for zl sessions
# Merges workbench.colorCustomizations into .vscode/settings.json

zl-vscode() {
    local project="$1"
    local wt="${2:-1}"
    local config_file="${HOME}/.config/zl/projects/${project}.sh"

    if [[ ! -f "$config_file" ]]; then
        echo "zl-vscode: unknown project '$project'"
        return 1
    fi

    # Source config, then read the values we need
    local WT_PATHS WT_VSCODE_ACCENT
    source "$config_file"

    local wt_path="${WT_PATHS[$wt]}"
    local accent="${WT_VSCODE_ACCENT[$wt]}"

    if [[ -z "$wt_path" || -z "$accent" ]]; then
        echo "zl-vscode: no config for ${project} wt${wt}"
        return 1
    fi

    local settings_dir="${wt_path}/.vscode"
    local settings_file="${settings_dir}/settings.json"

    mkdir -p "$settings_dir"

    local fg="#cccccc"

    local colors
    colors=$(cat <<ENDJSON
{
    "workbench.colorCustomizations": {
        "titleBar.activeBackground": "${accent}",
        "titleBar.activeForeground": "${fg}",
        "statusBar.background": "${accent}",
        "statusBar.foreground": "${fg}"
    }
}
ENDJSON
    )

    if ! command -v jq &>/dev/null; then
        echo "zl-vscode: jq is required but not installed"
        return 1
    fi

    if [[ -f "$settings_file" ]]; then
        local merged
        merged=$(jq -s '.[0] * .[1]' "$settings_file" <(echo "$colors"))
        echo "$merged" > "$settings_file"
    else
        echo "$colors" | jq '.' > "$settings_file"
    fi

    echo "Updated ${settings_file}"
}

zl-vscode-all() {
    local config_dir="${HOME}/.config/zl/projects"
    for config_file in "$config_dir"/*.sh; do
        local project
        project=$(basename "$config_file" .sh)

        # Source config to get WT_PATHS keys
        local WT_PATHS
        source "$config_file"

        local wt
        for wt in ${(k)WT_PATHS}; do
            if [[ -d "${WT_PATHS[$wt]}" ]]; then
                zl-vscode "$project" "$wt"
            else
                echo "Skipping ${project} wt${wt} — ${WT_PATHS[$wt]} does not exist"
            fi
        done
    done
}
