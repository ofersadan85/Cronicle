// Cronicle Engine Module
// Core scheduling and job management logic

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::storage::Storage;
use crate::scheduler::Scheduler;
use crate::job::JobManager;

/// The main Cronicle engine
pub struct Engine {
    config: Arc<Config>,
    storage: Arc<Box<dyn Storage>>,
    scheduler: Arc<Scheduler>,
    job_manager: Arc<JobManager>,
    state: Arc<RwLock<EngineState>>,
}

/// State of the Cronicle engine
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct EngineState {
    pub enabled: bool,
    pub is_primary: bool,
}

impl Engine {
    /// Create a new Engine instance
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        // Create storage backend
        let storage = crate::storage::create_storage(&config.storage).await?;
        let storage = Arc::new(storage);
        
        // Create scheduler with UTC timezone
        // TODO: Add timezone detection or make it configurable
        let scheduler = Scheduler::new(Arc::clone(&storage), "UTC")?;
        
        // Create job manager
        let job_manager = JobManager::new(Arc::clone(&storage));
        
        Ok(Self {
            config,
            storage,
            scheduler: Arc::new(scheduler),
            job_manager: Arc::new(job_manager),
            state: Arc::new(RwLock::new(EngineState {
                enabled: true,
                is_primary: false,
            })),
        })
    }

    /// Start the engine
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Cronicle engine starting up");
        
        // Create necessary directories
        self.setup_directories().await?;
        
        // Initialize state
        let mut state = self.state.write().await;
        state.enabled = true;
        drop(state); // Release lock before starting scheduler
        
        // Setup and start scheduler
        self.scheduler.setup(self.config.scheduler_startup_grace).await?;
        
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
        self.job_manager.active_job_count().await
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
        let engine = Engine::new(config).await.unwrap();
        assert_eq!(engine.active_job_count().await, 0);
        assert!(engine.is_enabled().await);
    }
}
