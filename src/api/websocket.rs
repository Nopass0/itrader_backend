use std::sync::Arc;
use std::collections::HashSet;
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::IntoResponse,
};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use tokio::sync::RwLock;

use crate::core::state::AppState;

#[derive(Debug, Clone)]
struct WsClient {
    subscriptions: Arc<RwLock<HashSet<String>>>,
}

impl WsClient {
    fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct WsResponse {
    #[serde(rename = "type")]
    msg_type: String,
    data: serde_json::Value,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected");
    let client = WsClient::new();

    // Send initial connection message
    let connect_msg = WsResponse {
        msg_type: "connected".to_string(),
        data: serde_json::json!({
            "message": "Connected to iTrader WebSocket",
            "version": "1.0.0"
        }),
    };

    if let Err(e) = socket.send(Message::Text(
        serde_json::to_string(&connect_msg).unwrap()
    )).await {
        error!("Failed to send connection message: {}", e);
        return;
    }

    // Handle incoming messages
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(msg) => {
                        if let Err(e) = handle_message(&mut socket, &state, &client, msg).await {
                            error!("Error handling message: {}", e);
                            send_error(&mut socket, "INTERNAL_ERROR", &e.to_string()).await;
                        }
                    }
                    Err(e) => {
                        warn!("Invalid message format: {}", e);
                        send_error(&mut socket, "INVALID_FORMAT", "Invalid JSON format").await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client disconnected");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

async fn handle_message(
    socket: &mut WebSocket,
    state: &Arc<AppState>,
    client: &WsClient,
    msg: WsMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match msg.msg_type.as_str() {
        "subscribe" => handle_subscribe(socket, client, msg.data).await?,
        "unsubscribe" => handle_unsubscribe(socket, client, msg.data).await?,
        "request" => handle_request(socket, state, msg.data).await?,
        "admin" => handle_admin(socket, state, msg.data).await?,
        _ => {
            send_error(socket, "UNKNOWN_TYPE", &format!("Unknown message type: {}", msg.msg_type)).await;
        }
    }
    Ok(())
}

async fn handle_subscribe(
    socket: &mut WebSocket,
    client: &WsClient,
    data: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct SubscribeData {
        channels: Vec<String>,
    }

    let sub_data: SubscribeData = serde_json::from_value(data)?;
    let mut subs = client.subscriptions.write().await;
    
    for channel in sub_data.channels {
        subs.insert(channel.clone());
        info!("Client subscribed to channel: {}", channel);
    }

    let response = WsResponse {
        msg_type: "subscribed".to_string(),
        data: serde_json::json!({
            "channels": subs.iter().cloned().collect::<Vec<_>>()
        }),
    };

    socket.send(Message::Text(serde_json::to_string(&response)?)).await?;
    Ok(())
}

async fn handle_unsubscribe(
    socket: &mut WebSocket,
    client: &WsClient,
    data: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct UnsubscribeData {
        channels: Vec<String>,
    }

    let unsub_data: UnsubscribeData = serde_json::from_value(data)?;
    let mut subs = client.subscriptions.write().await;
    
    for channel in unsub_data.channels {
        subs.remove(&channel);
        info!("Client unsubscribed from channel: {}", channel);
    }

    let response = WsResponse {
        msg_type: "unsubscribed".to_string(),
        data: serde_json::json!({
            "channels": subs.iter().cloned().collect::<Vec<_>>()
        }),
    };

    socket.send(Message::Text(serde_json::to_string(&response)?)).await?;
    Ok(())
}

async fn handle_request(
    socket: &mut WebSocket,
    state: &Arc<AppState>,
    data: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct RequestData {
        resource: String,
        filters: Option<serde_json::Value>,
    }

    let req_data: RequestData = serde_json::from_value(data)?;
    
    let response_data = match req_data.resource.as_str() {
        "orders" => {
            let orders = state.repository.get_active_orders().await?;
            serde_json::json!({ "orders": orders })
        }
        "transactions" => {
            // TODO: Implement transaction fetching with filters
            serde_json::json!({ "transactions": [] })
        }
        "metrics" => {
            // TODO: Implement metrics fetching
            serde_json::json!({
                "metrics": {
                    "active_orders": 0,
                    "pending_transactions": 0,
                    "total_volume_24h": "0.00",
                    "success_rate": 0.0
                }
            })
        }
        "status" => {
            serde_json::json!({
                "status": "running",
                "uptime": chrono::Utc::now().signed_duration_since(state.start_time).num_seconds(),
                "version": "1.0.0"
            })
        }
        _ => {
            send_error(socket, "INVALID_RESOURCE", &format!("Unknown resource: {}", req_data.resource)).await;
            return Ok(());
        }
    };

    let response = WsResponse {
        msg_type: "response".to_string(),
        data: response_data,
    };

    socket.send(Message::Text(serde_json::to_string(&response)?)).await?;
    Ok(())
}

async fn handle_admin(
    socket: &mut WebSocket,
    state: &Arc<AppState>,
    data: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct AdminData {
        token: String,
        command: String,
        params: Option<serde_json::Value>,
    }

    let admin_data: AdminData = serde_json::from_value(data)?;
    
    // Verify admin token
    if admin_data.token != state.config.admin_token {
        send_error(socket, "UNAUTHORIZED", "Invalid admin token").await;
        return Ok(());
    }

    let response_data = match admin_data.command.as_str() {
        "approve_transaction" => {
            // TODO: Implement transaction approval
            serde_json::json!({ "status": "success", "message": "Transaction approved" })
        }
        "reject_transaction" => {
            // TODO: Implement transaction rejection
            serde_json::json!({ "status": "success", "message": "Transaction rejected" })
        }
        "set_balance" => {
            // TODO: Implement balance setting
            serde_json::json!({ "status": "success", "message": "Balance updated" })
        }
        "toggle_auto_mode" => {
            // TODO: Implement auto mode toggle
            serde_json::json!({ "status": "success", "message": "Auto mode toggled" })
        }
        "get_system_status" => {
            serde_json::json!({
                "status": "running",
                "auto_mode": true,
                "active_orders": 0,
                "pending_transactions": 0,
                "last_check": chrono::Utc::now().to_rfc3339()
            })
        }
        _ => {
            send_error(socket, "INVALID_COMMAND", &format!("Unknown command: {}", admin_data.command)).await;
            return Ok(());
        }
    };

    let response = WsResponse {
        msg_type: "admin_response".to_string(),
        data: response_data,
    };

    socket.send(Message::Text(serde_json::to_string(&response)?)).await?;
    Ok(())
}

async fn send_error(socket: &mut WebSocket, code: &str, message: &str) {
    let error_response = WsResponse {
        msg_type: "error".to_string(),
        data: serde_json::json!({
            "code": code,
            "message": message
        }),
    };

    if let Ok(json) = serde_json::to_string(&error_response) {
        let _ = socket.send(Message::Text(json)).await;
    }
}

// Public function to broadcast updates to all connected clients
pub async fn broadcast_update(msg_type: &str, data: serde_json::Value) {
    // TODO: Implement broadcasting to subscribed clients
    // This would require maintaining a list of connected clients
    info!("Broadcasting {} update: {:?}", msg_type, data);
}