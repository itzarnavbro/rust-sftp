//! sftp-cli: binary crate that registers features and dispatches commands.
use std::env;

use sftp_core::{list_features, process_input, CoreError};

/// Entry point: registers features, handles flags and dispatches to core.
fn main() {
	// Register features at startup
	sftp_auth::register();
	sftp_transfer::register();

	// Gather raw args once for flag handling
	let raw_args: Vec<String> = env::args().skip(1).collect();

	// Flags
	if raw_args.is_empty() || raw_args.iter().any(|a| a == "--help" || a == "-h") {
		print_help();
		return;
	}
	if raw_args.iter().any(|a| a == "--version" || a == "-V") {
		println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
		return;
	}
	let verbose = raw_args.iter().any(|a| a == "--verbose" || a == "-v");
	if verbose {
		eprintln!("[verbose] args: {:?}", raw_args);
	}

	// Parse feature + arguments (ignoring known flags)
	let (feature, args) = parse_args();
	if feature.is_empty() {
		print_help();
		return;
	}

	// Build a raw command line preserving spaces via quoting when needed
	let mut cmd = String::new();
	cmd.push_str(&feature);
	for a in &args {
		cmd.push(' ');
		if a.chars().any(|c| c.is_whitespace()) {
			let escaped = a.replace('"', "\\\"");
			cmd.push('"');
			cmd.push_str(&escaped);
			cmd.push('"');
		} else {
			cmd.push_str(a);
		}
	}

	if verbose {
		eprintln!("[verbose] dispatch: {}", cmd);
	}

	match process_input(&cmd) {
		Ok(out) => {
			println!("{}", out.message);
			std::process::exit(out.code);
		}
		Err(err) => {
			print_error(&err);
			std::process::exit(1);
		}
	}
}

// Private helpers

/// Parse CLI arguments into `(feature, args)` ignoring known flags.
fn parse_args() -> (String, Vec<String>) {
	let mut args = Vec::new();
	for a in env::args().skip(1) {
		match a.as_str() {
			"--help" | "-h" | "--version" | "-V" | "--verbose" | "-v" => {
				// skip flags here; handled in main
			}
			_ => args.push(a),
		}
	}

	if args.is_empty() {
		return (String::new(), Vec::new());
	}
	let feature = args.remove(0);
	(feature, args)
}

/// Print CLI usage, available features and examples.
fn print_help() {
	let features = list_features();
	println!(
		"Usage:\n  {} [FLAGS] <feature> [args...]\n\nFeatures:\n  {}\n\nFlags:\n  -h, --help       Show help\n  -V, --version    Show version\n  -v, --verbose    Enable debug output\n\nExamples:\n  {bin} auth <user>\n  {bin} transfer <file>\n",
		env!("CARGO_PKG_NAME"),
		if features.is_empty() {
			"(none)".to_string()
		} else {
			features.join(", ")
		},
		bin = env!("CARGO_PKG_NAME")
	);
}

/// Print a formatted error and a hint to use `--help`.
fn print_error(err: &CoreError) {
	eprintln!("Error: {}", err);
	eprintln!("Run with --help to see usage.");
}
