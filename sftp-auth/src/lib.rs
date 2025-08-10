// Authentication-specific logic building on sftp-core

pub fn hello_auth() -> &'static str {
    sftp_core::hello_core();
    "hello from sftp-auth"
}
