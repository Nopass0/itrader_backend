use std::sync::Arc;
use axum::Router;
use tower_http::cors::CorsLayer;
use tracing::info;
use std::net::SocketAddr;

use crate::core::state::AppState;
use crate::utils::error::Result;

pub struct ApiServer {
    state: Arc<AppState>,
}

impl ApiServer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> Result<()> {
        let app = self.create_app();
        
        let addr: SocketAddr = format!("{}:{}", self.state.config.server.host, self.state.config.server.port)
            .parse()
            .map_err(|e| crate::utils::error::AppError::Config(format!("Invalid server address: {}", e)))?;

        info!("API server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await
            .map_err(|e| crate::utils::error::AppError::Internal(e.into()))?;
            
        axum::serve(listener, app).await
            .map_err(|e| crate::utils::error::AppError::Internal(e.into()))?;

        Ok(())
    }

    fn create_app(&self) -> Router {
        Router::new()
            .nest("/api/v1", crate::api::routes::create_routes(self.state.clone()))
            .layer(CorsLayer::permissive())
            .with_state(self.state.clone())
    }
}