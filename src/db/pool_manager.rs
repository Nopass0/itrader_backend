use std::sync::Arc;
use uuid::Uuid;
use serde_json::json;
use tracing::{info, debug, error};

use crate::utils::error::Result;
use crate::db::Repository;
use crate::db::models::{Order, OrderPool};

#[derive(Debug, Clone)]
pub enum PoolType {
    Pending,
    Active,
    Chat,
    Verification,
    Completed,
}

impl PoolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Chat => "chat",
            Self::Verification => "verification",
            Self::Completed => "completed",
        }
    }
}

pub struct PoolManager {
    repository: Arc<Repository>,
}

impl PoolManager {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn move_to_pool(&self, order_id: Uuid, pool_type: PoolType, data: serde_json::Value) -> Result<()> {
        info!("Moving order {} to {} pool", order_id, pool_type.as_str());
        
        // Add to new pool
        self.repository.add_to_pool(pool_type.as_str(), order_id, data).await?;
        
        // Update order status based on pool
        let status = match pool_type {
            PoolType::Pending => "pending",
            PoolType::Active => "active",
            PoolType::Chat => "chatting",
            PoolType::Verification => "payment_received",
            PoolType::Completed => "completed",
        };
        
        self.repository.update_order_status(order_id, status).await?;
        
        Ok(())
    }

    pub async fn get_pending_orders(&self) -> Result<Vec<Order>> {
        let pools = self.repository.get_pool_orders(PoolType::Pending.as_str()).await?;
        
        let mut orders = Vec::new();
        for pool in pools {
            if let Some(order) = self.repository.get_order_by_id(pool.order_id).await? {
                orders.push(order);
            }
        }
        
        Ok(orders)
    }

    pub async fn get_active_orders(&self) -> Result<Vec<Order>> {
        let pools = self.repository.get_pool_orders(PoolType::Active.as_str()).await?;
        
        let mut orders = Vec::new();
        for pool in pools {
            if let Some(order) = self.repository.get_order_by_id(pool.order_id).await? {
                orders.push(order);
            }
        }
        
        Ok(orders)
    }

    pub async fn restore_state(&self) -> Result<()> {
        info!("Restoring pool state from database");
        
        // Get all active orders
        let active_orders = self.repository.get_active_orders().await?;
        info!("Found {} active orders to restore", active_orders.len());
        
        for order in active_orders {
            debug!("Restoring order {} with status {}", order.id, order.status);
            
            // Determine which pool the order should be in based on status
            let pool_type = match order.status.as_str() {
                "pending" | "accepted" => PoolType::Pending,
                "advertised" | "buyer_found" => PoolType::Active,
                "chatting" => PoolType::Chat,
                "payment_received" => PoolType::Verification,
                _ => {
                    error!("Unknown order status: {}", order.status);
                    continue;
                }
            };
            
            // Check if already in correct pool
            let existing_pools = self.repository.get_pool_orders(pool_type.as_str()).await?;
            let already_in_pool = existing_pools.iter().any(|p| p.order_id == order.id);
            
            if !already_in_pool {
                // Add to appropriate pool
                let data = json!({
                    "restored": true,
                    "previous_status": order.status.clone(),
                    "metadata": order.metadata
                });
                
                self.repository.add_to_pool(pool_type.as_str(), order.id, data).await?;
                info!("Restored order {} to {} pool", order.id, pool_type.as_str());
            }
        }
        
        Ok(())
    }

    pub async fn cleanup_completed_pools(&self, days_to_keep: i64) -> Result<u64> {
        // TODO: Implement cleanup of old completed orders
        // This would delete pool entries older than specified days
        Ok(0)
    }
}