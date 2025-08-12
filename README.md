# rust-sftp (multi-crate workspace)

A simple Rust SFTP-oriented workspace split into feature crates. The CLI dispatches to feature crates directly (no dynamic registration), keeping each feature isolated and easy to evolve.

## Workspace layout

```
rust-sftp/
├─ Cargo.toml            # workspace definition
├─ sftp-core/            # core types and small helpers
│  └─ src/lib.rs
├─ sftp-auth/            # authentication feature (friend's code)
│  ├─ Cargo.toml
│  └─ src/lib.rs
├─ sftp-transfer/        # file transfer feature (friend's code)
│  ├─ Cargo.toml
│  └─ src/lib.rs
├─ sftp-cli/             # binary CLI that dispatches to features
│  ├─ Cargo.toml
│  └─ src/main.rs
└─ target/
```

## Build

- Prereqs: Rust toolchain (stable), Cargo
- Build the entire workspace:

```cmd
cargo build
```

If you only want to build the CLI binary:

```cmd
cargo build -p sftp-cli
```

## Run (CLI)

Show help and available features:

```cmd
cargo run -p sftp-cli -- --help
```

Verbose output (adds debug logs):

```cmd
cargo run -p sftp-cli -- -v --help
```

### Auth commands

Load users and their public keys from a file (OpenSSH format, space-separated `username key...` per line):

```cmd
cargo run -p sftp-cli -- auth load_keys C:\path\to\authorized_keys
```

Notes:
- Paths with spaces should be quoted in your shell, e.g. `"C:\\Users\\me\\My Keys\\authorized_keys"`.

### Transfer commands

Upload a file:

```cmd
cargo run -p sftp-cli -- transfer upload C:\path\to\src.txt C:\path\to\dest.txt
```

Download (local-to-local copy, placeholder semantics):

```cmd
cargo run -p sftp-cli -- transfer download C:\path\to\src.txt C:\path\to\dest.txt
```

List files in a directory:

```cmd
cargo run -p sftp-cli -- transfer ls C:\path\to\dir
```

## Design notes

- Direct dispatch: The CLI calls feature crates directly to keep your friend’s code unchanged.
- Core (`sftp-core`) currently provides a small set of shared types and a static `list_features()`.
- The feature crates expose plain functions/structs without CLI-specific types.

## Development

- Format/lint: use `cargo fmt` and `cargo clippy` if desired.
- Tests: add crate-level unit tests under each crate’s `src/` as needed.

## Troubleshooting

- Windows paths: prefer full absolute paths and quote when they contain spaces.
- If a dependency error appears, run `cargo update` once to refresh the lockfile.

## Roadmap (optional)

- Add real SFTP backend integration (e.g., via thrussh) when environment is prepared.
- Extend auth to support password verification and key-based auth checks.
- Extend transfer to remote endpoints and progress callbacks.

## License

MIT (or as you choose).
