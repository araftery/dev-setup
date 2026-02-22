use std::path::{Path, PathBuf};

/// Directories that are always allowed for read access
const ALLOWED_DIRS: &[&str] = &[
    "/Users/araftery/workspace",
    "/etc/ig",
    "/Users/araftery/.config/zl",
];

/// Normalize a path: resolve `.` and `..` components without requiring the path to exist.
/// Always uses manual component-based resolution for predictability and security.
pub fn normalize_path(path: &str, cwd: &str) -> PathBuf {
    let p = Path::new(path);

    // Make absolute
    let absolute = if p.is_absolute() {
        PathBuf::from(path)
    } else {
        PathBuf::from(cwd).join(path)
    };

    // Manual resolution — predictable and works for non-existent paths
    let mut components = Vec::new();
    for component in absolute.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            other => {
                components.push(other);
            }
        }
    }

    components.iter().collect()
}

/// Check if a path is within CWD or one of the allowed directories
pub fn is_in_allowed_dir(path: &Path, cwd: &str) -> bool {
    let path_str = path.to_string_lossy();

    // Check CWD
    if !cwd.is_empty() && path_str.starts_with(cwd) {
        return true;
    }

    // Check allowed directories
    for dir in ALLOWED_DIRS {
        if path_str.starts_with(dir) {
            return true;
        }
    }

    false
}

/// Check if a filename is a secrets file (.env, .env.*, .dev.vars)
pub fn is_secrets_file(path: &str) -> bool {
    let basename = Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    if basename == ".env" || basename == ".dev.vars" {
        return true;
    }

    if basename.starts_with(".env.") {
        return true;
    }

    false
}

/// Check if a glob pattern targets secrets files
pub fn glob_targets_secrets(pattern: &str) -> bool {
    // Check if the pattern itself would match secrets files
    let lower = pattern.to_lowercase();
    lower.contains(".env") || lower.contains(".dev.vars")
}

/// Strip matching surrounding quotes (single or double) from a string
fn strip_quotes(s: &str) -> &str {
    if s.len() >= 2 {
        if (s.starts_with('"') && s.ends_with('"'))
            || (s.starts_with('\'') && s.ends_with('\''))
        {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Check if any argument in a list of tokens references a secrets file
pub fn args_reference_secrets(args: &[&str]) -> bool {
    for arg in args {
        let stripped = strip_quotes(arg);
        // Skip flags
        if stripped.starts_with('-') {
            continue;
        }
        if is_secrets_file(stripped) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_absolute() {
        let result = normalize_path("/Users/araftery/workspace/src/index.ts", "/tmp");
        assert_eq!(result, PathBuf::from("/Users/araftery/workspace/src/index.ts"));
    }

    #[test]
    fn test_normalize_path_relative() {
        let result = normalize_path("src/index.ts", "/Users/araftery/workspace");
        assert_eq!(result, PathBuf::from("/Users/araftery/workspace/src/index.ts"));
    }

    #[test]
    fn test_normalize_path_traversal() {
        // ../../etc/passwd from /Users/araftery/workspace/project → /Users/araftery/etc/passwd
        let result = normalize_path("../../etc/passwd", "/Users/araftery/workspace/project");
        assert_eq!(result, PathBuf::from("/Users/araftery/etc/passwd"));

        // To actually reach /etc/passwd from this CWD, need 4 levels up
        let result = normalize_path("../../../../etc/passwd", "/Users/araftery/workspace/project");
        assert_eq!(result, PathBuf::from("/etc/passwd"));
    }

    #[test]
    fn test_is_in_allowed_dir() {
        assert!(is_in_allowed_dir(Path::new("/Users/araftery/workspace/foo"), "/Users/araftery/workspace"));
        assert!(is_in_allowed_dir(Path::new("/etc/ig/config.yaml"), "/tmp"));
        assert!(!is_in_allowed_dir(Path::new("/etc/passwd"), "/Users/araftery/workspace"));
        assert!(!is_in_allowed_dir(Path::new("/var/log/system.log"), "/Users/araftery/workspace"));
    }

    #[test]
    fn test_is_secrets_file() {
        assert!(is_secrets_file(".env"));
        assert!(is_secrets_file(".env.local"));
        assert!(is_secrets_file(".env.prod"));
        assert!(is_secrets_file(".dev.vars"));
        assert!(is_secrets_file("/Users/araftery/workspace/.env"));
        assert!(is_secrets_file("/foo/bar/.env.local"));
        assert!(!is_secrets_file("index.ts"));
        assert!(!is_secrets_file(".envrc"));
        assert!(!is_secrets_file("env.txt"));
    }

    #[test]
    fn test_glob_targets_secrets() {
        assert!(glob_targets_secrets("**/.env*"));
        assert!(glob_targets_secrets("*.dev.vars"));
        assert!(glob_targets_secrets(".env"));
        assert!(!glob_targets_secrets("**/*.ts"));
        assert!(!glob_targets_secrets("src/**/*.rs"));
    }
}
