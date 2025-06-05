use std::sync::Arc;
use tracing::{info, debug};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::utils::error::Result;
use super::client::GateClient;
use super::models::*;

pub struct GateAPI {
    client: Arc<GateClient>,
}

impl GateAPI {
    pub fn new(client: Arc<GateClient>) -> Self {
        Self { client }
    }

    pub async fn get_pending_transactions(&self) -> Result<Vec<GateTransaction>> {
        debug!("Fetching pending transactions from Gate.io");
        
        // Use get_available_transactions which properly filters for status 4 and 5
        let payouts = self.client.get_available_transactions().await?;
        
        // Convert Payout to GateTransaction
        let transactions: Vec<GateTransaction> = payouts.into_iter()
            .map(|payout| {
                // For status 4 transactions, amounts are empty until accepted
                // We'll use placeholder values and they'll be updated after acceptance
                let (rub_amount, rub_total) = if payout.status == 4 {
                    // Status 4 - pending, use placeholder amounts
                    (1000.0, 1000.0) // Will be updated after acceptance
                } else {
                    // Status 5 or others - extract actual amounts
                    let amount = payout.amount.trader
                        .get("643")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let total = payout.total.trader
                        .get("643")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    (amount, total)
                };

                GateTransaction {
                    id: payout.id.to_string(),
                    order_id: payout.id.to_string(),
                    amount: Decimal::from_f64_retain(rub_amount).unwrap_or_default(),
                    currency: "RUB".to_string(),
                    fiat_currency: "RUB".to_string(),
                    fiat_amount: Decimal::from_f64_retain(rub_total).unwrap_or_default(),
                    rate: Decimal::ONE,
                    status: payout.status,
                    buyer_name: payout.trader.as_ref().map(|t| t.name.clone()).unwrap_or_else(|| "Unknown".to_string()),
                    payment_method: payout.method.label,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            })
            .collect();
        
        info!("Found {} pending transactions (status 4 or 5)", transactions.len());
        
        Ok(transactions)
    }

    pub async fn accept_transaction(&self, transaction: &GateTransaction) -> Result<()> {
        info!("Accepting transaction: {} with amount {} {}", 
            transaction.id, transaction.amount, transaction.currency);
        
        self.client.accept_transaction(&transaction.id).await?;
        
        Ok(())
    }

    pub async fn complete_transaction(&self, transaction_id: &str) -> Result<()> {
        info!("Completing transaction: {}", transaction_id);
        
        self.client.complete_transaction(transaction_id).await?;
        
        Ok(())
    }

    pub async fn get_transaction_details(&self, transaction_id: &str) -> Result<Option<GateTransaction>> {
        let filter = TransactionFilter {
            status: None,
            currency: None,
            page: Some(1),
            limit: Some(100),
        };

        let transactions = self.client.get_transactions_with_filter(filter).await?;
        
        Ok(transactions.into_iter().find(|tx| tx.id == transaction_id))
    }

    pub async fn monitor_transactions<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(GateTransaction) -> Result<()>,
    {
        loop {
            let pending_transactions = self.get_pending_transactions().await?;
            
            for transaction in pending_transactions {
                if let Err(e) = callback(transaction.clone()) {
                    tracing::error!("Error processing transaction {}: {}", transaction.id, e);
                }
            }
            
            // Wait before next check
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}