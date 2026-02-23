use crate::paths;
use crate::types::{Decision, HookInput};

/// Safe read-only commands (first token whitelist)
const SAFE_COMMANDS: &[&str] = &[
    "cat", "head", "tail", "less", "more", "wc", "file", "stat", "du", "df",
    "ls", "find", "grep", "rg", "ag", "sort", "uniq", "diff", "comm", "tr",
    "cut", "jq", "yq", "which", "type", "command", "echo", "printf", "date",
    "uname", "whoami", "hostname", "pwd", "env", "printenv", "id", "groups",
    "test", "true", "false",
];

/// Safe git subcommands (read-only)
const SAFE_GIT_SUBCOMMANDS: &[&str] = &[
    "status", "log", "diff", "show", "branch", "tag", "remote", "describe",
    "rev-parse", "ls-files", "ls-tree", "blame", "shortlog",
];

/// Build/lint/test commands that are safe to auto-allow
const SAFE_BUILD_COMMANDS: &[&str] = &[
    "cargo", "pytest", "mypy", "ruff", "black", "flake8", "pylint",
    "eslint", "prettier", "tsc", "biome", "golangci-lint", "make", "cmake",
    "curl", "wget", "brew",
];

/// Safe subcommands for package managers (npm, pnpm, yarn, bun)
const SAFE_PKG_SUBCOMMANDS: &[&str] = &[
    "test", "run", "build", "lint", "typecheck", "check", "exec",
];

/// Package runner commands that act as transparent wrappers
const PKG_RUNNERS: &[&str] = &["npx", "uvx", "pnpx", "bunx"];

/// Safe cargo subcommands
const SAFE_CARGO_SUBCOMMANDS: &[&str] = &[
    "test", "build", "check", "clippy", "fmt",
];

/// Safe go subcommands
const SAFE_GO_SUBCOMMANDS: &[&str] = &[
    "test", "vet", "build",
];

/// Safe python -m modules
const SAFE_PYTHON_MODULES: &[&str] = &[
    "pytest", "mypy", "ruff", "black",
];

/// gh subcommands that require further sub-subcommand validation
/// These are only safe when used with read-only sub-subcommands
const GH_SUBCOMMANDS_NEEDING_CHECK: &[&str] = &[
    "pr", "issue", "repo", "run", "workflow", "release", "label", "project",
];

/// Safe sub-subcommands for gh (read-only operations)
const SAFE_GH_SUB_SUBCOMMANDS: &[&str] = &[
    "view", "list", "diff", "checks", "status", "ls",
];

/// Evaluate a Bash tool invocation
pub fn evaluate(input: &HookInput) -> Decision {
    let command = match input.get_input_str("command") {
        Some(cmd) => cmd.trim(),
        None => return Decision::Abstain,
    };

    if command.is_empty() {
        return Decision::Abstain;
    }

    let cwd = input.cwd.as_deref().unwrap_or("");

    // Split compound command into segments
    let segments = split_compound_command(command);

    let mut any_ask = false;
    let mut current_dir = cwd.to_string();

    for segment in &segments {
        let seg = segment.trim();
        if seg.is_empty() {
            continue;
        }

        // Track cd across segments
        if let Some(new_dir) = extract_cd_target(seg, &current_dir) {
            current_dir = new_dir;
            continue;
        }

        // Check for hard deny (rm -rf)
        if is_rm_rf(seg) {
            return Decision::Deny("rm -rf is never allowed".to_string());
        }

        // Check for secrets file references
        let tokens: Vec<&str> = seg.split_whitespace().collect();
        if !tokens.is_empty() && paths::args_reference_secrets(&tokens[1..]) {
            return Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string());
        }

        // Check for destructive commands
        if is_destructive(seg) {
            any_ask = true;
            continue;
        }

        // Check if it's a safe command
        if !is_safe_command(seg, &current_dir) {
            return Decision::Abstain;
        }
    }

    if any_ask {
        return Decision::Ask("Command contains destructive operations".to_string());
    }

    Decision::Allow("Safe read-only/build command".to_string())
}

/// Split a compound command on &&, ||, ;, and |
/// Respects single and double quotes — operators inside quotes are not split points.
/// Backslash escapes the immediately following character inside and outside quotes.
fn split_compound_command(command: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let bytes = command.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while i < len {
        let ch = bytes[i];

        // Backslash escape — consume next char literally
        if ch == b'\\' && !in_single_quote {
            current.push(ch as char);
            if i + 1 < len {
                i += 1;
                current.push(bytes[i] as char);
            }
            i += 1;
            continue;
        }

        // Quote tracking
        if ch == b'\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            current.push(ch as char);
            i += 1;
            continue;
        }
        if ch == b'"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            current.push(ch as char);
            i += 1;
            continue;
        }

        // Only split on operators when outside quotes
        if !in_single_quote && !in_double_quote {
            if ch == b'&' && i + 1 < len && bytes[i + 1] == b'&' {
                segments.push(current.clone());
                current.clear();
                i += 2;
                continue;
            }
            if ch == b'|' && i + 1 < len && bytes[i + 1] == b'|' {
                segments.push(current.clone());
                current.clear();
                i += 2;
                continue;
            }
            if ch == b'|' {
                segments.push(current.clone());
                current.clear();
                i += 1;
                continue;
            }
            if ch == b';' {
                segments.push(current.clone());
                current.clear();
                i += 1;
                continue;
            }
        }

        current.push(ch as char);
        i += 1;
    }

    if !current.trim().is_empty() {
        segments.push(current);
    }

    segments
}

/// Extract the target directory from a `cd` command, resolving it against current_dir
fn extract_cd_target(segment: &str, current_dir: &str) -> Option<String> {
    let tokens: Vec<&str> = segment.trim().split_whitespace().collect();
    if tokens.is_empty() || tokens[0] != "cd" {
        return None;
    }

    let target = tokens.get(1).copied().unwrap_or("~");
    let target = if target == "~" {
        "/Users/araftery"
    } else {
        target
    };

    let resolved = paths::normalize_path(target, current_dir);
    Some(resolved.to_string_lossy().to_string())
}

/// Check if a command is `rm` with both -r and -f flags (in any form)
fn is_rm_rf(segment: &str) -> bool {
    let tokens: Vec<&str> = segment.trim().split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }

    // Must start with `rm`
    if tokens[0] != "rm" {
        return false;
    }

    let mut has_recursive = false;
    let mut has_force = false;

    for token in &tokens[1..] {
        if !token.starts_with('-') {
            continue;
        }

        // Long flags
        if *token == "--recursive" {
            has_recursive = true;
            continue;
        }
        if *token == "--force" {
            has_force = true;
            continue;
        }

        // Short flags: could be combined like -rf, -fr, -rfi, etc.
        let flag_chars: Vec<char> = token.chars().skip(1).collect();
        if flag_chars.contains(&'r') {
            has_recursive = true;
        }
        if flag_chars.contains(&'f') {
            has_force = true;
        }
    }

    has_recursive && has_force
}

/// Check if a command is a destructive operation that should prompt for confirmation
fn is_destructive(segment: &str) -> bool {
    let tokens: Vec<&str> = segment.trim().split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }

    let cmd = tokens[0];

    // rm (any form) — but not rm -rf which is caught earlier as hard deny
    if cmd == "rm" {
        return true;
    }

    // mv
    if cmd == "mv" {
        return true;
    }

    // chmod, chown
    if cmd == "chmod" || cmd == "chown" {
        return true;
    }

    // git subcommands
    if cmd == "git" && tokens.len() > 1 {
        let subcmd = tokens[1];

        // git rm
        if subcmd == "rm" {
            return true;
        }

        // git rebase
        if subcmd == "rebase" {
            return true;
        }

        // git reset --hard
        if subcmd == "reset" && tokens.iter().any(|t| *t == "--hard") {
            return true;
        }

        // git checkout . (discard all changes)
        if subcmd == "checkout" && tokens.iter().any(|t| *t == ".") {
            return true;
        }

        // git clean
        if subcmd == "clean" {
            return true;
        }

        // Any git subcommand with -f or --force
        if tokens.iter().skip(2).any(|t| *t == "-f" || *t == "--force") {
            return true;
        }
    }

    false
}

/// Check if a command segment is a safe (auto-allowable) command
fn is_safe_command(segment: &str, current_dir: &str) -> bool {
    let tokens: Vec<&str> = segment.trim().split_whitespace().collect();
    if tokens.is_empty() {
        return true; // empty segment is safe
    }

    let cmd = tokens[0];

    // Package runners (npx, uvx, pnpx, bunx) — unwrap and evaluate inner command
    if PKG_RUNNERS.contains(&cmd) {
        let inner = unwrap_pkg_runner(&tokens);
        if inner.is_empty() {
            return false;
        }
        return is_safe_command(&inner.join(" "), current_dir);
    }

    // Simple safe commands
    if SAFE_COMMANDS.contains(&cmd) {
        return true;
    }

    // Build/lint/test commands
    if SAFE_BUILD_COMMANDS.contains(&cmd) {
        // For cargo, check subcommand
        if cmd == "cargo" {
            return tokens.get(1).map_or(false, |sub| SAFE_CARGO_SUBCOMMANDS.contains(sub));
        }
        return true;
    }

    // git with safe subcommand
    if cmd == "git" {
        if let Some(subcmd) = tokens.get(1) {
            if SAFE_GIT_SUBCOMMANDS.contains(subcmd) {
                return true;
            }
            // git stash list
            if *subcmd == "stash" && tokens.get(2).map_or(false, |t| *t == "list") {
                return true;
            }
            // git config --get or --list
            if *subcmd == "config" && tokens.iter().any(|t| *t == "--get" || *t == "--list") {
                return true;
            }
        }
        return false;
    }

    // gh (GitHub CLI) with safe subcommands
    if cmd == "gh" {
        if let Some(subcmd) = tokens.get(1) {
            // gh api is always safe (read-only by default)
            if *subcmd == "api" {
                return true;
            }
            // gh status, gh search — no sub-subcommand needed
            if *subcmd == "status" || *subcmd == "search" {
                return true;
            }
            // Subcommands that need a read-only sub-subcommand
            if GH_SUBCOMMANDS_NEEDING_CHECK.contains(subcmd) {
                if let Some(sub_subcmd) = tokens.get(2) {
                    return SAFE_GH_SUB_SUBCOMMANDS.contains(sub_subcmd);
                }
                // bare `gh pr` / `gh issue` with no action — not safe
                return false;
            }
        }
        return false;
    }

    // Package managers (npm, pnpm, yarn, bun)
    if matches!(cmd, "npm" | "pnpm" | "yarn" | "bun") {
        return tokens.get(1).map_or(false, |sub| SAFE_PKG_SUBCOMMANDS.contains(sub));
    }

    // go with safe subcommand
    if cmd == "go" {
        return tokens.get(1).map_or(false, |sub| SAFE_GO_SUBCOMMANDS.contains(sub));
    }

    // python -m <safe_module>
    if cmd == "python" || cmd == "python3" {
        if tokens.get(1) == Some(&"-m") {
            if let Some(module) = tokens.get(2) {
                return SAFE_PYTHON_MODULES.contains(module);
            }
        }
        return false;
    }

    false
}

/// Unwrap a package runner command (npx, uvx, etc.) to get the inner command tokens.
/// Strips the runner and any flags before the actual command.
fn unwrap_pkg_runner<'a>(tokens: &'a [&'a str]) -> Vec<&'a str> {
    // Skip the runner itself (tokens[0])
    let mut i = 1;

    // Skip flags that come before the command (e.g. --yes, -p, -y)
    while i < tokens.len() {
        let t = tokens[i];
        if t.starts_with('-') {
            i += 1;
            // If the flag takes a value (e.g. -p package), skip the next token too
            // Simple heuristic: if flag is -p or --package, skip next
            if matches!(t, "-p" | "--package") && i < tokens.len() {
                i += 1;
            }
        } else {
            break;
        }
    }

    if i < tokens.len() {
        tokens[i..].to_vec()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(command: &str, cwd: &str) -> HookInput {
        let mut tool_input = std::collections::HashMap::new();
        tool_input.insert("command".to_string(), serde_json::json!(command));
        HookInput {
            tool_name: Some("Bash".to_string()),
            tool_input: Some(tool_input),
            cwd: Some(cwd.to_string()),
        }
    }

    fn cwd() -> &'static str {
        "/Users/araftery/workspace/project"
    }

    // ===== Hard deny (rm -rf) =====

    #[test]
    fn test_rm_rf_root() {
        assert_eq!(evaluate(&make_input("rm -rf /", cwd())), Decision::Deny("rm -rf is never allowed".to_string()));
    }

    #[test]
    fn test_rm_rf_dot() {
        assert_eq!(evaluate(&make_input("rm -rf .", cwd())), Decision::Deny("rm -rf is never allowed".to_string()));
    }

    #[test]
    fn test_rm_fr() {
        assert_eq!(evaluate(&make_input("rm -fr /tmp/foo", cwd())), Decision::Deny("rm -rf is never allowed".to_string()));
    }

    #[test]
    fn test_rm_r_f_separate() {
        assert_eq!(evaluate(&make_input("rm -r -f foo", cwd())), Decision::Deny("rm -rf is never allowed".to_string()));
    }

    #[test]
    fn test_rm_recursive_f() {
        assert_eq!(evaluate(&make_input("rm --recursive -f bar", cwd())), Decision::Deny("rm -rf is never allowed".to_string()));
    }

    #[test]
    fn test_rm_rf_in_compound() {
        assert_eq!(evaluate(&make_input("ls && rm -rf /tmp", cwd())), Decision::Deny("rm -rf is never allowed".to_string()));
    }

    #[test]
    fn test_rm_rf_in_pipe() {
        assert_eq!(evaluate(&make_input("echo hello | rm -rf baz", cwd())), Decision::Deny("rm -rf is never allowed".to_string()));
    }

    // ===== Destructive → ask =====

    #[test]
    fn test_rm_file() {
        assert_eq!(evaluate(&make_input("rm foo.txt", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_rm_r_dir() {
        assert_eq!(evaluate(&make_input("rm -r dir/", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_git_rm() {
        assert_eq!(evaluate(&make_input("git rm file.txt", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_mv() {
        assert_eq!(evaluate(&make_input("mv foo bar", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_git_rebase() {
        assert_eq!(evaluate(&make_input("git rebase main", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_git_push_force() {
        assert_eq!(evaluate(&make_input("git push --force", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_git_push_f() {
        assert_eq!(evaluate(&make_input("git push -f origin main", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_git_reset_hard() {
        assert_eq!(evaluate(&make_input("git reset --hard HEAD~1", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_git_checkout_dot() {
        assert_eq!(evaluate(&make_input("git checkout .", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_git_clean() {
        assert_eq!(evaluate(&make_input("git clean -fd", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_chmod() {
        assert_eq!(evaluate(&make_input("chmod 777 file", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    #[test]
    fn test_chown() {
        assert_eq!(evaluate(&make_input("chown root file", cwd())), Decision::Ask("Command contains destructive operations".to_string()));
    }

    // ===== Secrets → deny =====

    #[test]
    fn test_cat_env() {
        assert_eq!(evaluate(&make_input("cat .env", cwd())), Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string()));
    }

    #[test]
    fn test_cat_env_prod() {
        assert_eq!(evaluate(&make_input("cat .env.prod", cwd())), Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string()));
    }

    #[test]
    fn test_cat_dev_vars() {
        assert_eq!(evaluate(&make_input("cat .dev.vars", cwd())), Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string()));
    }

    #[test]
    fn test_source_env() {
        assert_eq!(evaluate(&make_input("source .env", cwd())), Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string()));
    }

    #[test]
    fn test_grep_env_local() {
        assert_eq!(evaluate(&make_input("grep API_KEY .env.local", cwd())), Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string()));
    }

    #[test]
    fn test_secrets_in_compound() {
        assert_eq!(evaluate(&make_input("ls && cat .env", cwd())), Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string()));
    }

    // ===== Safe read-only → allow =====

    #[test]
    fn test_ls() {
        assert_eq!(evaluate(&make_input("ls", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_ls_la() {
        assert_eq!(evaluate(&make_input("ls -la /Users/araftery/workspace", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_cat_file() {
        assert_eq!(evaluate(&make_input("cat foo.txt", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_head() {
        assert_eq!(evaluate(&make_input("head -n 10 bar.ts", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_grep_pattern() {
        assert_eq!(evaluate(&make_input("grep pattern src/", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_rg() {
        assert_eq!(evaluate(&make_input("rg \"TODO\" .", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_find() {
        assert_eq!(evaluate(&make_input("find . -name \"*.ts\"", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_wc() {
        assert_eq!(evaluate(&make_input("wc -l file.txt", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_status() {
        assert_eq!(evaluate(&make_input("git status", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_log() {
        assert_eq!(evaluate(&make_input("git log --oneline", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_diff() {
        assert_eq!(evaluate(&make_input("git diff", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_diff_head() {
        assert_eq!(evaluate(&make_input("git diff HEAD~1", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_show() {
        assert_eq!(evaluate(&make_input("git show HEAD", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_branch() {
        assert_eq!(evaluate(&make_input("git branch -a", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_rev_parse() {
        assert_eq!(evaluate(&make_input("git rev-parse HEAD", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_ls_files() {
        assert_eq!(evaluate(&make_input("git ls-files", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_git_blame() {
        assert_eq!(evaluate(&make_input("git blame file.ts", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_safe_pipe() {
        assert_eq!(evaluate(&make_input("cat foo | grep bar | sort", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_safe_chain() {
        assert_eq!(evaluate(&make_input("ls && git status && echo done", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_echo() {
        assert_eq!(evaluate(&make_input("echo \"hello world\"", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_pwd() {
        assert_eq!(evaluate(&make_input("pwd", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_which() {
        assert_eq!(evaluate(&make_input("which node", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    // ===== Build/lint/test → allow =====

    #[test]
    fn test_npm_test() {
        assert_eq!(evaluate(&make_input("npm test", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_npm_run_lint() {
        assert_eq!(evaluate(&make_input("npm run lint", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_npm_run_typecheck() {
        assert_eq!(evaluate(&make_input("npm run typecheck", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_npm_run_build() {
        assert_eq!(evaluate(&make_input("npm run build", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_pnpm_test() {
        assert_eq!(evaluate(&make_input("pnpm test", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_pnpm_lint() {
        assert_eq!(evaluate(&make_input("pnpm lint", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_yarn_test() {
        assert_eq!(evaluate(&make_input("yarn test", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_bun_test() {
        assert_eq!(evaluate(&make_input("bun test", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_cargo_test() {
        assert_eq!(evaluate(&make_input("cargo test", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_cargo_build() {
        assert_eq!(evaluate(&make_input("cargo build", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_cargo_check() {
        assert_eq!(evaluate(&make_input("cargo check", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_cargo_clippy() {
        assert_eq!(evaluate(&make_input("cargo clippy", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_cargo_fmt_check() {
        assert_eq!(evaluate(&make_input("cargo fmt --check", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_pytest() {
        assert_eq!(evaluate(&make_input("pytest", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_python_m_pytest() {
        assert_eq!(evaluate(&make_input("python -m pytest", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_mypy() {
        assert_eq!(evaluate(&make_input("mypy src/", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_ruff_check() {
        assert_eq!(evaluate(&make_input("ruff check .", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_eslint() {
        assert_eq!(evaluate(&make_input("eslint src/", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_prettier() {
        assert_eq!(evaluate(&make_input("prettier --check .", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_tsc() {
        assert_eq!(evaluate(&make_input("tsc --noEmit", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_biome() {
        assert_eq!(evaluate(&make_input("biome check .", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_go_test() {
        assert_eq!(evaluate(&make_input("go test ./...", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_go_vet() {
        assert_eq!(evaluate(&make_input("go vet ./...", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_make() {
        assert_eq!(evaluate(&make_input("make", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_make_test() {
        assert_eq!(evaluate(&make_input("make test", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_curl() {
        assert_eq!(evaluate(&make_input("curl https://example.com", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    // ===== gh (GitHub CLI) read-only → allow =====

    #[test]
    fn test_gh_pr_view() {
        assert_eq!(evaluate(&make_input("gh pr view 123", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_pr_list() {
        assert_eq!(evaluate(&make_input("gh pr list", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_pr_diff() {
        assert_eq!(evaluate(&make_input("gh pr diff 123", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_pr_checks() {
        assert_eq!(evaluate(&make_input("gh pr checks 123", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_pr_status() {
        assert_eq!(evaluate(&make_input("gh pr status", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_issue_view() {
        assert_eq!(evaluate(&make_input("gh issue view 456", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_issue_list() {
        assert_eq!(evaluate(&make_input("gh issue list", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_run_view() {
        assert_eq!(evaluate(&make_input("gh run view 789", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_run_list() {
        assert_eq!(evaluate(&make_input("gh run list", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_repo_view() {
        assert_eq!(evaluate(&make_input("gh repo view owner/repo", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_release_list() {
        assert_eq!(evaluate(&make_input("gh release list", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_release_view() {
        assert_eq!(evaluate(&make_input("gh release view v1.0", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_api() {
        assert_eq!(evaluate(&make_input("gh api repos/owner/repo/pulls/123/comments", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_status() {
        assert_eq!(evaluate(&make_input("gh status", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_search() {
        assert_eq!(evaluate(&make_input("gh search issues something", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_workflow_list() {
        assert_eq!(evaluate(&make_input("gh workflow list", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_workflow_view() {
        assert_eq!(evaluate(&make_input("gh workflow view ci.yml", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_label_list() {
        assert_eq!(evaluate(&make_input("gh label list", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_gh_project_list() {
        assert_eq!(evaluate(&make_input("gh project list", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    // ===== gh mutating → fall through =====

    #[test]
    fn test_gh_pr_create() {
        assert_eq!(evaluate(&make_input("gh pr create --title test", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_pr_merge() {
        assert_eq!(evaluate(&make_input("gh pr merge 123", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_pr_close() {
        assert_eq!(evaluate(&make_input("gh pr close 123", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_issue_create() {
        assert_eq!(evaluate(&make_input("gh issue create", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_repo_create() {
        assert_eq!(evaluate(&make_input("gh repo create test-repo", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_repo_delete() {
        assert_eq!(evaluate(&make_input("gh repo delete owner/repo", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_run_rerun() {
        assert_eq!(evaluate(&make_input("gh run rerun 789", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_bare() {
        assert_eq!(evaluate(&make_input("gh", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_pr_bare() {
        assert_eq!(evaluate(&make_input("gh pr", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_pr_edit() {
        assert_eq!(evaluate(&make_input("gh pr edit 123", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_gh_pr_view_piped() {
        assert_eq!(evaluate(&make_input("gh pr view 123 | head -20", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    // ===== npx/uvx transparent wrapper → allow =====

    #[test]
    fn test_npx_tsc() {
        assert_eq!(evaluate(&make_input("npx tsc --noEmit", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_npx_eslint() {
        assert_eq!(evaluate(&make_input("npx eslint src/", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_npx_prettier() {
        assert_eq!(evaluate(&make_input("npx prettier --check .", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_uvx_ruff() {
        assert_eq!(evaluate(&make_input("uvx ruff check .", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_uvx_mypy() {
        assert_eq!(evaluate(&make_input("uvx mypy src/", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_pnpx_tsc() {
        assert_eq!(evaluate(&make_input("pnpx tsc", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_bunx_biome() {
        assert_eq!(evaluate(&make_input("bunx biome check .", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_npx_yes_tsc() {
        assert_eq!(evaluate(&make_input("npx --yes tsc", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    // ===== npx/uvx with unsafe → fall through =====

    #[test]
    fn test_npx_rimraf() {
        assert_eq!(evaluate(&make_input("npx rimraf dist", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_npx_unknown() {
        assert_eq!(evaluate(&make_input("npx unknown-tool", cwd())), Decision::Abstain);
    }

    // ===== cd tracking =====

    #[test]
    fn test_cd_workspace_ls() {
        assert_eq!(evaluate(&make_input("cd /Users/araftery/workspace && ls", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_cd_workspace_cat() {
        assert_eq!(evaluate(&make_input("cd /Users/araftery/workspace && cat foo.txt", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    #[test]
    fn test_cd_tmp_cat_env() {
        assert_eq!(evaluate(&make_input("cd /tmp && cat .env", cwd())), Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string()));
    }

    #[test]
    fn test_cd_up_ls() {
        assert_eq!(evaluate(&make_input("cd .. && cd .. && ls", cwd())), Decision::Allow("Safe read-only/build command".to_string()));
    }

    // ===== Package manager state-changing → fall through =====

    #[test]
    fn test_npm_install() {
        assert_eq!(evaluate(&make_input("npm install", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_npm_publish() {
        assert_eq!(evaluate(&make_input("npm publish", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_yarn_add() {
        assert_eq!(evaluate(&make_input("yarn add react", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_pip_install() {
        assert_eq!(evaluate(&make_input("pip install requests", cwd())), Decision::Abstain);
    }

    // ===== Edge cases =====

    #[test]
    fn test_empty_command() {
        assert_eq!(evaluate(&make_input("", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_whitespace_only() {
        assert_eq!(evaluate(&make_input("   ", cwd())), Decision::Abstain);
    }

    // ===== Quoted pipe characters should not split =====

    #[test]
    fn test_grep_with_escaped_pipes_in_double_quotes() {
        // grep -r "PORT\|port\|streamable\|transport" path 2>/dev/null | head -20
        // The \| inside double quotes are grep alternation, not shell pipes
        assert_eq!(
            evaluate(&make_input(
                r#"grep -r "PORT\|port\|streamable\|transport" /Users/araftery/workspace/project/README* 2>/dev/null | head -20"#,
                cwd()
            )),
            Decision::Allow("Safe read-only/build command".to_string())
        );
    }

    #[test]
    fn test_echo_with_pipe_in_double_quotes() {
        assert_eq!(
            evaluate(&make_input(r#"echo "hello | world""#, cwd())),
            Decision::Allow("Safe read-only/build command".to_string())
        );
    }

    #[test]
    fn test_echo_with_pipe_in_single_quotes() {
        assert_eq!(
            evaluate(&make_input("echo 'hello | world'", cwd())),
            Decision::Allow("Safe read-only/build command".to_string())
        );
    }

    #[test]
    fn test_grep_with_pipe_in_single_quotes() {
        assert_eq!(
            evaluate(&make_input("grep 'foo|bar' file.txt", cwd())),
            Decision::Allow("Safe read-only/build command".to_string())
        );
    }

    #[test]
    fn test_real_pipe_after_quoted_pipe() {
        // Quoted pipe should not split, but the real pipe after should
        assert_eq!(
            evaluate(&make_input(r#"grep "foo|bar" file.txt | head -5"#, cwd())),
            Decision::Allow("Safe read-only/build command".to_string())
        );
    }

    #[test]
    fn test_quoted_semicolon_should_not_split() {
        assert_eq!(
            evaluate(&make_input(r#"echo "hello; world""#, cwd())),
            Decision::Allow("Safe read-only/build command".to_string())
        );
    }

    #[test]
    fn test_quoted_ampersand_should_not_split() {
        assert_eq!(
            evaluate(&make_input(r#"echo "foo && bar""#, cwd())),
            Decision::Allow("Safe read-only/build command".to_string())
        );
    }

    #[test]
    fn test_mixed_quotes_and_real_operators() {
        // Real && between two commands, with quoted | inside grep pattern
        assert_eq!(
            evaluate(&make_input(r#"grep "foo|bar" file.txt && echo done"#, cwd())),
            Decision::Allow("Safe read-only/build command".to_string())
        );
    }

    #[test]
    fn test_secrets_in_quotes_still_detected() {
        // Quoting shouldn't let secrets through — the args still reference .env
        assert_eq!(
            evaluate(&make_input(r#"cat ".env""#, cwd())),
            Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }

    // ===== Performance =====

    #[test]
    fn test_perf_simple_command() {
        let input = make_input("ls -la", cwd());
        let start = std::time::Instant::now();
        let iterations = 10_000;
        for _ in 0..iterations {
            std::hint::black_box(evaluate(&input));
        }
        let elapsed = start.elapsed();
        let per_call = elapsed / iterations;
        eprintln!("simple command: {:?}/call ({iterations} iterations in {elapsed:?})", per_call);
        // Budget: < 100µs/call (generous for debug builds; release is ~0.1µs)
        assert!(per_call.as_micros() < 100, "simple command too slow: {per_call:?}/call");
    }

    #[test]
    fn test_perf_compound_with_quotes() {
        let input = make_input(
            r#"grep -r "PORT\|port\|streamable\|transport" /Users/araftery/workspace/project/README* 2>/dev/null | head -20"#,
            cwd(),
        );
        let start = std::time::Instant::now();
        let iterations = 10_000;
        for _ in 0..iterations {
            std::hint::black_box(evaluate(&input));
        }
        let elapsed = start.elapsed();
        let per_call = elapsed / iterations;
        eprintln!("compound+quotes: {:?}/call ({iterations} iterations in {elapsed:?})", per_call);
        assert!(per_call.as_micros() < 100, "compound+quotes too slow: {per_call:?}/call");
    }

    #[test]
    fn test_perf_long_chain() {
        let input = make_input(
            "ls && git status && cat foo.txt | grep bar | sort | uniq && echo done",
            cwd(),
        );
        let start = std::time::Instant::now();
        let iterations = 10_000;
        for _ in 0..iterations {
            std::hint::black_box(evaluate(&input));
        }
        let elapsed = start.elapsed();
        let per_call = elapsed / iterations;
        eprintln!("long chain: {:?}/call ({iterations} iterations in {elapsed:?})", per_call);
        assert!(per_call.as_micros() < 100, "long chain too slow: {per_call:?}/call");
    }
}
