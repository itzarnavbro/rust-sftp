//! sftp-net: SSH/SFTP networking layer using ssh2.
//!
//! Hinglish comments included for clarity.

use ssh2::{KnownHostFileKind, Session};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("TCP connect failed: {0}")]
    Tcp(#[from] std::io::Error),
    #[error("SSH error: {0}")]
    Ssh(#[from] ssh2::Error),
    #[error("Host key verification failed for {host}")]
    HostKey { host: String },
    #[error("Invalid argument: {0}")]
    Invalid(String),
}

/// Auth method: password ya key file.
#[derive(Debug, Clone)]
pub enum Auth {
    Password { username: String, password: String },
    KeyFile {
        username: String,
        private_key: PathBuf,
        passphrase: Option<String>,
        public_key: Option<PathBuf>,
    },
}

/// Host key policy: strict ya accept-new/ignore.
#[derive(Debug, Clone, Copy)]
pub enum HostKeyPolicy {
    Strict,
    AcceptNew,
    InsecureIgnore,
}

/// Connection configuration.
#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub auth: Auth,
    pub known_hosts: Option<PathBuf>,
    pub hostkey_policy: HostKeyPolicy,
    pub timeout_ms: Option<u64>,
}

/// High-level client: connect, upload, download, ls.
pub struct SftpClient {
    sess: Session,
}

impl SftpClient {
    /// Naya connection create karo.
    pub fn connect(cfg: &SshConfig) -> Result<Self, NetError> {
        // TCP connect
        let addr = format!("{}:{}", cfg.host, cfg.port);
        let tcp = TcpStream::connect(addr)?;
        if let Some(ms) = cfg.timeout_ms { tcp.set_read_timeout(Some(std::time::Duration::from_millis(ms))).ok(); tcp.set_write_timeout(Some(std::time::Duration::from_millis(ms))).ok(); }

        // SSH handshake
        // ssh2::Session::new() returns Result<Session, ssh2::Error>
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        // Host key verification
        verify_host_key(&sess, cfg)?;

        // Authentication
        match &cfg.auth {
            Auth::Password { username, password } => {
                sess.userauth_password(username, password)?;
            }
            Auth::KeyFile { username, private_key, passphrase, public_key } => {
                let pass = passphrase.as_deref();
                sess.userauth_pubkey_file(username, public_key.as_deref(), private_key, pass)?;
            }
        }

        if !sess.authenticated() {
            return Err(NetError::Invalid("authentication failed".into()));
        }

        Ok(Self { sess })
    }

    /// Remote pe file upload karo.
    pub fn upload_file<P: AsRef<Path>, Q: AsRef<Path>>(&self, local: P, remote: Q) -> Result<(), NetError> {
        let sftp = self.sess.sftp()?;
        let mut src = File::open(local)?;
        let mut dst = sftp.create(remote.as_ref())?; // default 0o644
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = src.read(&mut buf)?;
            if n == 0 { break; }
            dst.write_all(&buf[..n])?;
        }
        Ok(())
    }

    /// Remote se file download karo.
    pub fn download_file<P: AsRef<Path>, Q: AsRef<Path>>(&self, remote: P, local: Q) -> Result<(), NetError> {
        let sftp = self.sess.sftp()?;
        let mut src = sftp.open(remote.as_ref())?;
        let mut dst = File::create(local)?;
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = src.read(&mut buf)?;
            if n == 0 { break; }
            dst.write_all(&buf[..n])?;
        }
        Ok(())
    }

    /// Remote directory list karo.
    pub fn list_dir<P: AsRef<Path>>(&self, remote_dir: P) -> Result<Vec<String>, NetError> {
        let sftp = self.sess.sftp()?;
        let mut out = Vec::new();
        for entry in sftp.readdir(remote_dir.as_ref())? {
            let (path, _stat) = entry;
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                out.push(name.to_string());
            }
        }
        Ok(out)
    }
}

/// Host key ko verify karo known_hosts ke against.
fn verify_host_key(sess: &Session, cfg: &SshConfig) -> Result<(), NetError> {
    // Agar policy InsecureIgnore hai to skip kar do (not recommended).
    match cfg.hostkey_policy {
        HostKeyPolicy::InsecureIgnore => return Ok(()),
        _ => {}
    }

    let (host_key, key_type) = sess.host_key().ok_or_else(|| NetError::Invalid("missing host key".into()))?;

    // Known hosts load karo. Agar file missing hai aur policy Strict hai to error.
    if let Some(path) = &cfg.known_hosts {
        let mut kh = sess.known_hosts().map_err(NetError::Ssh)?;
        kh.read_file(path, KnownHostFileKind::OpenSSH)?;
        // Check karo existing entry milta hai kya
        match kh.check_port(&cfg.host, cfg.port, host_key) {
            ssh2::CheckResult::Match => Ok(()),
            ssh2::CheckResult::Mismatch => Err(NetError::HostKey { host: cfg.host.clone() }),
            ssh2::CheckResult::NotFound => match cfg.hostkey_policy {
                HostKeyPolicy::AcceptNew => {
                    // Naya host add kar do file me. Non-22 port ke liye OpenSSH format: [host]:port
                    let host_for_file = if cfg.port == 22 {
                        cfg.host.clone()
                    } else {
                        format!("[{}]:{}", cfg.host, cfg.port)
                    };
                    kh.add(&host_for_file, host_key, "", key_type.into())?;
                    kh.write_file(path, KnownHostFileKind::OpenSSH)?;
                    Ok(())
                }
                HostKeyPolicy::Strict => Err(NetError::HostKey { host: cfg.host.clone() }),
                HostKeyPolicy::InsecureIgnore => Ok(()),
            },
            ssh2::CheckResult::Failure => Err(NetError::Invalid("known_hosts check failed".into())),
        }
    } else {
        // No known_hosts path diya. Strict me error, warna allow.
        match cfg.hostkey_policy {
            HostKeyPolicy::Strict => Err(NetError::Invalid("known_hosts not provided for strict policy".into())),
            _ => Ok(()),
        }
    }
}
