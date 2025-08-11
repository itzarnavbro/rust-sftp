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
    
}


