// Cronicle Scheduler Module
// Handles event scheduling and timing
// Ported from lib/scheduler.js

use anyhow::Result;
use chrono::{DateTime, Utc, Timelike};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub mod timing;

use crate::storage::Storage;

/// Event schedule item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub catch_up: bool,
    pub timezone: Option<String>,
    pub timing: Option<timing::Timing>,
    pub queue: bool,
    #[serde(default)]
    pub notify_fail: bool,
    #[serde(default)]
    pub web_hook: Option<String>,
}

/// Scheduler state
pub struct Scheduler {
    storage: Arc<Box<dyn Storage>>,
    cursors: Arc<RwLock<HashMap<String, i64>>>,
    event_queue: Arc<RwLock<HashMap<String, usize>>>,
    enabled: Arc<RwLock<bool>>,
    default_timezone: Tz,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new(storage: Arc<Box<dyn Storage>>, default_timezone: &str) -> Result<Self> {
        let tz = default_timezone.parse::<Tz>()
            .unwrap_or(chrono_tz::UTC);
        
        Ok(Self {
            storage,
            cursors: Arc::new(RwLock::new(HashMap::new())),
            event_queue: Arc::new(RwLock::new(HashMap::new())),
            enabled: Arc::new(RwLock::new(true)),
            default_timezone: tz,
        })
    }

    /// Initialize the scheduler
    pub async fn setup(&self, startup_grace_seconds: u64) -> Result<()> {
        tracing::info!("Setting up scheduler");
        
        // Load previous state (cursors)
        self.load_state().await?;
        
        // Load schedule items
        let items = self.load_schedule().await?;
        tracing::debug!("Loaded {} schedule items", items.len());
        
        // Load event queue counts for queued events
        for item in &items {
            if item.queue {
                // In a full implementation, we'd load queue lengths from storage
                // For now, initialize to 0
                self.event_queue.write().await.insert(item.id.clone(), 0);
            }
        }
        
        // Start the minute ticker after grace period
        let scheduler = Arc::new(self.clone_refs());
        tokio::spawn(async move {
            // Wait for grace period
            tokio::time::sleep(Duration::from_secs(startup_grace_seconds)).await;
            tracing::info!("Scheduler grace period complete, starting minute ticker");
            
            // Start minute ticker
            scheduler.run_minute_ticker().await;
        });
        
        Ok(())
    }

    /// Clone references for spawning tasks
    fn clone_refs(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            cursors: Arc::clone(&self.cursors),
            event_queue: Arc::clone(&self.event_queue),
            enabled: Arc::clone(&self.enabled),
            default_timezone: self.default_timezone,
        }
    }

    /// Load scheduler state from storage
    async fn load_state(&self) -> Result<()> {
        // Try to load global/state
        if let Some(state_value) = self.storage.get("global/state").await? {
            if let Some(cursors_obj) = state_value.get("cursors") {
                if let Some(cursors_map) = cursors_obj.as_object() {
                    let mut cursors = self.cursors.write().await;
                    for (key, value) in cursors_map {
                        if let Some(timestamp) = value.as_i64() {
                            cursors.insert(key.clone(), timestamp);
                        }
                    }
                    tracing::debug!("Loaded {} event cursors", cursors.len());
                }
            }
        }
        
        Ok(())
    }

    /// Load schedule items from storage
    async fn load_schedule(&self) -> Result<Vec<ScheduleItem>> {
        let mut items = Vec::new();
        
        // List all schedule items
        let keys = self.storage.list("global/schedule").await?;
        
        for key in keys {
            if let Some(value) = self.storage.get(&key).await? {
                if let Ok(item) = serde_json::from_value::<ScheduleItem>(value) {
                    items.push(item);
                }
            }
        }
        
        Ok(items)
    }

    /// Run the minute ticker
    async fn run_minute_ticker(&self) {
        let mut interval = interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            if !*self.enabled.read().await {
                continue;
            }
            
            let now = Utc::now();
            if let Err(e) = self.minute_tick(now).await {
                tracing::error!("Error in minute tick: {}", e);
            }
        }
    }

    /// Process a minute tick
    async fn minute_tick(&self, now: DateTime<Utc>) -> Result<()> {
        tracing::debug!("Scheduler minute tick: {}", now.format("%Y-%m-%d %H:%M:%S"));
        
        // Normalize to minute boundary
        let now_minute = now.with_second(0).unwrap().with_nanosecond(0).unwrap();
        let now_timestamp = now_minute.timestamp();
        
        // Load current schedule
        let items = self.load_schedule().await?;
        
        for item in items {
            if !item.enabled {
                continue;
            }
            
            // Get or initialize cursor for this event
            let mut cursors = self.cursors.write().await;
            let cursor = cursors.entry(item.id.clone()).or_insert(now_timestamp - 60);
            
            // Determine timezone
            let tz = item.timezone.as_deref()
                .and_then(|tz_str| tz_str.parse::<Tz>().ok())
                .unwrap_or(self.default_timezone);
            
            // Process each missed minute if catch_up is enabled
            let mut current_cursor = *cursor;
            while current_cursor < now_timestamp {
                current_cursor += 60;
                
                // Convert to timezone-aware datetime
                let cursor_dt = DateTime::from_timestamp(current_cursor, 0)
                    .unwrap()
                    .with_timezone(&tz);
                
                // Check if event should run at this time
                if let Some(timing) = &item.timing {
                    if timing::check_timing(timing, &cursor_dt) {
                        tracing::info!(
                            "Event {} ({}) should run at {}",
                            item.id,
                            item.title,
                            cursor_dt.format("%Y-%m-%d %H:%M:%S %Z")
                        );
                        
                        // In full implementation, this would launch or queue the job
                        // For now, just log it
                    }
                }
            }
            
            // Update cursor
            *cursor = now_timestamp;
        }
        
        Ok(())
    }

    /// Enable the scheduler
    pub async fn enable(&self) {
        *self.enabled.write().await = true;
        tracing::info!("Scheduler enabled");
    }

    /// Disable the scheduler
    pub async fn disable(&self) {
        *self.enabled.write().await = false;
        tracing::info!("Scheduler disabled");
    }

    /// Check if scheduler is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageConfig;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let storage_config = StorageConfig::default();
        let storage = crate::storage::create_storage(&storage_config).await.unwrap();
        let storage = Arc::new(storage);
        
        let scheduler = Scheduler::new(storage, "UTC").unwrap();
        assert!(scheduler.is_enabled().await);
    }

    #[tokio::test]
    async fn test_scheduler_enable_disable() {
        let storage_config = StorageConfig::default();
        let storage = crate::storage::create_storage(&storage_config).await.unwrap();
        let storage = Arc::new(storage);
        
        let scheduler = Scheduler::new(storage, "UTC").unwrap();
        assert!(scheduler.is_enabled().await);
        
        scheduler.disable().await;
        assert!(!scheduler.is_enabled().await);
        
        scheduler.enable().await;
        assert!(scheduler.is_enabled().await);
    }
}
