use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgRow, Row};
use std::sync::Arc;

use crate::utils::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateAccount {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub balance: f64,
    pub status: String,
    pub last_auth: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BybitAccount {
    pub id: i32,
    pub account_name: String,
    pub api_key: String,
    pub api_secret: String,
    pub active_ads: i32,
    pub status: String,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct DbAccountManager {
    pool: Arc<PgPool>,
}

impl DbAccountManager {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // Gate account management
    pub async fn add_gate_account(&self, email: String, password: String) -> Result<i32> {
        // Check if account already exists
        let existing = sqlx::query!(
            "SELECT id FROM gate_accounts WHERE email = $1",
            email
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("Gate account already exists".to_string()));
        }

        let result = sqlx::query!(
            r#"
            INSERT INTO gate_accounts (email, password, balance, status)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            email,
            password, // In production, this should be encrypted
            10000000.0,
            "active"
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(result.id)
    }

    pub async fn get_gate_account(&self, id: i32) -> Result<Option<GateAccount>> {
        let account = sqlx::query_as!(
            GateAccount,
            r#"
            SELECT id, email, password, balance, status, 
                   last_auth, created_at, updated_at
            FROM gate_accounts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(account)
    }

    pub async fn get_gate_account_by_email(&self, email: &str) -> Result<Option<GateAccount>> {
        let account = sqlx::query_as!(
            GateAccount,
            r#"
            SELECT id, email, password, balance, status,
                   last_auth, created_at, updated_at
            FROM gate_accounts
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(account)
    }

    pub async fn update_gate_account_status(&self, id: i32, status: &str) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE gate_accounts
            SET status = $1, last_auth = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            status,
            id
        )
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Gate account not found".to_string()));
        }

        Ok(())
    }

    pub async fn update_gate_balance(&self, id: i32, balance: f64) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE gate_accounts
            SET balance = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            balance,
            id
        )
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Gate account not found".to_string()));
        }

        Ok(())
    }

    pub async fn get_active_gate_accounts(&self) -> Result<Vec<GateAccount>> {
        let accounts = sqlx::query_as!(
            GateAccount,
            r#"
            SELECT id, email, password, balance, status,
                   last_auth, created_at, updated_at
            FROM gate_accounts
            WHERE status = 'active'
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(accounts)
    }

    // Bybit account management
    pub async fn add_bybit_account(&self, account_name: String, api_key: String, api_secret: String) -> Result<i32> {
        // Check if account already exists
        let existing = sqlx::query!(
            "SELECT id FROM bybit_accounts WHERE account_name = $1",
            account_name
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("Bybit account already exists".to_string()));
        }

        let result = sqlx::query!(
            r#"
            INSERT INTO bybit_accounts (account_name, api_key, api_secret, active_ads, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            account_name,
            api_key,
            api_secret, // In production, this should be encrypted
            0,
            "available"
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(result.id)
    }

    pub async fn get_bybit_account(&self, id: i32) -> Result<Option<BybitAccount>> {
        let account = sqlx::query_as!(
            BybitAccount,
            r#"
            SELECT id, account_name, api_key, api_secret, active_ads,
                   status, last_used, created_at, updated_at
            FROM bybit_accounts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(account)
    }

    pub async fn get_available_bybit_account(&self) -> Result<Option<BybitAccount>> {
        let account = sqlx::query_as!(
            BybitAccount,
            r#"
            SELECT id, account_name, api_key, api_secret, active_ads,
                   status, last_used, created_at, updated_at
            FROM bybit_accounts
            WHERE status = 'available' AND active_ads < 4
            ORDER BY active_ads ASC, last_used ASC NULLS FIRST
            LIMIT 1
            "#
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(account)
    }

    pub async fn update_bybit_active_ads(&self, id: i32, active_ads: i32) -> Result<()> {
        let status = if active_ads >= 4 {
            "max_ads_reached"
        } else if active_ads > 0 {
            "busy"
        } else {
            "available"
        };

        let result = sqlx::query!(
            r#"
            UPDATE bybit_accounts
            SET active_ads = $1, status = $2, last_used = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
            "#,
            active_ads,
            status,
            id
        )
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Bybit account not found".to_string()));
        }

        Ok(())
    }

    pub async fn get_all_bybit_accounts(&self) -> Result<Vec<BybitAccount>> {
        let accounts = sqlx::query_as!(
            BybitAccount,
            r#"
            SELECT id, account_name, api_key, api_secret, active_ads,
                   status, last_used, created_at, updated_at
            FROM bybit_accounts
            ORDER BY id
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(accounts)
    }

    // Utility methods
    pub async fn get_stats(&self) -> Result<AccountStats> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM gate_accounts WHERE status = 'active') as gate_active,
                (SELECT COUNT(*) FROM gate_accounts) as gate_total,
                (SELECT COUNT(*) FROM bybit_accounts WHERE status = 'available') as bybit_available,
                (SELECT COUNT(*) FROM bybit_accounts) as bybit_total,
                (SELECT COALESCE(SUM(active_ads), 0) FROM bybit_accounts) as total_active_ads
            "#
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(AccountStats {
            gate_active: stats.gate_active.unwrap_or(0) as usize,
            gate_total: stats.gate_total.unwrap_or(0) as usize,
            bybit_available: stats.bybit_available.unwrap_or(0) as usize,
            bybit_total: stats.bybit_total.unwrap_or(0) as usize,
            total_active_ads: stats.total_active_ads.unwrap_or(0) as i32,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStats {
    pub gate_active: usize,
    pub gate_total: usize,
    pub bybit_available: usize,
    pub bybit_total: usize,
    pub total_active_ads: i32,
}