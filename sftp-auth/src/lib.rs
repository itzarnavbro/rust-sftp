use sftp_core::{register_feature, CoreError};

pub fn run(args: &[String]) -> Result<(), CoreError> {
	// Placeholder auth logic; in real crate, validate credentials, etc.
	if args.is_empty() {
		return Err(CoreError::InvalidArgs("auth requires at least one argument".into()));
	}
	Ok(())
}

fn handler(args: &[String]) -> Result<(), CoreError> {
	run(args)
}

/// Register this feature with the core registry.
pub fn register() {
	register_feature("auth", handler);
}
