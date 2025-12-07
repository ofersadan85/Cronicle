// Cronicle Job Management Module
// Handles job execution and monitoring
// Ported from lib/job.js

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::storage::Storage;

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Running,
    Completed,
    Failed,
    Aborted,
}

/// Job result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub code: i32,
    pub description: String,
    #[serde(default)]
    pub output: String,
}

/// Active job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub event_id: String,
    pub event_title: String,
    pub plugin: String,
    pub category: String,
    pub started_at: i64,
    pub status: JobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JobResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
}

impl Job {
    /// Create a new job
    pub fn new(event_id: String, event_title: String, plugin: String, category: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            event_id,
            event_title,
            plugin,
            category,
            started_at: now,
            status: JobStatus::Running,
            result: None,
            completed_at: None,
        }
    }

    /// Mark job as completed
    pub fn complete(&mut self, code: i32, description: String, output: String) {
        self.status = if code == 0 {
            JobStatus::Completed
        } else {
            JobStatus::Failed
        };
        self.result = Some(JobResult {
            code,
            description,
            output,
        });
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    /// Mark job as aborted
    pub fn abort(&mut self, description: String) {
        self.status = JobStatus::Aborted;
        self.result = Some(JobResult {
            code: -1,
            description,
            output: String::new(),
        });
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }
}

/// Job manager
pub struct JobManager {
    storage: Arc<Box<dyn Storage>>,
    active_jobs: Arc<RwLock<HashMap<String, Job>>>,
}

impl JobManager {
    /// Create a new job manager
    pub fn new(storage: Arc<Box<dyn Storage>>) -> Self {
        Self {
            storage,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Launch a new job
    pub async fn launch_job(
        &self,
        event_id: String,
        event_title: String,
        plugin: String,
        category: String,
    ) -> Result<String> {
        let job = Job::new(event_id.clone(), event_title, plugin, category);
        let job_id = job.id.clone();
        
        // Store in active jobs
        self.active_jobs.write().await.insert(job_id.clone(), job.clone());
        
        // In a full implementation, this would:
        // 1. Check server availability and resource limits
        // 2. Spawn the plugin process
        // 3. Monitor the process
        // 4. Capture output and logs
        // 5. Handle timeouts and resource limits
        
        tracing::info!("Job {} launched for event {}", job_id, event_id);
        
        Ok(job_id)
    }

    /// Complete a job
    pub async fn complete_job(
        &self,
        job_id: &str,
        code: i32,
        description: String,
        output: String,
    ) -> Result<()> {
        let mut jobs = self.active_jobs.write().await;
        
        if let Some(job) = jobs.get_mut(job_id) {
            job.complete(code, description, output);
            
            // In a full implementation, this would:
            // 1. Store job result in storage
            // 2. Update job logs
            // 3. Send notifications if configured
            // 4. Clean up resources
            
            tracing::info!(
                "Job {} completed with status: {:?}",
                job_id,
                job.status
            );
            
            // Remove from active jobs after a delay (in production)
            // For now, keep it for status queries
        }
        
        Ok(())
    }

    /// Abort a running job
    pub async fn abort_job(&self, job_id: &str, reason: String) -> Result<()> {
        let mut jobs = self.active_jobs.write().await;
        
        if let Some(job) = jobs.get_mut(job_id) {
            job.abort(reason);
            
            // In a full implementation, this would:
            // 1. Kill the running process
            // 2. Clean up resources
            // 3. Store abort information
            
            tracing::info!("Job {} aborted", job_id);
        }
        
        Ok(())
    }

    /// Get job status
    pub async fn get_job(&self, job_id: &str) -> Option<Job> {
        self.active_jobs.read().await.get(job_id).cloned()
    }

    /// Get all active jobs
    pub async fn get_active_jobs(&self) -> Vec<Job> {
        self.active_jobs
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Get active job count
    pub async fn active_job_count(&self) -> usize {
        self.active_jobs.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageConfig;

    async fn create_test_job_manager() -> JobManager {
        let storage_config = StorageConfig::default();
        let storage = crate::storage::create_storage(&storage_config)
            .await
            .unwrap();
        JobManager::new(Arc::new(storage))
    }

    #[tokio::test]
    async fn test_launch_job() {
        let manager = create_test_job_manager().await;
        
        let job_id = manager
            .launch_job(
                "event1".to_string(),
                "Test Event".to_string(),
                "test-plugin".to_string(),
                "test-category".to_string(),
            )
            .await
            .unwrap();
        
        assert!(!job_id.is_empty());
        assert_eq!(manager.active_job_count().await, 1);
        
        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.event_id, "event1");
    }

    #[tokio::test]
    async fn test_complete_job() {
        let manager = create_test_job_manager().await;
        
        let job_id = manager
            .launch_job(
                "event1".to_string(),
                "Test Event".to_string(),
                "test-plugin".to_string(),
                "test-category".to_string(),
            )
            .await
            .unwrap();
        
        manager
            .complete_job(&job_id, 0, "Success".to_string(), "output".to_string())
            .await
            .unwrap();
        
        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_abort_job() {
        let manager = create_test_job_manager().await;
        
        let job_id = manager
            .launch_job(
                "event1".to_string(),
                "Test Event".to_string(),
                "test-plugin".to_string(),
                "test-category".to_string(),
            )
            .await
            .unwrap();
        
        manager
            .abort_job(&job_id, "User requested".to_string())
            .await
            .unwrap();
        
        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Aborted);
    }

    #[tokio::test]
    async fn test_get_active_jobs() {
        let manager = create_test_job_manager().await;
        
        manager
            .launch_job(
                "event1".to_string(),
                "Test Event 1".to_string(),
                "plugin1".to_string(),
                "cat1".to_string(),
            )
            .await
            .unwrap();
        
        manager
            .launch_job(
                "event2".to_string(),
                "Test Event 2".to_string(),
                "plugin2".to_string(),
                "cat2".to_string(),
            )
            .await
            .unwrap();
        
        let active_jobs = manager.get_active_jobs().await;
        assert_eq!(active_jobs.len(), 2);
    }
}
