use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
    extract::State,
    response::IntoResponse,
    Json,
    http::StatusCode,
    middleware,
};
use serde_json::json;
use tower::ServiceBuilder;

use crate::core::state::AppState;
use crate::api::admin;

pub fn create_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    // Public routes
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/orders", get(get_orders))
        .route("/pools", get(get_pools))
        .route("/metrics", get(get_metrics))
        .route("/ws", get(crate::api::websocket::websocket_handler));
    
    // Admin routes with authentication middleware
    let admin_routes = Router::new()
        .route("/admin/approve", post(admin::approve_transaction))
        .route("/admin/reject", post(admin::reject_transaction))
        .route("/admin/balance", post(admin::set_balance))
        .route("/admin/auto-mode", post(admin::toggle_auto_mode))
        .route("/admin/status", get(admin::get_system_status))
        .route("/admin/transactions", get(admin::get_transactions))
        .route("/admin/logs", get(admin::get_logs))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    admin::admin_auth_middleware
                ))
        );
    
    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // TODO: Implement proper health checks
    (StatusCode::OK, Json(json!({
        "status": "healthy",
        "uptime": chrono::Utc::now().signed_duration_since(state.start_time).num_seconds(),
    })))
}

async fn get_orders(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.repository.get_active_orders().await {
        Ok(orders) => (StatusCode::OK, Json(json!({ "orders": orders }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to fetch orders" }))),
    }
}

async fn get_pools(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // TODO: Implement pool fetching
    (StatusCode::OK, Json(json!({ "pools": [] })))
}

async fn get_metrics(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // TODO: Implement metrics
    (StatusCode::OK, Json(json!({ "metrics": {} })))
}