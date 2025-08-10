fn main() {
    println!("{} | {} | {}",
        sftp_core::hello_core(),
        sftp_auth::hello_auth(),
        sftp_transfer::hello_transfer()
    );
}
