//! Core crate: feature registry, command parsing, and dispatch.
//!
//! Feature crates call `register_feature` at startup to register their handler
//! functions, and the CLI calls `process_input` to parse and dispatch commands.
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::{Mutex, OnceLock};

/// Result type returned by `process_input`.
///
/// - `message`: human-friendly message suitable for CLI output
/// - `code`: process exit code (0 for success, non-zero for error)
#[derive(Debug, Clone)]
pub struct Output {
    pub message: String,
    pub code: i32,
}

/// Unified error type for the core to keep error handling consistent across
/// feature crates and the CLI.
#[derive(Debug, Clone)]
pub enum CoreError {
    UnknownFeature(String),
    InvalidArgs(String),
    ExecutionFailed(String),
}

impl Display for CoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::UnknownFeature(name) => write!(f, "Unknown feature: {}", name),
            CoreError::InvalidArgs(msg) => write!(f, "Invalid arguments: {}", msg),
            CoreError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
        }
    }
}

impl std::error::Error for CoreError {}

// Feature registry
type FeatureFn = fn(&[String]) -> Result<(), CoreError>;

static REGISTRY: OnceLock<Mutex<HashMap<String, FeatureFn>>> = OnceLock::new();

/// Internal accessor for the global feature registry.
fn registry() -> &'static Mutex<HashMap<String, FeatureFn>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register a feature handler by name (to be called by feature crates at startup)
pub fn register_feature(name: &str, handler: FeatureFn) {
    let mut map = registry().lock().expect("registry poisoned");
    map.insert(name.to_string(), handler);
}

// Public API

/// Take a raw command string from the CLI, parse it, and run the correct feature.
pub fn process_input(input: &str) -> Result<Output, CoreError> {
    let (feature, args) = parse_command(input);
    if feature.trim().is_empty() {
        return Err(CoreError::InvalidArgs(
            "no feature provided; try one of: ".to_string() + &list_features().join(", "),
        ));
    }
    execute_feature(&feature, &args)?;
    Ok(Output {
        message: format!("{} executed successfully", feature),
        code: 0,
    })
}

/// Central dispatcher to call the matching feature crate.
pub fn execute_feature(feature_name: &str, args: &[String]) -> Result<(), CoreError> {
    let map = registry().lock().expect("registry poisoned");
    if let Some(handler) = map.get(feature_name) {
        handler(args)
    } else {
        Err(CoreError::UnknownFeature(feature_name.to_string()))
    }
}

/// Return all available feature names (for CLI help/autocomplete).
pub fn list_features() -> Vec<String> {
    let map = registry().lock().expect("registry poisoned");
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort();
    keys
}

// Private helpers

/// Splits raw CLI input into a (feature_name, args) tuple.
fn parse_command(input: &str) -> (String, Vec<String>) {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return (String::new(), Vec::new());
    }
    let feature = tokens[0].clone();
    let args = tokens[1..].to_vec();
    (feature, args)
}

fn tokenize(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double => {
                if in_single {
                    in_single = false;
                } else {
                    in_single = true;
                }
            }
            '"' if !in_single => {
                if in_double {
                    in_double = false;
                } else {
                    in_double = true;
                }
            }
            '\\' => {
                if let Some(next) = chars.next() {
                    buf.push(next);
                }
            }
            ch if ch.is_whitespace() && !in_single && !in_double => {
                if !buf.is_empty() {
                    out.push(std::mem::take(&mut buf));
                }
            }
            _ => buf.push(c),
        }
    }

    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

/// Checks if the feature is implemented/available.
#[allow(dead_code)]
fn feature_exists(name: &str) -> bool {
    let map = registry().lock().expect("registry poisoned");
    map.contains_key(name)
}

/// Turns internal errors into nice CLI messages.
#[allow(dead_code)]
fn format_error(err: CoreError) -> String {
    match err {
        CoreError::UnknownFeature(name) => format!(
            "Unknown feature: {}. Available: {}",
            name,
            list_features().join(", ")
        ),
        CoreError::InvalidArgs(msg) => format!("Invalid arguments: {}", msg),
        CoreError::ExecutionFailed(msg) => format!("Execution failed: {}", msg),
    }
}

// Optional: small self-check tests (only compiled with tests)
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_handler(args: &[String]) -> Result<(), CoreError> {
        if args.is_empty() {
            return Err(CoreError::InvalidArgs("expected at least 1 arg".into()));
        }
        Ok(())
    }

    #[test]
    fn registry_and_dispatch() {
        register_feature("sample", sample_handler);
        assert!(feature_exists("sample"));
        assert!(list_features().contains(&"sample".to_string()));
        assert!(execute_feature("sample", &["ok".into()]).is_ok());
        assert!(execute_feature("missing", &[]).is_err());
    }

    #[test]
    fn parse_and_process() {
        register_feature("echo", |_| Ok(()));
        let out = process_input("echo 'hello world'").unwrap();
        assert_eq!(out.code, 0);
    }
}

