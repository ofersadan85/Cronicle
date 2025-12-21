// Storage abstraction layer for Cronicle
// Supports multiple backends: Filesystem, Couchbase, S3

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod filesystem;

/// Main storage trait that all backends must implement
#[async_trait]
pub trait Storage: Send + Sync {
    /// Initialize the storage backend
    async fn init(&self) -> Result<()>;
    
    /// Store a record with the given key and value
    async fn put(&self, key: &str, value: serde_json::Value) -> Result<()>;
    
    /// Retrieve a record by key
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>>;
    
    /// Delete a record by key
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// List all keys matching a prefix
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
    
    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;
    
    /// Store multiple records in a transaction (if supported)
    async fn put_multi(&self, records: HashMap<String, serde_json::Value>) -> Result<()> {
        // Default implementation - can be overridden for atomic operations
        for (key, value) in records {
            self.put(&key, value).await?;
        }
        Ok(())
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_engine")]
    pub engine: String,
    
    #[serde(default)]
    pub filesystem: filesystem::FilesystemConfig,
}

fn default_engine() -> String {
    "Filesystem".to_string()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            engine: default_engine(),
            filesystem: filesystem::FilesystemConfig::default(),
        }
    }
}

/// Factory function to create the appropriate storage backend
pub async fn create_storage(config: &StorageConfig) -> Result<Box<dyn Storage>> {
    match config.engine.as_str() {
        "Filesystem" => {
            let storage = filesystem::FilesystemStorage::new(&config.filesystem)?;
            storage.init().await?;
            Ok(Box::new(storage))
        }
        engine => {
            anyhow::bail!("Unsupported storage engine: {}", engine)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_filesystem_storage() {
        let config = StorageConfig::default();
        let result = create_storage(&config).await;
        assert!(result.is_ok());
    }
}
