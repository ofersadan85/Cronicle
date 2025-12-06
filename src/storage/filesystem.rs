// Filesystem storage backend for Cronicle
// Stores data as JSON files in a directory structure

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::Storage;

/// Filesystem storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    #[serde(default = "default_base_dir")]
    pub base_dir: String,
}

fn default_base_dir() -> String {
    "data".to_string()
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            base_dir: default_base_dir(),
        }
    }
}

/// Filesystem storage implementation
pub struct FilesystemStorage {
    base_path: PathBuf,
}

impl FilesystemStorage {
    /// Create a new filesystem storage instance
    pub fn new(config: &FilesystemConfig) -> Result<Self> {
        Ok(Self {
            base_path: PathBuf::from(&config.base_dir),
        })
    }

    /// Convert a storage key to a file path
    fn key_to_path(&self, key: &str) -> PathBuf {
        // Split key by slashes to create directory structure
        // e.g., "events/123" becomes base_path/events/123.json
        let parts: Vec<&str> = key.split('/').collect();
        let mut path = self.base_path.clone();
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part is the filename
                path.push(format!("{}.json", part));
            } else {
                // Intermediate parts are directories
                path.push(part);
            }
        }
        
        path
    }

    /// Extract a key from a file path
    fn path_to_key(&self, path: &Path) -> Result<String> {
        let relative = path.strip_prefix(&self.base_path)
            .context("Path is not within base directory")?;
        
        let key = relative
            .to_str()
            .context("Path contains invalid UTF-8")?
            .trim_end_matches(".json")
            .replace(std::path::MAIN_SEPARATOR, "/");
        
        Ok(key)
    }
}

#[async_trait]
impl Storage for FilesystemStorage {
    async fn init(&self) -> Result<()> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&self.base_path).await?;
        tracing::debug!("Filesystem storage initialized at {:?}", self.base_path);
        Ok(())
    }

    async fn put(&self, key: &str, value: serde_json::Value) -> Result<()> {
        let path = self.key_to_path(key);
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Serialize value to pretty JSON
        let json = serde_json::to_string_pretty(&value)?;
        
        // Write to file
        let mut file = fs::File::create(&path).await?;
        file.write_all(json.as_bytes()).await?;
        file.sync_all().await?;
        
        tracing::trace!("Stored key '{}' to {:?}", key, path);
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let path = self.key_to_path(key);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let contents = fs::read_to_string(&path).await?;
        let value: serde_json::Value = serde_json::from_str(&contents)?;
        
        tracing::trace!("Retrieved key '{}' from {:?}", key, path);
        Ok(Some(value))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.key_to_path(key);
        
        if path.exists() {
            fs::remove_file(&path).await?;
            tracing::trace!("Deleted key '{}' at {:?}", key, path);
        }
        
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        
        // Convert prefix to path
        let prefix_path = if prefix.is_empty() {
            self.base_path.clone()
        } else {
            self.key_to_path(prefix).parent().unwrap().to_path_buf()
        };
        
        if !prefix_path.exists() {
            return Ok(keys);
        }
        
        // Recursively walk the directory
        let mut stack = vec![prefix_path];
        
        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(key) = self.path_to_key(&path) {
                        if prefix.is_empty() || key.starts_with(prefix) {
                            keys.push(key);
                        }
                    }
                }
            }
        }
        
        keys.sort();
        Ok(keys)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.key_to_path(key);
        Ok(path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    async fn create_test_storage() -> (FilesystemStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig {
            base_dir: temp_dir.path().to_str().unwrap().to_string(),
        };
        let storage = FilesystemStorage::new(&config).unwrap();
        storage.init().await.unwrap();
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_put_and_get() {
        let (storage, _temp) = create_test_storage().await;
        
        let value = json!({"name": "test", "value": 123});
        storage.put("test/item", value.clone()).await.unwrap();
        
        let retrieved = storage.get("test/item").await.unwrap();
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_delete() {
        let (storage, _temp) = create_test_storage().await;
        
        let value = json!({"name": "test"});
        storage.put("test/item", value).await.unwrap();
        assert!(storage.exists("test/item").await.unwrap());
        
        storage.delete("test/item").await.unwrap();
        assert!(!storage.exists("test/item").await.unwrap());
    }

    #[tokio::test]
    async fn test_list() {
        let (storage, _temp) = create_test_storage().await;
        
        storage.put("items/1", json!({"id": 1})).await.unwrap();
        storage.put("items/2", json!({"id": 2})).await.unwrap();
        storage.put("other/3", json!({"id": 3})).await.unwrap();
        
        let items = storage.list("items").await.unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&"items/1".to_string()));
        assert!(items.contains(&"items/2".to_string()));
    }

    #[tokio::test]
    async fn test_nonexistent_key() {
        let (storage, _temp) = create_test_storage().await;
        
        let result = storage.get("nonexistent").await.unwrap();
        assert_eq!(result, None);
    }
}
