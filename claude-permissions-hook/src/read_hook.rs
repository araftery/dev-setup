use crate::paths;
use crate::types::{Decision, HookInput};

/// Evaluate a Read, Glob, or Grep tool invocation
pub fn evaluate(input: &HookInput) -> Decision {
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    let cwd = input.cwd.as_deref().unwrap_or("");

    match tool_name {
        "Read" => evaluate_read(input, cwd),
        "Glob" => evaluate_glob(input, cwd),
        "Grep" => evaluate_grep(input, cwd),
        _ => Decision::Abstain,
    }
}

fn evaluate_read(input: &HookInput, cwd: &str) -> Decision {
    let file_path = match input.get_input_str("file_path") {
        Some(p) => p,
        None => return Decision::Abstain,
    };

    let normalized = paths::normalize_path(file_path, cwd);

    // Check secrets
    if paths::is_secrets_file(&normalized.to_string_lossy()) {
        return Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string());
    }

    // Check allowed directories
    if paths::is_in_allowed_dir(&normalized, cwd) {
        return Decision::Allow("File within allowed directories".to_string());
    }

    Decision::Abstain
}

fn evaluate_glob(input: &HookInput, cwd: &str) -> Decision {
    // Check pattern for secrets
    if let Some(pattern) = input.get_input_str("pattern") {
        if paths::glob_targets_secrets(pattern) {
            return Decision::Deny("Glob pattern targets secrets files".to_string());
        }
    }

    // Check path
    let path = input.get_input_str("path").unwrap_or(cwd);
    if path.is_empty() {
        // No path = uses CWD, check if CWD is allowed
        return Decision::Abstain;
    }

    let normalized = paths::normalize_path(path, cwd);

    if paths::is_in_allowed_dir(&normalized, cwd) {
        return Decision::Allow("Path within allowed directories".to_string());
    }

    Decision::Abstain
}

fn evaluate_grep(input: &HookInput, cwd: &str) -> Decision {
    let path = input.get_input_str("path").unwrap_or(cwd);
    if path.is_empty() {
        return Decision::Abstain;
    }

    let normalized = paths::normalize_path(path, cwd);

    // Check secrets
    if paths::is_secrets_file(&normalized.to_string_lossy()) {
        return Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string());
    }

    if paths::is_in_allowed_dir(&normalized, cwd) {
        return Decision::Allow("Path within allowed directories".to_string());
    }

    Decision::Abstain
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_read_input(file_path: &str, cwd: &str) -> HookInput {
        let mut tool_input = HashMap::new();
        tool_input.insert("file_path".to_string(), serde_json::json!(file_path));
        HookInput {
            tool_name: Some("Read".to_string()),
            tool_input: Some(tool_input),
            cwd: Some(cwd.to_string()),
        }
    }

    fn make_glob_input(pattern: &str, path: Option<&str>, cwd: &str) -> HookInput {
        let mut tool_input = HashMap::new();
        tool_input.insert("pattern".to_string(), serde_json::json!(pattern));
        if let Some(p) = path {
            tool_input.insert("path".to_string(), serde_json::json!(p));
        }
        HookInput {
            tool_name: Some("Glob".to_string()),
            tool_input: Some(tool_input),
            cwd: Some(cwd.to_string()),
        }
    }

    fn make_grep_input(pattern: &str, path: &str, cwd: &str) -> HookInput {
        let mut tool_input = HashMap::new();
        tool_input.insert("pattern".to_string(), serde_json::json!(pattern));
        tool_input.insert("path".to_string(), serde_json::json!(path));
        HookInput {
            tool_name: Some("Grep".to_string()),
            tool_input: Some(tool_input),
            cwd: Some(cwd.to_string()),
        }
    }

    fn cwd() -> &'static str {
        "/Users/araftery/workspace/project"
    }

    // ===== Allow within workspace =====

    #[test]
    fn test_read_workspace_file() {
        assert_eq!(
            evaluate(&make_read_input("/Users/araftery/workspace/src/index.ts", cwd())),
            Decision::Allow("File within allowed directories".to_string())
        );
    }

    #[test]
    fn test_read_etc_ig() {
        assert_eq!(
            evaluate(&make_read_input("/etc/ig/config.yaml", cwd())),
            Decision::Allow("File within allowed directories".to_string())
        );
    }

    #[test]
    fn test_glob_workspace() {
        assert_eq!(
            evaluate(&make_glob_input("**/*.ts", Some("/Users/araftery/workspace"), cwd())),
            Decision::Allow("Path within allowed directories".to_string())
        );
    }

    #[test]
    fn test_grep_etc_ig() {
        assert_eq!(
            evaluate(&make_grep_input("pattern", "/etc/ig", cwd())),
            Decision::Allow("Path within allowed directories".to_string())
        );
    }

    #[test]
    fn test_glob_no_path_cwd() {
        // Glob with no path uses CWD — CWD is within workspace → allow
        assert_eq!(
            evaluate(&make_glob_input("**/*.ts", None, cwd())),
            Decision::Allow("Path within allowed directories".to_string())
        );
    }

    // ===== Deny secrets =====

    #[test]
    fn test_read_env() {
        assert_eq!(
            evaluate(&make_read_input("/Users/araftery/workspace/.env", cwd())),
            Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }

    #[test]
    fn test_read_env_local() {
        assert_eq!(
            evaluate(&make_read_input("/Users/araftery/workspace/.env.local", cwd())),
            Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }

    #[test]
    fn test_read_env_prod() {
        assert_eq!(
            evaluate(&make_read_input("/Users/araftery/workspace/.env.prod", cwd())),
            Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }

    #[test]
    fn test_read_dev_vars() {
        assert_eq!(
            evaluate(&make_read_input("/etc/ig/.dev.vars", cwd())),
            Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }

    #[test]
    fn test_glob_env_pattern() {
        assert_eq!(
            evaluate(&make_glob_input("**/.env*", Some("/Users/araftery/workspace"), cwd())),
            Decision::Deny("Glob pattern targets secrets files".to_string())
        );
    }

    #[test]
    fn test_glob_dev_vars_pattern() {
        assert_eq!(
            evaluate(&make_glob_input("*.dev.vars", Some("/Users/araftery/workspace"), cwd())),
            Decision::Deny("Glob pattern targets secrets files".to_string())
        );
    }

    // ===== Outside allowed dirs → fall through =====

    #[test]
    fn test_read_etc_passwd() {
        assert_eq!(evaluate(&make_read_input("/etc/passwd", cwd())), Decision::Abstain);
    }

    #[test]
    fn test_read_var_log() {
        assert_eq!(evaluate(&make_read_input("/var/log/system.log", cwd())), Decision::Abstain);
    }

    // ===== Path traversal =====

    #[test]
    fn test_read_traversal_outside() {
        assert_eq!(
            evaluate(&make_read_input("/Users/araftery/workspace/../../etc/passwd", cwd())),
            Decision::Abstain
        );
    }

    #[test]
    fn test_read_traversal_secrets() {
        assert_eq!(
            evaluate(&make_read_input("./../../.env", "/Users/araftery/workspace/project")),
            Decision::Deny("Access to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }
}
