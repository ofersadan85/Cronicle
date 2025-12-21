// Cronicle Web Server Module
// HTTP server for the web UI and API

use anyhow::Result;
use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::info;

use crate::config::Config;
use crate::engine::Engine;

/// The Cronicle HTTP server
pub struct Server {
    config: Arc<Config>,
    engine: Arc<Engine>,
}

impl Server {
    /// Create a new Server instance
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        let engine = Arc::new(Engine::new(config.clone()).await?);
        
        // Start the engine
        engine.start().await?;
        
        Ok(Self {
            config,
            engine,
        })
    }

    /// Run the server
    pub async fn run(self) -> Result<()> {
        let port = self.config.get_http_port();
        let bind_addr = format!("{}:{}", self.config.web_server.http_bind_address, port);
        
        info!("Starting HTTP server on {}", bind_addr);
        
        // Build the application router
        let app = self.build_router();
        
        // Create the listener
        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        info!("Server listening on {}", bind_addr);
        
        // Run the server
        axum::serve(listener, app).await?;
        
        Ok(())
    }

    /// Build the application router
    fn build_router(self) -> Router {
        // Serve static files from htdocs
        let serve_dir = ServeDir::new("htdocs");
        
        Router::new()
            // Health check endpoint
            .route("/health", get(health_check))
            // API routes will be added here
            .route("/api/app/status", get({
                let engine = self.engine.clone();
                move || status_handler(engine.clone())
            }))
            // Serve static files
            .fallback_service(serve_dir)
    }
}

/// Health check handler
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Status API handler
async fn status_handler(engine: Arc<Engine>) -> impl IntoResponse {
    use serde_json::json;
    
    let active_jobs = engine.active_job_count().await;
    let enabled = engine.is_enabled().await;
    
    let response = json!({
        "code": 0,
        "description": "Success",
        "active_jobs": active_jobs,
        "enabled": enabled,
        "version": env!("CARGO_PKG_VERSION"),
    });
    
    (StatusCode::OK, axum::Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
