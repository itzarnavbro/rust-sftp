// Authentication-specific logic building on sftp-core
use std::fs;
use std::path::Path;
use sshkeys::PublicKey;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid key")]
    InvalidKey,
    #[error("invalid password")]
    InvalidPassword,
    #[error("user not found")]
    UserNotFound,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("key parse error: {0}")]
    Key(#[from] sshkeys::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    Username: String,
    Public_key: String,
}


pub enum AuthMethod {
    Password,
    PublicKey,
}

pub struct AuthManager {
    #[allow(dead_code)]
    users: HashMap<String,PublicKey>
}

impl AuthManager {
    pub fn new() -> Self{
        AuthManager{
            users: HashMap::new(),
        }
    }
}


pub fn load_keys_from_file(path: &Path) -> Result<Vec<User>, AuthError> {

        let file_content = fs::read_to_string(path)?;
    
        let mut users = Vec::new();
    
        
        for line in file_content.lines() {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
    
            if parts.len() < 2 {
                return Err(AuthError::Parse(format!("Invalid key line: {}", line)));
            }
    
            let username = parts[0].to_string();
            let key_str = parts[1..].join(" "); 
    
            
            let _parsed_key = PublicKey::from_string(&key_str)?;
    
           
            users.push(User {
                Username: username,
                Public_key: key_str,
            });
        }
    
        Ok(users)
  
}


