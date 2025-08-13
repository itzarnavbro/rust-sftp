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

## Remote SFTP over SSH (new)

Use the built-in ssh2 backend via the `sftp` feature in the CLI.

Examples (Windows cmd):

```cmd
# List a remote directory (strict host key check)
sftp-cli.exe sftp ls --host 192.168.1.10 --port 22 --user alice --key C:\id_ed25519 --known-hosts C:\Users\you\.ssh\known_hosts --strict /home/alice

# First-time trusted LAN (auto-add host key)
sftp-cli.exe sftp upload --host 192.168.1.10 --user alice --key C:\id_ed25519 --accept-new C:\src.txt /home/alice/dest.txt

# Password auth
sftp-cli.exe sftp download --host 192.168.1.10 --user alice --password secret /home/alice/src.txt C:\dest.txt
```

Notes:
- For non-22 ports, known_hosts uses OpenSSH format: `[host]:port`.
- Prefer key-based auth. Use `--key-pass` if your key is encrypted.
- Security: Use `--strict` with a curated known_hosts in production.

### Windows build note (OpenSSL)

If building locally, ensure Strawberry Perl is used for vendored OpenSSL builds:

```cmd
set "PATH=C:\Strawberry\perl\bin;%PATH%"
set PERL=C:\Strawberry\perl\bin\perl.exe
cargo build --workspace
```

## Testing

- Unit tests run via:

```cmd
cargo test --workspace --all-features -- --nocapture
```

- Optional integration smoke test for remote SFTP (requires an SSH server): set environment variables before running tests (only needed for sftp-net crate tests):

```cmd
set SFTP_TEST_HOST=192.168.1.10
set SFTP_TEST_PORT=22
set SFTP_TEST_USER=alice
set SFTP_TEST_PASSWORD=secret   # or set SFTP_TEST_KEY=C:\id_ed25519 and optionally SFTP_TEST_KEY_PASS=...
set SFTP_TEST_KNOWN_HOSTS=C:\Users\you\.ssh\known_hosts
cargo test -p sftp-net -- --nocapture
```

### Automation helpers (included in release zips)

- scripts/macos/setup_sshd.sh: Enable Remote Login (sshd) and prepare `~/.ssh/authorized_keys` on mac.
- scripts/windows/test_sftp.ps1: One-shot Windows smoke test to ls/upload/download against the mac.
- .vscode/tasks.json: VS Code tasks to run the smoke test (Windows).

Tip (Windows cmd):

```cmd
powershell -ExecutionPolicy Bypass -File scripts\windows\test_sftp.ps1 -Host 192.168.1.10 -User alice
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
