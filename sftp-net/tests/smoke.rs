// Integration smoke test: runs only if SFTP_TEST_* env vars are set.
// Hinglish: Agar env vars nahi mile to test gracefully pass ho jayega.

use sftp_net::{Auth, HostKeyPolicy, SshConfig, SftpClient};

#[test]
fn sftp_smoke_env() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    let host = match env::var("SFTP_TEST_HOST") { Ok(v) => v, Err(_) => return Ok(()) };
    let port = env::var("SFTP_TEST_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(22);
    let user = match env::var("SFTP_TEST_USER") { Ok(v) => v, Err(_) => return Ok(()) };
    let password = env::var("SFTP_TEST_PASSWORD").ok();
    let key = env::var("SFTP_TEST_KEY").ok();
    let key_pass = env::var("SFTP_TEST_KEY_PASS").ok();
    let known_hosts = env::var("SFTP_TEST_KNOWN_HOSTS").ok();
    let dir = env::var("SFTP_TEST_DIR").unwrap_or_else(|_| "/".to_string());
    let policy = if env::var("SFTP_TEST_ACCEPT_NEW").ok().as_deref() == Some("1") {
        HostKeyPolicy::AcceptNew
    } else {
        HostKeyPolicy::Strict
    };

    let auth = if let Some(k) = key {
        Auth::KeyFile { username: user.clone(), private_key: k.into(), passphrase: key_pass, public_key: None }
    } else if let Some(pw) = password {
        Auth::Password { username: user.clone(), password: pw }
    } else {
        return Ok(());
    };

    let cfg = SshConfig {
        host,
        port,
        auth,
        known_hosts: known_hosts.map(Into::into),
        hostkey_policy: policy,
        timeout_ms: Some(30_000),
    };

    let client = SftpClient::connect(&cfg)?;
    let _ = client.list_dir(&dir)?;
    Ok(())
}
