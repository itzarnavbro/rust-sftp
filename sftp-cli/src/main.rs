//! sftp-cli: binary crate that dispatches commands to feature crates.
use std::env;

use sftp_core::list_features;
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
		// Naya remote SFTP path: network transfer over SSH
		"net" | "sftp" => handle_net(&args, verbose),
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
	// Hin-glish: Neeche network SFTP examples add kiye hain
	println!(
		"\nNetwork SFTP examples:\n  {bin} sftp upload --host 192.168.1.10 --port 22 --user alice --key C:\\id_ed25519 C:\\src.txt /home/alice/dest.txt\n  {bin} sftp download --host 192.168.1.10 --user alice --password secret /home/alice/src.txt C:\\dest.txt\n  {bin} sftp ls --host 192.168.1.10 --user alice --key C:\\id_ed25519 /home/alice\n  # Known hosts strict mode (recommended)\n  {bin} sftp ls --host 192.168.1.10 --user alice --key C:\\id_ed25519 --known-hosts C:\\Users\\you\\.ssh\\known_hosts --strict /home/alice\n",
		bin = env!("CARGO_PKG_NAME")
	);
}

/// Print a formatted error and a hint to use `--help`.
fn print_error(err: &str) {
	eprintln!("Error: {}", err);
	eprintln!("Run with --help to see usage.");
}

// Remote SFTP handler using sftp-net (ssh2)
fn handle_net(args: &[String], verbose: bool) -> i32 {
	// Hin-glish: Simple flag parser banate hain (no external clap)
	if args.is_empty() {
		print_error("sftp requires a subcommand: upload|download|ls");
		return 2;
	}
	let sub = &args[0];

	// Defaults
	let mut host = String::new();
	let mut port: u16 = 22;
	let mut username = String::new();
	let mut password: Option<String> = None;
	let mut key_path: Option<String> = None;
	let mut key_pub: Option<String> = None;
	let mut key_pass: Option<String> = None;
	let mut known_hosts: Option<String> = None;
	let mut strict = false;
	let mut accept_new = false;

	// Collect positional operands after flags
	let mut rest: Vec<String> = Vec::new();

	let mut i = 1;
	while i < args.len() {
		let a = &args[i];
		match a.as_str() {
			"--host" => { i+=1; host = args.get(i).cloned().unwrap_or_default(); }
			"--port" => { i+=1; port = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(22); }
			"--user" | "--username" => { i+=1; username = args.get(i).cloned().unwrap_or_default(); }
			"--password" => { i+=1; password = args.get(i).cloned(); }
			"--key" | "--identity-file" => { i+=1; key_path = args.get(i).cloned(); }
			"--key-pass" | "--passphrase" => { i+=1; key_pass = args.get(i).cloned(); }
			"--key-pub" => { i+=1; key_pub = args.get(i).cloned(); }
			"--known-hosts" => { i+=1; known_hosts = args.get(i).cloned(); }
			"--strict" => { strict = true; }
			"--accept-new" => { accept_new = true; }
			_ => rest.push(a.clone()),
		}
		i += 1;
	}

	if verbose {
		eprintln!("[verbose][sftp] host={host} port={port} user={username} pass?={} key?={} known_hosts?={} strict={} accept_new={} rest={:?}",
			password.is_some(), key_path.is_some(), known_hosts.is_some(), strict, accept_new, rest);
	}

	if host.is_empty() || username.is_empty() {
		print_error("--host and --user are required");
		return 2;
	}

	// Host key policy
	let policy = if strict { sftp_net::HostKeyPolicy::Strict } else if accept_new { sftp_net::HostKeyPolicy::AcceptNew } else { sftp_net::HostKeyPolicy::InsecureIgnore };
	let kh_path = known_hosts.map(std::path::PathBuf::from);

	// Auth select karo
	let auth = if let Some(k) = key_path.clone() {
		sftp_net::Auth::KeyFile { username: username.clone(), private_key: k.into(), passphrase: key_pass.clone(), public_key: key_pub.clone().map(Into::into) }
	} else if let Some(pw) = password.clone() {
		sftp_net::Auth::Password { username: username.clone(), password: pw }
	} else {
		print_error("either --key or --password must be provided");
		return 2;
	};

	let cfg = sftp_net::SshConfig { host: host.clone(), port, auth, known_hosts: kh_path, hostkey_policy: policy, timeout_ms: Some(30_000) };

	// Connect once per command
	let client = match sftp_net::SftpClient::connect(&cfg) {
		Ok(c) => c,
		Err(e) => { print_error(&format!("connect failed: {e}")); return 1; }
	};

	match sub.as_str() {
		"upload" => {
			if rest.len() != 2 { print_error("sftp upload requires <local_src> <remote_dest>"); return 2; }
			let local = &rest[0];
			let remote = &rest[1];
			if verbose { eprintln!("[verbose][sftp] upload {local} -> {remote}"); }
			match client.upload_file(local, remote) {
				Ok(()) => { println!("Upload OK"); 0 }
				Err(e) => { print_error(&format!("upload failed: {e}")); 1 }
			}
		}
		"download" => {
			if rest.len() != 2 { print_error("sftp download requires <remote_src> <local_dest>"); return 2; }
			let remote = &rest[0];
			let local = &rest[1];
			if verbose { eprintln!("[verbose][sftp] download {remote} -> {local}"); }
			match client.download_file(remote, local) {
				Ok(()) => { println!("Download OK"); 0 }
				Err(e) => { print_error(&format!("download failed: {e}")); 1 }
			}
		}
		"ls" => {
			if rest.len() != 1 { print_error("sftp ls requires <remote_dir>"); return 2; }
			let dir = &rest[0];
			if verbose { eprintln!("[verbose][sftp] ls {dir}"); }
			match client.list_dir(dir) {
				Ok(files) => { for f in files { println!("{}", f); } 0 }
				Err(e) => { print_error(&format!("ls failed: {e}")); 1 }
			}
		}
		_ => { print_error("unknown sftp subcommand (use upload|download|ls)"); 2 }
	}
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
