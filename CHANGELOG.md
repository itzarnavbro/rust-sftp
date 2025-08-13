# Changelog

All notable changes to this project will be documented in this file.

## v0.2.1 - 2025-08-13
- chore: include scripts/ and .vscode/ tasks in release artifacts for turn-key testing
- docs: README updates for automation helpers

## v0.2.0 - 2025-08-13
- feat: ssh2-based remote SFTP backend (crate: sftp-net)
- feat: CLI `sftp` commands: upload, download, ls with --host/--user and key/password auth
- feat: host key verification with known_hosts (Strict/AcceptNew/InsecureIgnore)
- chore: CI publishes Windows/macOS artifacts on tag
- fix: clippy warnings in sftp-core and sftp-auth

### Notes
- Added automation helpers to release packages: scripts for macOS/Windows and VS Code tasks for a one-click Windows smoke test.

## v0.1.0 - 2025-08-13
- Initial multi-crate workspace split (core, auth, transfer, cli)
- Local file transfer (copy/list) + auth key loading
- CI and packaging for Windows/macOS

## Roadmap
- Optional pure-Rust SSH backend (russh) behind a Cargo feature
- Integration tests against ephemeral sshd
- Config profiles and key agent support
