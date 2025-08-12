//! Authentication feature crate.
//! Provides `run` to execute the feature and `register` to register it with core.

use sftp_core::{register_feature, CoreError};

/// Execute the `auth` feature.
///
/// Returns `InvalidArgs` if no arguments are provided.
pub fn run(args: &[String]) -> Result<(), CoreError> {
	// Placeholder auth logic; in real crate, validate credentials, etc.
	if args.is_empty() {
		return Err(CoreError::InvalidArgs("auth requires at least one argument".into()));
	}
	Ok(())
}

/// Thin adapter used by the registry to call `run`.
fn handler(args: &[String]) -> Result<(), CoreError> {
	run(args)
}

/// Register this feature with the core registry.
pub fn register() {
	register_feature("auth", handler);
}
