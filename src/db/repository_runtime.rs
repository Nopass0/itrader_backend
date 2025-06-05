use sqlx::{PgPool, postgres::PgPoolOptions, Row, Connection, Executor};
use tracing::{info, warn};
use chrono::Utc;
use uuid::Uuid;

use crate::utils::error::{AppError, Result};
use crate::core::config::DatabaseConfig;
use super::models::*;

pub struct Repository {
    pool: PgPool,
}

impl Repository {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        // Try to connect to the database
        let pool = match PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&config.url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => {
                // If database doesn't exist, try to create it
                if e.to_string().contains("database \"itrader\" does not exist") {
                    warn!("Database 'itrader' does not exist. Creating it...");
                    Self::create_database(config).await?;
                    
                    // Try to connect again
                    PgPoolOptions::new()
                        .max_connections(config.max_connections)
                        .min_connections(config.min_connections)
                        .connect(&config.url)
                        .await
                        .map_err(|e| AppError::Database(e))?
                } else {
                    return Err(AppError::Database(e));
                }
            }
        };

        info!("Database connection pool created");
        
        // Run migrations
        Self::run_migrations(&pool).await?;
        
        Ok(Self { pool })
    }
    
    async fn create_database(config: &DatabaseConfig) -> Result<()> {
        // Parse the connection URL to get components
        let url = &config.url;
        let db_name = "itrader";
        
        // Connect to postgres database to create our database
        let postgres_url = url.replace("/itrader", "/postgres");
        
        let mut conn = sqlx::postgres::PgConnection::connect(&postgres_url)
            .await
            .map_err(|e| AppError::Database(e))?;
            
        // Create the database
        let query = format!("CREATE DATABASE {}", db_name);
        conn.execute(query.as_str())
            .await
            .map_err(|e| AppError::Database(e))?;
            
        info!("Database '{}' created successfully", db_name);
        Ok(())
    }
    
    async fn run_migrations(pool: &PgPool) -> Result<()> {
        info!("Running database migrations...");
        
        // Create tables if they don't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS gate_accounts (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) UNIQUE NOT NULL,
                password_encrypted TEXT NOT NULL,
                cookies TEXT,
                last_auth TIMESTAMP WITH TIME ZONE,
                balance DECIMAL(20, 8) DEFAULT 0,
                status VARCHAR(50) DEFAULT 'inactive',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bybit_accounts (
                id SERIAL PRIMARY KEY,
                account_name VARCHAR(255) UNIQUE NOT NULL,
                api_key TEXT NOT NULL,
                api_secret_encrypted TEXT NOT NULL,
                active_ads INTEGER DEFAULT 0,
                status VARCHAR(50) DEFAULT 'active',
                last_used TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS orders (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                gate_transaction_id VARCHAR(255) UNIQUE NOT NULL,
                gate_account_id INTEGER REFERENCES gate_accounts(id),
                bybit_account_id INTEGER REFERENCES bybit_accounts(id),
                bybit_order_id VARCHAR(255),
                amount DECIMAL(20, 8) NOT NULL,
                currency VARCHAR(10) NOT NULL,
                fiat_amount DECIMAL(20, 8) NOT NULL,
                fiat_currency VARCHAR(10) NOT NULL,
                rate DECIMAL(20, 8) NOT NULL,
                status VARCHAR(50) NOT NULL,
                buyer_name VARCHAR(255),
                payment_method VARCHAR(100),
                receipt_path TEXT,
                receipt_validated BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS order_pools (
                id SERIAL PRIMARY KEY,
                order_id UUID REFERENCES orders(id),
                pool_type VARCHAR(50) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ai_conversations (
                id SERIAL PRIMARY KEY,
                order_id UUID REFERENCES orders(id) UNIQUE,
                messages JSONB DEFAULT '[]'::jsonb,
                current_stage VARCHAR(50),
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS email_receipts (
                id SERIAL PRIMARY KEY,
                message_id VARCHAR(255) UNIQUE NOT NULL,
                from_email VARCHAR(255),
                subject TEXT,
                body TEXT,
                received_at TIMESTAMP WITH TIME ZONE,
                processed BOOLEAN DEFAULT FALSE,
                attachment_paths TEXT[],
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        info!("Database migrations completed successfully");
        Ok(())
    }

    // Gate Account methods
    pub async fn get_all_gate_accounts(&self) -> Result<Vec<GateAccount>> {
        let rows = sqlx::query("SELECT * FROM gate_accounts ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;

        let accounts = rows.into_iter().map(|row| GateAccount {
            id: row.get("id"),
            email: row.get("email"),
            password_encrypted: row.get("password_encrypted"),
            cookies: row.get("cookies"),
            last_auth: row.get("last_auth"),
            balance: row.get("balance"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();

        Ok(accounts)
    }

    pub async fn get_active_gate_accounts(&self) -> Result<Vec<GateAccount>> {
        let rows = sqlx::query("SELECT * FROM gate_accounts WHERE status = 'active' ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;

        let accounts = rows.into_iter().map(|row| GateAccount {
            id: row.get("id"),
            email: row.get("email"),
            password_encrypted: row.get("password_encrypted"),
            cookies: row.get("cookies"),
            last_auth: row.get("last_auth"),
            balance: row.get("balance"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();

        Ok(accounts)
    }

    pub async fn update_gate_account_status(&self, email: &str, status: &str) -> Result<()> {
        sqlx::query("UPDATE gate_accounts SET status = $1, updated_at = NOW() WHERE email = $2")
            .bind(status)
            .bind(email)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_gate_account_last_auth(&self, email: &str) -> Result<()> {
        sqlx::query("UPDATE gate_accounts SET last_auth = NOW(), updated_at = NOW() WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_gate_account_balance(&self, email: &str, balance: rust_decimal::Decimal) -> Result<()> {
        sqlx::query("UPDATE gate_accounts SET balance = $1, updated_at = NOW() WHERE email = $2")
            .bind(balance)
            .bind(email)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn save_gate_account_cookies(&self, email: &str, cookies: serde_json::Value) -> Result<()> {
        sqlx::query("UPDATE gate_accounts SET cookies = $1, updated_at = NOW() WHERE email = $2")
            .bind(cookies)
            .bind(email)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Bybit Account methods
    pub async fn get_available_bybit_account(&self) -> Result<Option<BybitAccount>> {
        let row = sqlx::query(
            "SELECT * FROM bybit_accounts 
             WHERE status = 'available' AND active_ads < 2 
             ORDER BY active_ads ASC, updated_at ASC 
             LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| BybitAccount {
            id: row.get("id"),
            account_name: row.get("account_name"),
            api_key: row.get("api_key"),
            api_secret_encrypted: row.get("api_secret_encrypted"),
            active_ads: row.get("active_ads"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    pub async fn increment_bybit_active_ads(&self, account_id: i32) -> Result<()> {
        sqlx::query(
            "UPDATE bybit_accounts 
             SET active_ads = active_ads + 1, updated_at = NOW() 
             WHERE id = $1"
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn decrement_bybit_active_ads(&self, account_id: i32) -> Result<()> {
        sqlx::query(
            "UPDATE bybit_accounts 
             SET active_ads = GREATEST(active_ads - 1, 0), updated_at = NOW() 
             WHERE id = $1"
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Order methods
    pub async fn create_order(&self, order: &Order) -> Result<Order> {
        let row = sqlx::query(
            "INSERT INTO orders (
                id, gate_transaction_id, bybit_order_id, gate_account_id, 
                bybit_account_id, amount, currency, fiat_currency, rate, 
                total_fiat, status, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *"
        )
        .bind(order.id)
        .bind(&order.gate_transaction_id)
        .bind(&order.bybit_order_id)
        .bind(order.gate_account_id)
        .bind(order.bybit_account_id)
        .bind(&order.amount)
        .bind(&order.currency)
        .bind(&order.fiat_currency)
        .bind(&order.rate)
        .bind(&order.total_fiat)
        .bind(&order.status)
        .bind(&order.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(Order {
            id: row.get("id"),
            gate_transaction_id: row.get("gate_transaction_id"),
            bybit_order_id: row.get("bybit_order_id"),
            gate_account_id: row.get("gate_account_id"),
            bybit_account_id: row.get("bybit_account_id"),
            amount: row.get("amount"),
            currency: row.get("currency"),
            fiat_currency: row.get("fiat_currency"),
            rate: row.get("rate"),
            total_fiat: row.get("total_fiat"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            completed_at: row.get("completed_at"),
        })
    }

    pub async fn update_order_status(&self, order_id: Uuid, status: &str) -> Result<()> {
        let completed_at = if status == "completed" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query(
            "UPDATE orders 
             SET status = $1, updated_at = NOW(), completed_at = $2 
             WHERE id = $3"
        )
        .bind(status)
        .bind(completed_at)
        .bind(order_id)
        .execute(&self.pool)
        .await?;

        // Also record in status history
        sqlx::query(
            "INSERT INTO order_status_history (order_id, status, details)
             VALUES ($1, $2, $3)"
        )
        .bind(order_id)
        .bind(status)
        .bind(serde_json::json!({}))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_order_bybit_id(&self, order_id: Uuid, bybit_order_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE orders 
             SET bybit_order_id = $1, updated_at = NOW() 
             WHERE id = $2"
        )
        .bind(bybit_order_id)
        .bind(order_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_order_metadata(&self, order_id: Uuid, metadata: serde_json::Value) -> Result<()> {
        sqlx::query(
            "UPDATE orders 
             SET metadata = $1, updated_at = NOW() 
             WHERE id = $2"
        )
        .bind(metadata)
        .bind(order_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_order_by_id(&self, order_id: Uuid) -> Result<Option<Order>> {
        let row = sqlx::query("SELECT * FROM orders WHERE id = $1")
            .bind(order_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|row| Order {
            id: row.get("id"),
            gate_transaction_id: row.get("gate_transaction_id"),
            bybit_order_id: row.get("bybit_order_id"),
            gate_account_id: row.get("gate_account_id"),
            bybit_account_id: row.get("bybit_account_id"),
            amount: row.get("amount"),
            currency: row.get("currency"),
            fiat_currency: row.get("fiat_currency"),
            rate: row.get("rate"),
            total_fiat: row.get("total_fiat"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            completed_at: row.get("completed_at"),
        }))
    }

    pub async fn get_order_by_gate_tx_id(&self, gate_tx_id: &str) -> Result<Option<Order>> {
        let row = sqlx::query("SELECT * FROM orders WHERE gate_transaction_id = $1")
            .bind(gate_tx_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|row| Order {
            id: row.get("id"),
            gate_transaction_id: row.get("gate_transaction_id"),
            bybit_order_id: row.get("bybit_order_id"),
            gate_account_id: row.get("gate_account_id"),
            bybit_account_id: row.get("bybit_account_id"),
            amount: row.get("amount"),
            currency: row.get("currency"),
            fiat_currency: row.get("fiat_currency"),
            rate: row.get("rate"),
            total_fiat: row.get("total_fiat"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            completed_at: row.get("completed_at"),
        }))
    }

    pub async fn get_active_orders(&self) -> Result<Vec<Order>> {
        let rows = sqlx::query(
            "SELECT * FROM orders 
             WHERE status IN ('pending', 'accepted', 'advertised', 'buyer_found', 'chatting', 'payment_received')
             ORDER BY created_at"
        )
        .fetch_all(&self.pool)
        .await?;

        let orders = rows.into_iter().map(|row| Order {
            id: row.get("id"),
            gate_transaction_id: row.get("gate_transaction_id"),
            bybit_order_id: row.get("bybit_order_id"),
            gate_account_id: row.get("gate_account_id"),
            bybit_account_id: row.get("bybit_account_id"),
            amount: row.get("amount"),
            currency: row.get("currency"),
            fiat_currency: row.get("fiat_currency"),
            rate: row.get("rate"),
            total_fiat: row.get("total_fiat"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            completed_at: row.get("completed_at"),
        }).collect();

        Ok(orders)
    }

    // Pool methods
    pub async fn add_to_pool(&self, pool_type: &str, order_id: Uuid, data: serde_json::Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO order_pools (pool_type, order_id, data, status)
             VALUES ($1, $2, $3, 'active')"
        )
        .bind(pool_type)
        .bind(order_id)
        .bind(data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pool_orders(&self, pool_type: &str) -> Result<Vec<OrderPool>> {
        let rows = sqlx::query(
            "SELECT * FROM order_pools 
             WHERE pool_type = $1 AND status = 'active' 
             ORDER BY created_at"
        )
        .bind(pool_type)
        .fetch_all(&self.pool)
        .await?;

        let pools = rows.into_iter().map(|row| OrderPool {
            id: row.get("id"),
            pool_type: row.get("pool_type"),
            order_id: row.get("order_id"),
            data: row.get("data"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();

        Ok(pools)
    }

    pub async fn update_pool_status(&self, pool_id: i32, status: &str) -> Result<()> {
        sqlx::query(
            "UPDATE order_pools 
             SET status = $1, updated_at = NOW() 
             WHERE id = $2"
        )
        .bind(status)
        .bind(pool_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // AI Conversation methods
    pub async fn create_conversation(&self, order_id: Uuid) -> Result<i32> {
        let row = sqlx::query(
            "INSERT INTO ai_conversations (order_id, messages)
             VALUES ($1, '[]'::jsonb)
             RETURNING id"
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    pub async fn add_conversation_message(
        &self, 
        order_id: Uuid, 
        message: serde_json::Value
    ) -> Result<()> {
        sqlx::query(
            "UPDATE ai_conversations 
             SET messages = messages || $1::jsonb, updated_at = NOW() 
             WHERE order_id = $2"
        )
        .bind(message)
        .bind(order_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_conversation(&self, order_id: Uuid) -> Result<Option<AIConversation>> {
        let row = sqlx::query("SELECT * FROM ai_conversations WHERE order_id = $1")
            .bind(order_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|row| AIConversation {
            id: row.get("id"),
            order_id: row.get("order_id"),
            messages: row.get("messages"),
            customer_language: row.get("customer_language"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    // Email Receipt methods
    pub async fn save_email_receipt(&self, receipt: EmailReceipt) -> Result<()> {
        sqlx::query(
            "INSERT INTO email_receipts (
                order_id, email_from, email_subject, receipt_data, 
                ocr_result, is_valid
            ) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(receipt.order_id)
        .bind(&receipt.email_from)
        .bind(&receipt.email_subject)
        .bind(&receipt.receipt_data)
        .bind(&receipt.ocr_result)
        .bind(receipt.is_valid)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    
    pub async fn get_gate_account_by_id(&self, id: i32) -> Result<Option<GateAccount>> {
        let row = sqlx::query("SELECT * FROM gate_accounts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|row| GateAccount {
            id: row.get("id"),
            email: row.get("email"),
            password_encrypted: row.get("password_encrypted"),
            cookies: row.get("cookies"),
            last_auth: row.get("last_auth"),
            balance: row.get("balance"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }
}