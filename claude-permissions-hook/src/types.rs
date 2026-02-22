use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Raw hook input from Claude Code via stdin
#[derive(Debug, Deserialize)]
pub struct HookInput {
    /// The tool being invoked (e.g. "Bash", "Read", "Glob", "Grep", "Edit", "Write", "WebFetch", "WebSearch")
    pub tool_name: Option<String>,
    /// The tool's input parameters
    pub tool_input: Option<HashMap<String, serde_json::Value>>,
    /// Current working directory
    pub cwd: Option<String>,
}

impl HookInput {
    /// Get a string field from tool_input
    pub fn get_input_str(&self, key: &str) -> Option<&str> {
        self.tool_input
            .as_ref()?
            .get(key)?
            .as_str()
    }
}

/// The decision a hook can make
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow(String),
    Deny(String),
    Ask(String),
    Abstain, // fall through — no output, exit 0
}

/// Output format Claude Code expects
#[derive(Debug, Serialize)]
pub struct HookOutput {
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: HookSpecificOutput,
}

#[derive(Debug, Serialize)]
pub struct HookSpecificOutput {
    #[serde(rename = "hookEventName")]
    pub hook_event_name: String,
    #[serde(rename = "permissionDecision")]
    pub permission_decision: String,
    #[serde(rename = "permissionDecisionReason")]
    pub permission_decision_reason: String,
}

impl HookOutput {
    pub fn new(decision: &str, reason: &str) -> Self {
        HookOutput {
            hook_specific_output: HookSpecificOutput {
                hook_event_name: "PreToolUse".to_string(),
                permission_decision: decision.to_string(),
                permission_decision_reason: reason.to_string(),
            },
        }
    }
}
