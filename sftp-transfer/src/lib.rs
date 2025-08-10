// Transfer-specific logic building on sftp-core

pub fn hello_transfer() -> &'static str {
    sftp_core::hello_core();
    "hello from sftp-transfer"
}
