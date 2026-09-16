pub mod app_registry;
pub mod crypto;
pub mod registry;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub password_hash: String,
    pub lock_all: bool,
    pub locked_extensions: Vec<String>,
    pub exceptions: Vec<String>,
    pub explicit_locked_paths: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            password_hash: String::new(),
            lock_all: false,
            locked_extensions: vec!["txt".to_string()],
            exceptions: Vec::new(),
            explicit_locked_paths: Vec::new(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push("StandardLock");
        let _ = fs::create_dir_all(&p);
        p.push("config.json");
        p
    }

    pub fn load() -> Result<Self, String> {
        let p = Self::config_path();
        if !p.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn save(&self) -> Result<(), String> {
        let p = Self::config_path();
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(p, content).map_err(|e| e.to_string())
    }
}

pub fn hash_password(pw: &str) -> Result<String, String> {
    crypto::hash_password(pw)
}

pub fn verify_password(entered: &str, hash: &str) -> bool {
    crypto::verify_password(entered, hash)
}
