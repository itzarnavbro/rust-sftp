// Transfer-specific logic building on sftp-core

use std::{fmt, io, path::PathBuf};
use thiserror::Error;
use std::{fs, io::{Read, Write}, path::Path};

pub enum TransferError{
    FileNotFound(PathBuf),
    PermissionDenied(PathBuf),
    Io(#[from] io::Error),
}


pub struct TransferProgress {
    pub bytes_transferred: usize,
    pub total_bytes:usize,
}

impl TransferProgress{
    pub fn new(total_bytes: usize) -> Self {
        Self {
            bytes_transferred: 0,
            total_bytes,
        }
    }

    pub fn update(&mutself, bytes_sent:usize) {
        Self.bytes_transferred += bytes_sent;
    }

    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_transferred as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

pub struct TransferManager;

impl TransferManager{
    pub fn upload_file(src:&Path , dest:&Path) -> Result< TransferProgress , TransferError > {
        if !src.exists(){
            return Err(TransferError::FileNotFound(src.to_path_buf()));
        }

        let total_size = fs::metadata(src)?.len() as usize;
        let mut progress = TransferProgress::new(total_size);

        let mut src_file = fs::File::open(src)?;
        let mut dest_file = fs::File::create(dest)?;
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = src_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            dest_file.write_all(&buffer[..bytes_read])?;
            progress.update(bytes_read);
            println!("Progress: {:.2}%", progress.percentage());
        }

        Ok(progress)
    }

  pub fn download_file(src:&Path , dest:&Path) -> Result<() , TransferError> {
   
    if !src.exists(){
        return Err(TransferError::FileNotFound(src.to_path_buf()));
    }
    let mut src_file = fs::File::open(src)?;
    let mut dest_file = fs::File::create(dest)?;
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = src_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        dest_file.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
  }

  pub fn list_files(dir: &Path) -> Result<Vec<String>, TransferError> {
    if !dir.exists() {
        return Err(TransferError::FileNotFound(dir.to_path_buf()));
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        files.push(entry.file_name().to_string_lossy().into_owned());
    }
    Ok(files)
}

}


