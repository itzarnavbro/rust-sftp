// Authentication-specific logic building on sftp-core

use sshkeys::PublicKey;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    InvalidKey,
    InvalidPassword,
    UserNotFound,
}

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
                return Err(AuthError::Parse(format!("Invalid key line: {}", line).into()));
            }
    
            let username = parts[0].to_string();
            let key_str = parts[1..].join(" "); 
    
            
            let _parsed_key = PublicKey::from_string(&key_str)?;
    
           
            users.push(User {
                username,
                public_key: key_str,
            });
        }
    
        Ok(users)
  
}


