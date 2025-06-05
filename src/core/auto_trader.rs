use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc, FixedOffset, Timelike};
use uuid::Uuid;
use rust_decimal::{Decimal, prelude::*};
use serde_json::json;

use crate::utils::error::{AppError, Result};
use crate::utils::confirmation::ConfirmationHelper;
use crate::core::state::AppState;
use crate::core::config::AutoTraderConfig;
use crate::gate::models::Payout;
use crate::bybit::models::{Advertisement, P2POrder};
use crate::db::models::Order;

pub struct AutoTrader {
    state: Arc<AppState>,
    config: AutoTraderConfig,
    last_balance_check: DateTime<Utc>,
}


impl AutoTrader {
    pub fn new(state: Arc<AppState>, config: AutoTraderConfig) -> Self {
        Self {
            state,
            config,
            last_balance_check: Utc::now(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Auto-Trader");
        info!("Mode: {}", if self.config.auto_confirm { "AUTOMATIC" } else { "MANUAL" });
        
        // Set initial balance on startup
        info!("Setting initial Gate.io balances to 10M RUB");
        if let Err(e) = self.update_all_balances().await {
            error!("Failed to set initial balances: {}", e);
        }
        self.last_balance_check = Utc::now();
        
        let mut check_interval = interval(Duration::from_secs(self.config.check_interval_secs));
        let balance_check_duration = Duration::from_secs(self.config.balance_check_interval_hours * 3600);

        loop {
            check_interval.tick().await;
            
            // Check if we need to update balances
            if Utc::now() - self.last_balance_check > chrono::Duration::from_std(balance_check_duration).unwrap() {
                if let Err(e) = self.update_all_balances().await {
                    error!("Failed to update balances: {}", e);
                }
                self.last_balance_check = Utc::now();
            }

            // Process pending transactions
            if let Err(e) = self.process_pending_transactions().await {
                error!("Failed to process pending transactions: {}", e);
            }

            // Monitor active orders
            if let Err(e) = self.monitor_active_orders().await {
                error!("Failed to monitor active orders: {}", e);
            }

            // Process verification queue
            if let Err(e) = self.process_verification_queue().await {
                error!("Failed to process verification queue: {}", e);
            }
        }
    }

    async fn update_all_balances(&self) -> Result<()> {
        info!("Updating Gate.io account balances");
        
        let accounts = self.state.repository.get_active_gate_accounts().await?;
        
        for account in accounts {
            let current_balance = account.balance.to_f64().unwrap_or(0.0);
            
            if !self.config.auto_confirm {
                if !ConfirmationHelper::confirm_balance_update(
                    &account.email,
                    current_balance,
                    self.config.target_balance_rub,
                ) {
                    info!("Balance update cancelled for {}", account.email);
                    continue;
                }
            }
            
            match self.state.gate_manager.set_balance(&account.email, self.config.target_balance_rub).await {
                Ok(new_balance) => {
                    info!("Updated balance for {}: {} RUB", account.email, new_balance);
                    // Balance already updated in database by update_gate_account_balance
                }
                Err(e) => {
                    error!("Failed to update balance for {}: {}", account.email, e);
                }
            }
        }
        
        Ok(())
    }

    async fn process_pending_transactions(&self) -> Result<()> {
        let gate_accounts = self.state.repository.get_active_gate_accounts().await?;
        
        for account in gate_accounts {
            match self.state.gate_manager.get_pending_transactions(&account.email).await {
                Ok(transactions) => {
                    for tx in transactions {
                        // Check if already processing
                        if let Ok(Some(_)) = self.state.repository.get_order_by_gate_tx_id(&tx.id.to_string()).await {
                            continue;
                        }

                        // Extract fiat amount from tx.amount.trader
                        let fiat_amount = if let Some(rub_amount) = tx.amount.trader.get("643") {
                            rub_amount.as_f64().unwrap_or(0.0)
                        } else {
                            // Try to get first available currency
                            tx.amount.trader.values().next()
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0)
                        };

                        // Check transaction limits
                        if fiat_amount < self.config.min_order_amount || 
                           fiat_amount > self.config.max_order_amount {
                            warn!("Transaction {} outside limits: {} RUB", tx.id, fiat_amount);
                            continue;
                        }

                        info!("Found new transaction: {} for {} RUB", tx.id, fiat_amount);
                        
                        if let Err(e) = self.create_virtual_transaction(account.id, &account.email, tx).await {
                            error!("Failed to create virtual transaction: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get pending transactions for {}: {}", account.email, e);
                }
            }
        }
        
        Ok(())
    }

    async fn create_virtual_transaction(&self, gate_account_id: i32, gate_email: &str, tx: Payout) -> Result<()> {
        // Extract details
        let phone = tx.wallet.clone();
        let bank = if let Some(bank_info) = &tx.bank {
            bank_info.name.clone()
        } else {
            "Unknown Bank".to_string()
        };
        
        // Extract fiat amount and currency from tx.amount.trader
        let (fiat_amount, fiat_currency) = if let Some(rub_amount) = tx.amount.trader.get("643") {
            (rub_amount.as_f64().unwrap_or(0.0), "RUB")
        } else {
            // Try to get first available currency
            let first = tx.amount.trader.iter().next();
            if let Some((code, amount)) = first {
                (amount.as_f64().unwrap_or(0.0), code.as_str())
            } else {
                return Err(AppError::InternalError("No amount found in transaction".to_string()));
            }
        };
        
        if !self.config.auto_confirm {
            if !ConfirmationHelper::confirm_transaction(
                &tx.id.to_string(),
                fiat_amount,
                fiat_currency,
                &phone,
                &bank,
            ) {
                info!("Transaction {} rejected by user", tx.id);
                return Ok(());
            }
        }

        // Accept transaction on Gate
        self.state.gate_manager.approve_transaction(gate_email, &tx.id.to_string()).await?;
        
        // Calculate rate and create Bybit ad
        let rate = self.calculate_rate(fiat_amount);
        let amount_usdt = fiat_amount / rate;
        
        // Find available Bybit account
        let bybit_account = self.state.repository.get_available_bybit_account().await?
            .ok_or_else(|| AppError::InternalError("No available Bybit accounts".to_string()))?;
        
        if !self.config.auto_confirm {
            let payment_method = self.determine_payment_method(&bybit_account.account_name).await?;
            if !ConfirmationHelper::confirm_bybit_ad(
                fiat_amount,
                amount_usdt,
                rate,
                &payment_method,
                &bybit_account.account_name,
            ) {
                info!("Bybit ad creation cancelled");
                return Ok(());
            }
        }
        
        // Create advertisement on Bybit
        let ad = self.create_bybit_ad(&bybit_account.account_name, fiat_amount, rate).await?;
        
        // Create order in database
        let order = Order {
            id: Uuid::new_v4(),
            gate_transaction_id: tx.id.to_string(),
            bybit_order_id: Some(ad.id.clone()),
            gate_account_id: Some(gate_account_id),
            bybit_account_id: Some(bybit_account.id),
            amount: Decimal::from_f64(amount_usdt).unwrap_or_default(),
            currency: "USDT".to_string(),
            fiat_currency: fiat_currency.to_string(),
            rate: Decimal::from_f64(rate).unwrap_or_default(),
            total_fiat: Decimal::from_f64(fiat_amount).unwrap_or_default(),
            status: "active".to_string(),
            metadata: serde_json::json!({
                "phone": phone,
                "bank": bank,
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };
        
        self.state.repository.create_order(&order).await?;
        info!("Created virtual transaction linking Gate {} to Bybit ad {}", tx.id, ad.id);
        
        Ok(())
    }

    async fn create_bybit_ad(&self, account_name: &str, amount_rub: f64, rate: f64) -> Result<Advertisement> {
        let amount_usdt = amount_rub / rate + 1.0; // Add 1 USDT buffer
        
        let params = crate::bybit::models::AdParams {
            asset: "USDT".to_string(),
            fiat: "RUB".to_string(),
            price: rate.to_string(),
            amount: amount_usdt.to_string(),
            min_amount: Some(amount_rub.to_string()),
            max_amount: Some(amount_rub.to_string()),
            payment_methods: vec!["SBP".to_string()],
            remarks: Some(self.get_ad_template()),
        };
        
        // Get the Bybit client for this account
        let bybit_client = self.state.get_bybit_client(1) // TODO: Get proper account ID
            .ok_or_else(|| AppError::InternalError("No Bybit client available".to_string()))?;
        
        bybit_client.create_advertisement(params).await
            .map_err(|e| AppError::InternalError(format!("Failed to create advertisement: {}", e)))
    }

    async fn monitor_active_orders(&self) -> Result<()> {
        let active_orders = self.state.repository.get_active_orders().await?;
        
        for order in active_orders {
            if let Some(bybit_order_id) = &order.bybit_order_id {
                let bybit_account_id = order.bybit_account_id.unwrap_or(1);
                let bybit_client = self.state.get_bybit_client(bybit_account_id)
                    .ok_or_else(|| AppError::InternalError("No Bybit client available".to_string()))?;
                    
                match bybit_client.get_order(bybit_order_id).await {
                    Ok(p2p_order) => {
                        match p2p_order.status.as_str() {
                            "PAYMENT_PENDING" => {
                                // Send reminder about receipt
                                self.send_receipt_reminder(&order, &p2p_order).await?;
                            }
                            "PAYMENT_DONE" => {
                                // Move to verification queue
                                self.state.repository.update_order_status(order.id, "verifying_payment").await?;
                                info!("Order {} marked as paid, awaiting receipt", order.id);
                            }
                            "COMPLETED" => {
                                self.state.repository.update_order_status(order.id, "completed").await?;
                                info!("Order {} completed", order.id);
                            }
                            "CANCELLED" => {
                                self.state.repository.update_order_status(order.id, "cancelled").await?;
                                warn!("Order {} cancelled", order.id);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("Failed to get Bybit order status for {}: {}", bybit_order_id, e);
                    }
                }
            } else {
                // Check for new orders on our ad
                if let Some(ad_id) = order.metadata.get("bybit_ad_id").and_then(|v| v.as_str()) {
                    match self.check_for_new_orders(&order, ad_id).await {
                        Ok(Some(p2p_order)) => {
                            self.state.repository.update_order_bybit_id(order.id, &p2p_order.id).await?;
                            self.send_initial_message(&order, &p2p_order).await?;
                        }
                        Ok(None) => {}
                        Err(e) => error!("Failed to check for new orders: {}", e),
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn process_verification_queue(&self) -> Result<()> {
        // Get active orders and filter for verifying payment
        let orders = self.state.repository.get_active_orders().await?
            .into_iter()
            .filter(|o| o.status == "verifying_payment")
            .collect::<Vec<_>>();
        
        for order in orders {
            // Check for receipt in email
            match self.check_for_receipt(&order).await {
                Ok(Some(receipt_path)) => {
                    // Validate receipt
                    match self.validate_receipt(&order, &receipt_path).await {
                        Ok(true) => {
                            // Store receipt path in metadata
                            let mut metadata = order.metadata.clone();
                            metadata["receipt_path"] = json!(receipt_path);
                            self.state.repository.update_order_metadata(order.id, metadata).await?;
                            
                            if self.config.auto_confirm {
                                self.complete_order(&order).await?;
                            } else {
                                // Show validation result and ask for confirmation
                                if ConfirmationHelper::confirm_order_completion(
                                    order.id,
                                    &order.gate_transaction_id,
                                    order.bybit_order_id.as_ref().unwrap_or(&"N/A".to_string()),
                                    order.total_fiat.to_f64().unwrap_or(0.0),
                                    true,
                                ) {
                                    self.complete_order(&order).await?;
                                } else {
                                    info!("Order {} completion cancelled by user", order.id);
                                    self.state.repository.update_order_status(order.id, "manual_review").await?;
                                }
                            }
                        }
                        Ok(false) => {
                            warn!("Receipt validation failed for order {}", order.id);
                            self.state.repository.update_order_status(order.id, "manual_review").await?;
                        }
                        Err(e) => {
                            error!("Failed to validate receipt for order {}: {}", order.id, e);
                        }
                    }
                }
                Ok(None) => {
                    // No receipt yet, check timeout
                    let elapsed = Utc::now() - order.created_at;
                    if elapsed > chrono::Duration::minutes(30) {
                        warn!("Order {} timed out waiting for receipt", order.id);
                        self.state.repository.update_order_status(order.id, "manual_review").await?;
                    }
                }
                Err(e) => {
                    error!("Failed to check for receipt: {}", e);
                }
            }
        }
        
        Ok(())
    }

    async fn complete_order(&self, order: &Order) -> Result<()> {
        info!("Completing order {}", order.id);
        
        // Release funds on Bybit
        if let Some(bybit_order_id) = &order.bybit_order_id {
            let bybit_account_id = order.bybit_account_id.unwrap_or(1);
            let bybit_client = self.state.get_bybit_client(bybit_account_id)
                .ok_or_else(|| AppError::InternalError("No Bybit client available".to_string()))?;
            bybit_client.release_order(bybit_order_id).await?;
        }
        
        // Approve transaction on Gate
        // Get gate account email
        let gate_account = self.state.repository.get_gate_account_by_id(order.gate_account_id.unwrap_or(0)).await?
            .ok_or_else(|| AppError::InternalError("Gate account not found".to_string()))?;
        self.state.gate_manager.approve_transaction(&gate_account.email, &order.gate_transaction_id).await?;
        
        // Update order status
        self.state.repository.update_order_status(order.id, "completed").await?;
        
        // Delete Bybit ad
        if let Some(ad_id) = order.metadata.get("bybit_ad_id").and_then(|v| v.as_str()) {
            let bybit_account_id = order.bybit_account_id.unwrap_or(1);
            let bybit_client = self.state.get_bybit_client(bybit_account_id)
                .ok_or_else(|| AppError::InternalError("No Bybit client available".to_string()))?;
            bybit_client.delete_advertisement(ad_id).await?;
        }
        
        info!("Order {} completed successfully", order.id);
        Ok(())
    }

    fn calculate_rate(&self, amount_rub: f64) -> f64 {
        // Get Moscow time
        let moscow_offset = FixedOffset::east_opt(3 * 3600).unwrap();
        let moscow_time = Utc::now().with_timezone(&moscow_offset);
        let hour = moscow_time.hour();
        
        // Base rate
        let mut rate = 80.0;
        
        // Time-based adjustments
        rate *= match hour {
            0..=6 => 1.03,   // Night: +3%
            7..=9 => 1.015,  // Morning rush: +1.5%
            10..=16 => 1.02, // Day: +2%
            17..=19 => 1.015, // Evening rush: +1.5%
            20..=23 => 1.025, // Evening: +2.5%
            _ => 1.0,
        };
        
        // Amount-based adjustments
        rate *= match amount_rub {
            x if x <= 10_000.0 => 1.025,  // Small: +2.5%
            x if x <= 50_000.0 => 1.02,   // Medium: +2%
            x if x <= 200_000.0 => 1.015, // Large: +1.5%
            _ => 1.01,                     // Very large: +1%
        };
        
        // Round to 2 decimal places
        ((rate * 100.0) as f64).round() / 100.0
    }

    async fn determine_payment_method(&self, bybit_account: &str) -> Result<String> {
        // Check existing ads to determine which payment method to use
        // For now, always use SBP
        Ok("SBP".to_string())
        
    }

    fn get_ad_template(&self) -> String {
        let email = std::env::var("EMAIL_ADDRESS").unwrap_or_else(|_| "support@example.com".to_string());
        format!(
            "Добро пожаловать! Внимательно ознакомьтесь с условиями перед открытием сделки:\n\
            📌 СПОСОБ ОПЛАТЫ:\n\
            Принимаю переводы через СБП (Система быстрых платежей)\n\
            ВАЖНО: Переводы принимаются исключительно от Т-Банка\n\n\
            📋 ТРЕБОВАНИЯ К ПОДТВЕРЖДЕНИЮ:\n\
            PDF-чек обязательно отправить на email: {}\n\
            Чек должен быть отправлен непосредственно от имени банка\n\n\
            ⚠️ ОБРАТИТЕ ВНИМАНИЕ:\n\
            При открытии ордера внимательно проверяйте указанные реквизиты\n\
            Переводы, отправленные с других банков (кроме Т-Банка), не обрабатываются",
            email
        )
    }

    async fn send_initial_message(&self, order: &Order, p2p_order: &P2POrder) -> Result<()> {
        let message = "Здравствуйте! Прочитали ли вы правила сделки? Пожалуйста, ответьте Да/Нет.";
        let bybit_account_id = order.bybit_account_id.unwrap_or(1);
        let bybit_client = self.state.get_bybit_client(bybit_account_id)
            .ok_or_else(|| AppError::InternalError("No Bybit client available".to_string()))?;
        bybit_client.send_chat_message(&p2p_order.id, message).await?;
        Ok(())
    }

    async fn send_receipt_reminder(&self, order: &Order, p2p_order: &P2POrder) -> Result<()> {
        let email = std::env::var("EMAIL_ADDRESS").unwrap_or_else(|_| "support@example.com".to_string());
        let message = format!(
            "Пожалуйста, отправьте PDF чек на email: {}\n\
            Номер телефона для перевода: {}\n\
            Банк: {}",
            email,
            order.metadata.get("phone").and_then(|v| v.as_str()).unwrap_or("N/A"),
            order.metadata.get("bank").and_then(|v| v.as_str()).unwrap_or("N/A")
        );
        let bybit_account_id = order.bybit_account_id.unwrap_or(1);
        let bybit_client = self.state.get_bybit_client(bybit_account_id)
            .ok_or_else(|| AppError::InternalError("No Bybit client available".to_string()))?;
        bybit_client.send_chat_message(&p2p_order.id, &message).await?;
        Ok(())
    }

    async fn check_for_new_orders(&self, order: &Order, ad_id: &str) -> Result<Option<P2POrder>> {
        let bybit_account_id = order.bybit_account_id.unwrap_or(1);
        let bybit_client = self.state.get_bybit_client(bybit_account_id)
            .ok_or_else(|| AppError::InternalError("No Bybit client available".to_string()))?;
        let orders = bybit_client.get_advertisement_orders(ad_id).await?;
        Ok(orders.into_iter().find(|o| o.status == "TRADING"))
    }

    async fn check_for_receipt(&self, order: &Order) -> Result<Option<String>> {
        // TODO: Implement email manager check for receipt
        // self.state.email_manager.check_for_receipt(&order.gate_transaction_id, order.created_at).await
        Ok(None)
    }

    async fn validate_receipt(&self, order: &Order, _receipt_path: &str) -> Result<bool> {
        // TODO: Implement OCR processor
        // For now, return true to continue development
        Ok(true)
    }
}