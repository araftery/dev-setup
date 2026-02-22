mod types;
mod paths;
mod bash_hook;
mod read_hook;
mod write_hook;
mod web_hook;

use std::io::Read;
use types::{Decision, HookInput, HookOutput};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let hook_type = args.get(1).map(|s| s.as_str()).unwrap_or("");

    // Read JSON from stdin
    let mut input_str = String::new();
    if std::io::stdin().read_to_string(&mut input_str).is_err() {
        // Can't read stdin — abstain
        return;
    }

    let input: HookInput = match serde_json::from_str(&input_str) {
        Ok(i) => i,
        Err(_) => {
            // Can't parse input — abstain
            return;
        }
    };

    let decision = match hook_type {
        "bash" => bash_hook::evaluate(&input),
        "read" => read_hook::evaluate(&input),
        "write" => write_hook::evaluate(&input),
        "web" => web_hook::evaluate(&input),
        _ => {
            eprintln!("Unknown hook type: {}", hook_type);
            Decision::Abstain
        }
    };

    match decision {
        Decision::Allow(reason) => {
            let output = HookOutput::new("allow", &reason);
            println!("{}", serde_json::to_string(&output).unwrap());
        }
        Decision::Deny(reason) => {
            let output = HookOutput::new("deny", &reason);
            println!("{}", serde_json::to_string(&output).unwrap());
        }
        Decision::Ask(reason) => {
            let output = HookOutput::new("ask", &reason);
            println!("{}", serde_json::to_string(&output).unwrap());
        }
        Decision::Abstain => {
            // No output — fall through to Claude's default permissions
        }
    }
}
