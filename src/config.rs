// Configuration module for Cronicle
// Handles loading and parsing of config.json

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_base_app_url")]
    pub base_app_url: String,
    
    #[serde(default = "default_email_from")]
    pub email_from: String,
    
    #[serde(default = "default_smtp_hostname")]
    pub smtp_hostname: String,
    
    #[serde(default = "default_secret_key")]
    pub secret_key: String,
    
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    
    #[serde(default = "default_queue_dir")]
    pub queue_dir: String,
    
    #[serde(default = "default_job_memory_max")]
    pub job_memory_max: u64,
    
    #[serde(default)]
    pub debug: bool,
    
    #[serde(default)]
    pub master: bool,
    
    #[serde(default = "default_scheduler_startup_grace")]
    pub scheduler_startup_grace: u64,
    
    #[serde(default = "default_job_startup_grace")]
    pub job_startup_grace: u64,
    
    #[serde(default)]
    pub web_server: WebServerConfig,
    
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServerConfig {
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    
    #[serde(default = "default_http_bind_address")]
    pub http_bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_engine")]
    pub engine: String,
    
    #[serde(default)]
    pub filesystem: FilesystemStorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemStorageConfig {
    #[serde(default = "default_base_dir")]
    pub base_dir: String,
}

// Default value functions
fn default_base_app_url() -> String {
    "http://localhost:3012".to_string()
}

fn default_email_from() -> String {
    "cronicle@localhost".to_string()
}

fn default_smtp_hostname() -> String {
    "localhost".to_string()
}

fn default_secret_key() -> String {
    "CHANGE_ME".to_string()
}

fn default_http_port() -> u16 {
    3012
}

fn default_http_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn default_log_dir() -> String {
    "logs".to_string()
}

fn default_queue_dir() -> String {
    "data/queue".to_string()
}

fn default_job_memory_max() -> u64 {
    1073741824 // 1GB in bytes
}

fn default_scheduler_startup_grace() -> u64 {
    10
}

fn default_job_startup_grace() -> u64 {
    10
}

fn default_storage_engine() -> String {
    "Filesystem".to_string()
}

fn default_base_dir() -> String {
    "data".to_string()
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            http_port: default_http_port(),
            http_bind_address: default_http_bind_address(),
        }
    }
}

impl Default for FilesystemStorageConfig {
    fn default() -> Self {
        Self {
            base_dir: default_base_dir(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            engine: default_storage_engine(),
            filesystem: FilesystemStorageConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a JSON file
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Check if file exists
        if !path.exists() {
            tracing::warn!("Config file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }
        
        // Read and parse the config file
        let contents = fs::read_to_string(path)
            .await
            .context("Failed to read config file")?;
        
        let config: Config = serde_json::from_str(&contents)
            .context("Failed to parse config JSON")?;
        
        Ok(config)
    }
    
    /// Get the HTTP port from either the main config or web_server config
    pub fn get_http_port(&self) -> u16 {
        if self.web_server.http_port != default_http_port() {
            self.web_server.http_port
        } else {
            self.http_port
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_app_url: default_base_app_url(),
            email_from: default_email_from(),
            smtp_hostname: default_smtp_hostname(),
            secret_key: default_secret_key(),
            http_port: default_http_port(),
            log_dir: default_log_dir(),
            queue_dir: default_queue_dir(),
            job_memory_max: default_job_memory_max(),
            debug: false,
            master: false,
            scheduler_startup_grace: default_scheduler_startup_grace(),
            job_startup_grace: default_job_startup_grace(),
            web_server: WebServerConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.http_port, 3012);
        assert_eq!(config.get_http_port(), 3012);
    }

    #[tokio::test]
    async fn test_load_nonexistent_config() {
        let result = Config::load("/nonexistent/path/config.json").await;
        assert!(result.is_ok());
    }
}
