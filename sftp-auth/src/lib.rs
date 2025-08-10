// Authentication-specific logic building on sftp-core

use sshkeys::PublicKey;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Error)]
