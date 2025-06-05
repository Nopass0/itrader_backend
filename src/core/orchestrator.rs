use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug, span, Level};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use std::str::FromStr;
use uuid::Uuid;
use serde_json::json;

use crate::utils::error::{AppError, Result};
use crate::core::state::AppState;
use crate::gate::{GateTransaction, api::GateAPI};
use crate::db::models::Order;
use crate::db::pool_manager::{PoolManager, PoolType};
use crate::ai::ChatManager;
// use crate::email::EmailMonitor;
use crate::ocr::ReceiptProcessor;

pub struct Orchestrator {
    state: Arc<AppState>,
    gate_api: Arc<GateAPI>,
    pool_manager: Arc<PoolManager>,
    chat_manager: Arc<ChatManager>,
    // email_monitor: Arc<EmailMonitor>,
    receipt_processor: Arc<ReceiptProcessor>,
}

impl Orchestrator {
    pub fn new(state: Arc<AppState>) -> Result<Self> {
        let gate_api = Arc::new(GateAPI::new(state.gate_manager.client.clone()));
        let pool_manager = Arc::new(PoolManager::new(state.repository.clone()));
        let chat_manager = Arc::new(ChatManager::new(state.config.ai.clone()));
        // let email_monitor = Arc::new(EmailMonitor::new(state.config.email.clone())?);
        let receipt_processor = Arc::new(ReceiptProcessor::new());

        Ok(Self {
            state,
            gate_api,
            pool_manager,
            chat_manager,
            // email_monitor,
            receipt_processor,
        })
    }

    pub async fn start_processing(&self) -> Result<()> {
        info!("Starting order processing loop");

        // Restore state from database
        self.pool_manager.restore_state().await?;

        // Start monitoring tasks
        self.start_monitoring_tasks().await;

        let mut interval = interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            // Check for new transactions
            if let Err(e) = self.process_new_transactions().await {
                error!("Error processing transactions: {}", e);
            }

            // Monitor active orders
            if let Err(e) = self.monitor_active_orders().await {
                error!("Error monitoring active orders: {}", e);
            }

            // Check if shutdown requested
            // Check for shutdown signal with select
            if tokio::select! {
                _ = self.state.shutdown_signal.notified() => true,
                _ = tokio::time::sleep(Duration::from_millis(0)) => false,
            } {
                info!("Orchestrator shutting down");
                break;
            }
        }

        Ok(())
    }

    async fn start_monitoring_tasks(&self) {
        // TODO: Re-enable email monitoring when email module is restored
        // Email monitoring task
        // let email_monitor = self.email_monitor.clone();
        // let receipt_processor = self.receipt_processor.clone();
        // let repository = self.state.repository.clone();
        // 
        // tokio::spawn(async move {
        //     loop {
        //         if let Err(e) = email_monitor.monitor_receipts(
        //             |receipt_data| async {
        //                 // Process receipt with OCR
        //                 // This is a placeholder - implement actual logic
        //                 Ok(())
        //             }
        //         ).await {
        //             error!("Email monitoring error: {}", e);
        //         }
        //         tokio::time::sleep(Duration::from_secs(30)).await;
        //     }
        // });
    }

    async fn process_new_transactions(&self) -> Result<()> {
        let pending_transactions = self.gate_api.get_pending_transactions().await?;

        for transaction in pending_transactions {
            // Check if already processing
            if let Some(_) = self.state.repository.get_order_by_gate_tx_id(&transaction.id).await? {
                continue;
            }

            info!("Processing new transaction: {}", transaction.id);
            
            if let Err(e) = self.process_transaction(transaction).await {
                error!("Error processing transaction: {}", e);
            }
        }

        Ok(())
    }

    async fn process_transaction(&self, tx: GateTransaction) -> Result<()> {
        let span = span!(Level::INFO, "process_transaction", tx_id = %tx.id);
        let _enter = span.enter();

        info!("Processing new transaction: {} for {} {}", tx.id, tx.amount, tx.currency);

        // 1. Accept on Gate
        self.gate_api.accept_transaction(&tx).await
            .map_err(|e| {
                error!("Failed to accept transaction: {}", e);
                e
            })?;

        // 2. Find available Bybit account
        let bybit_account = self.state.repository
            .get_available_bybit_account()
            .await?
            .ok_or_else(|| AppError::NoAvailableAccounts)?;

        let bybit_client = self.state
            .get_bybit_client(bybit_account.id)
            .ok_or_else(|| AppError::Config("Bybit client not initialized".to_string()))?;

        // 3. Calculate rate using formula
        let rate = self.calculate_rate(&tx)?;
        info!("Calculated rate: {} {} per {}", rate, tx.fiat_currency, tx.currency);

        // 4. Create Bybit advertisement
        let ad = bybit_client
            .create_sell_ad_from_transaction(&tx, rate)
            .await
            .map_err(|e| {
                error!("Failed to create advertisement: {}", e);
                e
            })?;

        // 5. Save to database
        let mut order = Order::new(
            tx.id.clone(),
            tx.amount,
            tx.currency,
            tx.fiat_currency,
            rate,
            tx.fiat_amount,
        );
        order.gate_account_id = Some(1); // TODO: Get actual account ID
        order.bybit_account_id = Some(bybit_account.id);
        order.metadata = json!({
            "bybit_ad_id": ad.id,
            "buyer_name": tx.buyer_name,
            "payment_method": tx.payment_method,
        });

        let order = self.state.repository.create_order(&order).await?;
        info!("Created order: {}", order.id);

        // 6. Add to active pool
        self.pool_manager.move_to_pool(
            order.id,
            PoolType::Active,
            json!({
                "ad_id": ad.id,
                "status": "waiting_for_buyer"
            })
        ).await?;

        // 7. Increment Bybit active ads
        self.state.repository.increment_bybit_active_ads(bybit_account.id).await?;

        // 8. Start monitoring in background
        let orchestrator = Arc::new(self.clone());
        tokio::spawn(async move {
            if let Err(e) = orchestrator.monitor_order(order).await {
                error!("Error monitoring order: {}", e);
            }
        });

        Ok(())
    }

    fn calculate_rate(&self, tx: &GateTransaction) -> Result<Decimal> {
        // TODO: Implement actual rate calculation based on:
        // - Order amount
        // - Time of day (Moscow time)
        // - Last X pages from SEP/Tinkoff filter
        
        // MOCK RATE CALCULATION - временное решение
        let mock_rate = Decimal::from_str("103.50")
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse mock rate: {}", e)))?; // Моковый курс 103.50 RUB за USDT
        
        info!(
            "📊 MOCK RATE SELECTED: {} RUB/USDT (это временный курс для тестирования)",
            mock_rate
        );
        info!(
            "📋 Transaction details: ID={}, Amount={} {}, Payment method: {}",
            tx.id, tx.amount, tx.currency, tx.payment_method
        );
        
        Ok(mock_rate)
    }

    async fn monitor_order(&self, order: Order) -> Result<()> {
        let span = span!(Level::INFO, "monitor_order", order_id = %order.id);
        let _enter = span.enter();

        info!("Starting to monitor order: {}", order.id);

        let bybit_client = self.state
            .get_bybit_client(order.bybit_account_id.unwrap())
            .ok_or_else(|| AppError::Config("Bybit client not found".to_string()))?;

        // Get advertisement ID from metadata
        let ad_id = order.metadata["bybit_ad_id"]
            .as_str()
            .ok_or_else(|| AppError::Config("Missing ad_id in metadata".to_string()))?;

        // Monitor for new orders on this advertisement
        loop {
            let orders = bybit_client.get_active_orders().await?;
            
            for p2p_order in orders {
                if p2p_order.ad_id == ad_id {
                    info!("Found buyer for order {}: {}", order.id, p2p_order.id);
                    
                    // Update order with Bybit order ID
                    self.state.repository.update_order_bybit_id(order.id, &p2p_order.id).await?;
                    
                    // Move to chat pool
                    self.pool_manager.move_to_pool(
                        order.id,
                        PoolType::Chat,
                        json!({
                            "bybit_order_id": p2p_order.id,
                            "buyer_id": p2p_order.buyer_id,
                        })
                    ).await?;
                    
                    // Start chat handling
                    return self.handle_order_chat(order, p2p_order).await;
                }
            }
            
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    async fn handle_order_chat(&self, order: Order, p2p_order: crate::bybit::models::P2POrder) -> Result<()> {
        info!("Handling chat for order: {}", order.id);

        let bybit_client = self.state
            .get_bybit_client(order.bybit_account_id.unwrap())
            .ok_or_else(|| AppError::Config("Bybit client not found".to_string()))?;

        // Send initial message
        let initial_message = self.get_initial_message(&order);
        bybit_client.send_message(&p2p_order.id, &initial_message).await?;

        // Create AI conversation
        self.state.repository.create_conversation(order.id).await?;

        // Monitor order status
        loop {
            let updated_order = bybit_client.get_order(&p2p_order.id).await?;
            
            match updated_order.status.as_str() {
                "PAID" => {
                    info!("Buyer marked order as paid: {}", order.id);
                    
                    // Move to verification pool
                    self.pool_manager.move_to_pool(
                        order.id,
                        PoolType::Verification,
                        json!({
                            "awaiting_receipt": true,
                            "payment_marked_at": chrono::Utc::now(),
                        })
                    ).await?;
                    
                    // Send message asking for receipt
                    let receipt_message = "Пожалуйста, отправьте чек об оплате. Please send the payment receipt.";
                    bybit_client.send_message(&p2p_order.id, receipt_message).await?;
                    
                    // Wait for receipt
                    return self.wait_for_receipt(order, p2p_order).await;
                }
                "CANCELLED" => {
                    warn!("Order cancelled by buyer: {}", order.id);
                    self.state.repository.update_order_status(order.id, "cancelled").await?;
                    return Ok(());
                }
                "APPEAL" => {
                    error!("Order under appeal: {}", order.id);
                    self.state.repository.update_order_status(order.id, "appeal").await?;
                    return Ok(());
                }
                _ => {
                    // Check for new messages and respond with AI
                    if let Err(e) = self.handle_chat_messages(&order, &p2p_order).await {
                        error!("Error handling chat messages: {}", e);
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_chat_messages(&self, order: &Order, p2p_order: &crate::bybit::models::P2POrder) -> Result<()> {
        let bybit_client = self.state
            .get_bybit_client(order.bybit_account_id.unwrap())
            .ok_or_else(|| AppError::Config("Bybit client not found".to_string()))?;

        let messages = bybit_client.get_chat_messages(&p2p_order.id).await?;
        
        // TODO: Process new messages with AI and respond
        
        Ok(())
    }

    async fn wait_for_receipt(&self, order: Order, p2p_order: crate::bybit::models::P2POrder) -> Result<()> {
        info!("Waiting for receipt for order: {}", order.id);
        
        // TODO: Implement receipt waiting logic
        // This would monitor email and chat for receipt images
        
        Ok(())
    }

    async fn monitor_active_orders(&self) -> Result<()> {
        // Monitor orders in different pools
        let active_orders = self.pool_manager.get_active_orders().await?;
        debug!("Monitoring {} active orders", active_orders.len());
        
        // TODO: Implement monitoring logic for orders in different states
        
        Ok(())
    }

    fn get_initial_message(&self, _order: &Order) -> String {
        // Template message in Russian and English
        "Здравствуйте! Спасибо за ваш заказ. Пожалуйста, произведите оплату и отправьте чек. Принимаем только Т-Банк.\n\nHello! Thank you for your order. Please make the payment and send the receipt. We only accept T-Bank.".to_string()
    }
}

impl Clone for Orchestrator {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            gate_api: self.gate_api.clone(),
            pool_manager: self.pool_manager.clone(),
            chat_manager: self.chat_manager.clone(),
            // email_monitor: self.email_monitor.clone(),
            receipt_processor: self.receipt_processor.clone(),
        }
    }
}