use std::env;

fn main() {
	// Register features at startup
	sftp_auth::register();
	sftp_transfer::register();

	// Compose an input string from CLI args: <feature> <args...>
	let args: Vec<String> = env::args().skip(1).collect();
	let input = if args.is_empty() {
		// Default to listing features
		println!("Available features: {}", sftp_core::list_features().join(", "));
		return;
	} else {
		let mut s = String::new();
		for (i, a) in args.iter().enumerate() {
			if i > 0 { s.push(' '); }
			s.push_str(a);
		}
		s
	};

	match sftp_core::process_input(&input) {
		Ok(out) => {
			println!("{}", out.message);
			std::process::exit(out.code);
		}
		Err(err) => {
			eprintln!("{}", err);
			std::process::exit(1);
		}
	}
}
