// Cronicle Engine Module
// Core scheduling and job management logic

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

/// The main Cronicle engine
pub struct Engine {
    config: Arc<Config>,
    active_jobs: Arc<RwLock<HashMap<String, Job>>>,
    state: Arc<RwLock<EngineState>>,
}

/// State of the Cronicle engine
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct EngineState {
    pub enabled: bool,
    pub is_primary: bool,
}

/// Represents a running job
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Job {
    pub id: String,
    pub event_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl Engine {
    /// Create a new Engine instance
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(EngineState {
                enabled: true,
                is_primary: false,
            })),
        }
    }

    /// Start the engine
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Cronicle engine starting up");
        
        // Create necessary directories
        self.setup_directories().await?;
        
        // Initialize state
        let mut state = self.state.write().await;
        state.enabled = true;
        
        tracing::info!("Cronicle engine startup complete");
        
        Ok(())
    }

    /// Setup required directories
    async fn setup_directories(&self) -> Result<()> {
        use tokio::fs;
        
        // Create log directory
        let log_jobs_dir = format!("{}/jobs", self.config.log_dir);
        fs::create_dir_all(&log_jobs_dir).await?;
        tracing::debug!("Created log directory: {}", log_jobs_dir);
        
        // Create queue directory
        fs::create_dir_all(&self.config.queue_dir).await?;
        tracing::debug!("Created queue directory: {}", self.config.queue_dir);
        
        Ok(())
    }

    /// Get the number of active jobs
    pub async fn active_job_count(&self) -> usize {
        self.active_jobs.read().await.len()
    }

    /// Check if the engine is enabled
    pub async fn is_enabled(&self) -> bool {
        self.state.read().await.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let config = Arc::new(Config::default());
        let engine = Engine::new(config);
        assert_eq!(engine.active_job_count().await, 0);
        assert!(engine.is_enabled().await);
    }
}
