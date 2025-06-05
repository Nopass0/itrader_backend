use std::sync::Arc;
use axum::{
    extract::{State, Query, Json},
    response::IntoResponse,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    body::Body,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};
use rust_decimal::{Decimal, prelude::*};
use crate::core::state::AppState;
use crate::utils::error::{Result, AppError};

#[derive(Debug, Deserialize)]
pub struct AdminAuth {
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct TransactionAction {
    transaction_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SetBalanceRequest {
    amount: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleAutoModeRequest {
    enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct AdminResponse {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

// Middleware to validate admin token
pub async fn admin_auth_middleware(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    request: axum::extract::Request,
    next: Next,
) -> Result<impl IntoResponse> {
    // Check Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));
    
    // Also check query parameter as fallback
    let query_token = request.uri()
        .query()
        .and_then(|q| {
            serde_urlencoded::from_str::<AdminAuth>(q).ok()
        })
        .map(|auth| auth.token);
    
    let token = auth_header.map(|s| s.to_string()).or(query_token);
    
    match token {
        Some(t) if t == state.config.admin_token => {
            Ok(next.run(request).await)
        }
        Some(_) => {
            warn!("Invalid admin token attempt");
            Err(AppError::Authentication("Invalid admin token".to_string()))
        }
        None => {
            Err(AppError::Authentication("Admin token required".to_string()))
        }
    }
}

// Admin handlers
pub async fn approve_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TransactionAction>,
) -> impl IntoResponse {
    info!("Admin: Approving transaction {}", req.transaction_id);
    
    match state.gate_client.approve_transaction(&req.transaction_id).await {
        Ok(_) => {
            (StatusCode::OK, Json(AdminResponse {
                success: true,
                message: format!("Transaction {} approved", req.transaction_id),
                data: None,
            }))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminResponse {
                success: false,
                message: format!("Failed to approve transaction: {}", e),
                data: None,
            }))
        }
    }
}

pub async fn reject_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TransactionAction>,
) -> impl IntoResponse {
    info!("Admin: Rejecting transaction {}", req.transaction_id);
    
    match state.gate_client.cancel_order(&req.transaction_id).await {
        Ok(_) => {
            (StatusCode::OK, Json(AdminResponse {
                success: true,
                message: format!("Transaction {} rejected", req.transaction_id),
                data: None,
            }))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminResponse {
                success: false,
                message: format!("Failed to reject transaction: {}", e),
                data: None,
            }))
        }
    }
}

pub async fn set_balance(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetBalanceRequest>,
) -> impl IntoResponse {
    info!("Admin: Setting balance to {}", req.amount);
    
    match rust_decimal::Decimal::from_str_exact(&req.amount) {
        Ok(amount) => {
            match state.gate_client.update_balance(amount.to_f64().unwrap_or(0.0)).await {
                Ok(_) => {
                    (StatusCode::OK, Json(AdminResponse {
                        success: true,
                        message: format!("Balance updated to {}", amount),
                        data: None,
                    }))
                }
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminResponse {
                        success: false,
                        message: format!("Failed to update balance: {}", e),
                        data: None,
                    }))
                }
            }
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(AdminResponse {
                success: false,
                message: format!("Invalid amount format: {}", e),
                data: None,
            }))
        }
    }
}

pub async fn toggle_auto_mode(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ToggleAutoModeRequest>,
) -> impl IntoResponse {
    info!("Admin: Setting auto mode to {}", req.enabled);
    
    let mut auto_mode = state.auto_approve_mode.write().await;
    *auto_mode = req.enabled;
    
    (StatusCode::OK, Json(AdminResponse {
        success: true,
        message: format!("Auto mode set to {}", req.enabled),
        data: None,
    }))
}

pub async fn get_system_status(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let auto_mode = *state.auto_approve_mode.read().await;
    let uptime = chrono::Utc::now().signed_duration_since(state.start_time).num_seconds();
    
    // Get active orders
    let active_orders = match state.repository.get_active_orders().await {
        Ok(orders) => orders.len(),
        Err(_) => 0,
    };
    
    // Get system metrics
    let data = json!({
        "status": "running",
        "uptime_seconds": uptime,
        "auto_mode": auto_mode,
        "active_orders": active_orders,
        "version": "1.0.0",
        "gate_authenticated": state.gate_client.is_authenticated().await,
        "last_check": state.last_check_time.read().await.to_rfc3339(),
    });
    
    (StatusCode::OK, Json(AdminResponse {
        success: true,
        message: "System status retrieved".to_string(),
        data: Some(data),
    }))
}

pub async fn get_transactions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let status_filter = params.get("status");
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);
    
    // TODO: Implement transaction fetching with filters
    let data = json!({
        "transactions": [],
        "total": 0,
        "limit": limit,
        "filters": {
            "status": status_filter
        }
    });
    
    (StatusCode::OK, Json(AdminResponse {
        success: true,
        message: "Transactions retrieved".to_string(),
        data: Some(data),
    }))
}

pub async fn get_logs(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let level = params.get("level").cloned().unwrap_or_else(|| "info".to_string());
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(100);
    
    // TODO: Implement log retrieval
    let data = json!({
        "logs": [],
        "total": 0,
        "limit": limit,
        "level": level
    });
    
    (StatusCode::OK, Json(AdminResponse {
        success: true,
        message: "Logs retrieved".to_string(),
        data: Some(data),
    }))
}