use sftp_core::{register_feature, CoreError};

pub fn run(args: &[String]) -> Result<(), CoreError> {
	if args.is_empty() {
		return Err(CoreError::InvalidArgs("transfer requires at least one argument".into()));
	}
	Ok(())
}

fn handler(args: &[String]) -> Result<(), CoreError> {
	run(args)
}

/// Register this feature with the core registry.
pub fn register() {
	register_feature("transfer", handler);
}
