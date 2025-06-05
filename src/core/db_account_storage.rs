use sqlx::{PgPool, postgres::PgRow, Row};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::utils::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCookie {
    pub id: String,
    pub email: String,
    pub password_encrypted: String,
    pub status: String,
    pub cookies: Option<serde_json::Value>,
    pub last_auth: Option<DateTime<Utc>>,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BybitSession {
    pub id: String,
    pub api_key: String,
    pub api_secret: String,
    pub status: String,
    pub testnet: bool,
    pub active_ads: i32,
    pub last_error: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct DbAccountStorage {
    pool: Arc<PgPool>,
}

impl DbAccountStorage {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // Gate cookie methods
    pub async fn save_gate_account(&self, login: &str, password: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        sqlx::query!(
            r#"
            INSERT INTO gate_cookies (id, email, password_encrypted, status, balance)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            id,
            login,
            password, // In production, this should be encrypted
            "inactive",
            10000000.0
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(id)
    }

    pub async fn load_gate_account(&self, id: &str) -> Result<Option<(String, String, Option<serde_json::Value>)>> {
        let cookie = sqlx::query!(
            r#"
            SELECT email, password_encrypted, cookies
            FROM gate_cookies
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        match cookie {
            Some(c) => Ok(Some((c.email, c.password_encrypted, c.cookies))),
            None => Ok(None),
        }
    }

    pub async fn update_gate_cookies(&self, id: &str, cookies: serde_json::Value) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE gate_cookies
            SET cookies = $1, last_auth = CURRENT_TIMESTAMP, status = 'active',
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            cookies,
            id
        )
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Gate cookie not found".to_string()));
        }

        Ok(())
    }

    pub async fn update_gate_balance(&self, id: &str, balance: f64) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE gate_cookies
            SET balance = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            balance,
            id
        )
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Gate cookie not found".to_string()));
        }

        Ok(())
    }

    pub async fn list_gate_accounts(&self) -> Result<Vec<GateCookie>> {
        let cookies = sqlx::query_as!(
            GateCookie,
            r#"
            SELECT id, email, password_encrypted, status, cookies,
                   last_auth, balance, created_at, updated_at
            FROM gate_cookies
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(cookies)
    }

    // Bybit session methods
    pub async fn save_bybit_account(&self, api_key: &str, api_secret: &str) -> Result<String> {
        let id = api_key.to_string(); // Use API key as ID for consistency

        sqlx::query!(
            r#"
            INSERT INTO bybit_sessions (id, api_key, api_secret, status)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE 
            SET api_secret = EXCLUDED.api_secret,
                updated_at = CURRENT_TIMESTAMP
            "#,
            id,
            api_key,
            api_secret, // In production, this should be encrypted
            "active"
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(id)
    }

    pub async fn load_bybit_account(&self, id: &str) -> Result<Option<(String, String)>> {
        let session = sqlx::query!(
            r#"
            SELECT api_key, api_secret
            FROM bybit_sessions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        match session {
            Some(s) => Ok(Some((s.api_key, s.api_secret))),
            None => Ok(None),
        }
    }

    pub async fn update_bybit_active_ads(&self, id: &str, active_ads: i32) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE bybit_sessions
            SET active_ads = $1, last_login = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            active_ads,
            id
        )
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Bybit session not found".to_string()));
        }

        Ok(())
    }

    pub async fn list_bybit_accounts(&self) -> Result<Vec<BybitSession>> {
        let sessions = sqlx::query_as!(
            BybitSession,
            r#"
            SELECT id, api_key, api_secret, status, testnet,
                   active_ads, last_error, last_login, created_at, updated_at
            FROM bybit_sessions
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(sessions)
    }

    // Settings methods
    pub async fn get_admin_token(&self) -> Result<Option<String>> {
        let result = sqlx::query!(
            "SELECT admin_token FROM settings WHERE id = 1"
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(result.and_then(|r| r.admin_token))
    }

    pub async fn update_admin_token(&self, token: &str) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO settings (id, admin_token)
            VALUES (1, $1)
            ON CONFLICT (id) DO UPDATE 
            SET admin_token = EXCLUDED.admin_token,
                updated_at = CURRENT_TIMESTAMP
            "#,
            token
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    pub async fn get_settings(&self) -> Result<serde_json::Value> {
        let settings = sqlx::query!(
            r#"
            SELECT admin_token, balance_update_interval, gate_relogin_interval,
                   rate_limit_per_minute, payment_methods, alternate_payments,
                   ocr_validation, cleanup_days, receipt_email
            FROM settings
            WHERE id = 1
            "#
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        if let Some(s) = settings {
            Ok(serde_json::json!({
                "admin_token": s.admin_token,
                "balance_update_interval": s.balance_update_interval,
                "gate_relogin_interval": s.gate_relogin_interval,
                "rate_limit_per_minute": s.rate_limit_per_minute,
                "payment_methods": s.payment_methods,
                "alternate_payments": s.alternate_payments,
                "ocr_validation": s.ocr_validation,
                "cleanup_days": s.cleanup_days,
                "receipt_email": s.receipt_email
            }))
        } else {
            Ok(serde_json::json!({}))
        }
    }
}