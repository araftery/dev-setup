use crate::types::{Decision, HookInput};

/// Evaluate a WebFetch or WebSearch tool invocation — always auto-allow
pub fn evaluate(_input: &HookInput) -> Decision {
    Decision::Allow("Web operations auto-approved".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_web_input(tool: &str) -> HookInput {
        let mut tool_input = HashMap::new();
        if tool == "WebSearch" {
            tool_input.insert("query".to_string(), serde_json::json!("rust programming"));
        } else {
            tool_input.insert("url".to_string(), serde_json::json!("https://example.com"));
        }
        HookInput {
            tool_name: Some(tool.to_string()),
            tool_input: Some(tool_input),
            cwd: Some("/tmp".to_string()),
        }
    }

    #[test]
    fn test_web_search() {
        assert_eq!(
            evaluate(&make_web_input("WebSearch")),
            Decision::Allow("Web operations auto-approved".to_string())
        );
    }

    #[test]
    fn test_web_fetch() {
        assert_eq!(
            evaluate(&make_web_input("WebFetch")),
            Decision::Allow("Web operations auto-approved".to_string())
        );
    }
}
