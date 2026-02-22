use crate::paths;
use crate::types::{Decision, HookInput};

/// Evaluate an Edit or Write tool invocation
pub fn evaluate(input: &HookInput) -> Decision {
    let cwd = input.cwd.as_deref().unwrap_or("");

    let file_path = match input.get_input_str("file_path") {
        Some(p) => p,
        None => return Decision::Abstain,
    };

    let normalized = paths::normalize_path(file_path, cwd);

    // Deny writes to secrets files
    if paths::is_secrets_file(&normalized.to_string_lossy()) {
        return Decision::Deny("Writing to secrets files (.env, .dev.vars) is blocked".to_string());
    }

    // Fall through for all other writes — Claude's default approval handles the rest
    Decision::Abstain
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_write_input(tool: &str, file_path: &str, cwd: &str) -> HookInput {
        let mut tool_input = HashMap::new();
        tool_input.insert("file_path".to_string(), serde_json::json!(file_path));
        HookInput {
            tool_name: Some(tool.to_string()),
            tool_input: Some(tool_input),
            cwd: Some(cwd.to_string()),
        }
    }

    fn cwd() -> &'static str {
        "/Users/araftery/workspace/project"
    }

    #[test]
    fn test_write_env() {
        assert_eq!(
            evaluate(&make_write_input("Write", "/Users/araftery/workspace/.env", cwd())),
            Decision::Deny("Writing to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }

    #[test]
    fn test_write_env_prod() {
        assert_eq!(
            evaluate(&make_write_input("Write", "/Users/araftery/workspace/.env.prod", cwd())),
            Decision::Deny("Writing to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }

    #[test]
    fn test_edit_dev_vars() {
        assert_eq!(
            evaluate(&make_write_input("Edit", "/Users/araftery/workspace/.dev.vars", cwd())),
            Decision::Deny("Writing to secrets files (.env, .dev.vars) is blocked".to_string())
        );
    }

    #[test]
    fn test_write_normal_file() {
        assert_eq!(
            evaluate(&make_write_input("Write", "/Users/araftery/workspace/src/index.ts", cwd())),
            Decision::Abstain
        );
    }

    #[test]
    fn test_edit_config() {
        assert_eq!(
            evaluate(&make_write_input("Edit", "/etc/ig/config.yaml", cwd())),
            Decision::Abstain
        );
    }
}
