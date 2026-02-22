PROJECT_NAME="agentic-cro"
SOURCE_REPO="$HOME/workspace/intelligems"
WT_PATHS=([1]="$HOME/workspace/agentic-cro" [2]="$HOME/workspace/agentic-cro-2")
WT_BG=([1]="#2a1a1e" [2]="#381e28")
WT_VSCODE_ACCENT=([1]="#5c2a2e" [2]="#703444")
DEV_LAYOUT="top-split"
DEV_COMMANDS=("cd ./cro-agent && exec zsh" "cd ./cro-agent/backend && pnpm run dev")
DEV_COMMANDS_2=("cd ./cro-agent && exec zsh" "cd ./cro-agent/backend && PORT=3001 pnpm run dev")
