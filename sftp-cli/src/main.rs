//! sftp-cli: binary crate that dispatches commands to feature crates.
use std::env;

use sftp_core::{list_features, CoreError};
use std::path::Path;

/// Entry point: handles flags and dispatches to feature crates.
fn main() {
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

	// Dispatch directly to feature crates (keeping their APIs unchanged)
	let code = match feature.as_str() {
		"auth" => handle_auth(&args, verbose),
		"transfer" => handle_transfer(&args, verbose),
		other => {
			eprintln!("Unknown feature: {}", other);
			print_help();
			1
		}
	};
	std::process::exit(code);
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
		"Usage:\n  {} [FLAGS] <feature> [args...]\n\nFeatures:\n  {}\n\nFlags:\n  -h, --help       Show help\n  -V, --version    Show version\n  -v, --verbose    Enable debug output\n\nAuth examples:\n  {bin} auth load_keys <authorized_keys_path>\n\nTransfer examples:\n  {bin} transfer upload <src> <dest>\n  {bin} transfer download <src> <dest>\n  {bin} transfer ls <dir>\n",
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
fn print_error(err: &str) {
	eprintln!("Error: {}", err);
	eprintln!("Run with --help to see usage.");
}

// Feature handlers (no changes to feature crate APIs)

fn handle_auth(args: &[String], verbose: bool) -> i32 {
	if args.is_empty() {
		print_help();
		return 1;
	}
	match args[0].as_str() {
		"load_keys" => {
			if args.len() != 2 {
				print_error("auth load_keys requires <authorized_keys_path>");
				return 2;
			}
			let path = std::path::Path::new(&args[1]);
			if verbose {
				eprintln!("[verbose][auth] load_keys {:?}", path);
			}
			match sftp_auth::load_keys_from_file(path) {
				Ok(users) => {
					println!("Loaded {} users", users.len());
					0
				}
				Err(_e) => {
					// Avoid forcing feature crate error types; print generic error
					print_error("failed to load keys");
					1
				}
			}
		}
		other => {
			print_error(&format!("unknown auth subcommand: {}", other));
			1
		}
	}
}

fn handle_transfer(args: &[String], verbose: bool) -> i32 {
	if args.is_empty() {
		print_help();
		return 1;
	}
	match args[0].as_str() {
		"upload" => {
			if args.len() != 3 {
				print_error("transfer upload requires <src> <dest>");
				return 2;
			}
			let src = Path::new(&args[1]);
			let dest = Path::new(&args[2]);
			if verbose { eprintln!("[verbose][transfer] upload {:?} -> {:?}", src, dest); }
			match sftp_transfer::TransferManager::upload_file(src, dest) {
				Ok(_progress) => {
					println!("Upload completed");
					0
				}
				Err(_e) => {
					print_error("upload failed");
					1
				}
			}
		}
		"download" => {
			if args.len() != 3 {
				print_error("transfer download requires <src> <dest>");
				return 2;
			}
			let src = Path::new(&args[1]);
			let dest = Path::new(&args[2]);
			if verbose { eprintln!("[verbose][transfer] download {:?} -> {:?}", src, dest); }
			match sftp_transfer::TransferManager::download_file(src, dest) {
				Ok(()) => { println!("Download completed"); 0 }
				Err(_e) => { print_error("download failed"); 1 }
			}
		}
		"ls" => {
			if args.len() != 2 {
				print_error("transfer ls requires <dir>");
				return 2;
			}
			let dir = Path::new(&args[1]);
			if verbose { eprintln!("[verbose][transfer] ls {:?}", dir); }
			match sftp_transfer::TransferManager::list_files(dir) {
				Ok(files) => { for f in files { println!("{}", f); } 0 }
				Err(_e) => { print_error("ls failed"); 1 }
			}
		}
		other => {
			print_error(&format!("unknown transfer subcommand: {}", other));
			1
		}
	}
}
