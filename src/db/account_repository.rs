use crate::db::models::{GateAccount, BybitAccount};
use crate::utils::error::{Result, AppError};
use sqlx::{PgPool, postgres::PgRow, Row};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use tracing::{info, error, debug};

pub struct AccountRepository {
    pool: PgPool,
}

impl AccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========== Gate.io Account Methods ==========

    pub async fn create_gate_account(
        &self,
        email: &str,
        password: &str,
        balance: Decimal,
    ) -> Result<GateAccount> {
        let account = sqlx::query_as!(
            GateAccount,
            r#"
            INSERT INTO gate_accounts (email, password, balance, status)
            VALUES ($1, $2, $3, 'active')
            RETURNING *
            "#,
            email,
            password,
            balance
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create Gate account: {}", e);
            AppError::Database(e.to_string())
        })?;

        info!("Created Gate account: {}", email);
        Ok(account)
    }

    pub async fn get_gate_account(&self, id: i32) -> Result<Option<GateAccount>> {
        let account = sqlx::query_as!(
            GateAccount,
            "SELECT * FROM gate_accounts WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(account)
    }

    pub async fn get_gate_account_by_email(&self, email: &str) -> Result<Option<GateAccount>> {
        let account = sqlx::query_as!(
            GateAccount,
            "SELECT * FROM gate_accounts WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(account)
    }

    pub async fn list_gate_accounts(&self) -> Result<Vec<GateAccount>> {
        let accounts = sqlx::query_as!(
            GateAccount,
            "SELECT * FROM gate_accounts ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(accounts)
    }

    pub async fn list_active_gate_accounts(&self) -> Result<Vec<GateAccount>> {
        let accounts = sqlx::query_as!(
            GateAccount,
            "SELECT * FROM gate_accounts WHERE status = 'active' ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(accounts)
    }

    pub async fn update_gate_account(
        &self,
        id: i32,
        email: Option<&str>,
        password: Option<&str>,
        balance: Option<Decimal>,
        status: Option<&str>,
    ) -> Result<GateAccount> {
        let mut query_parts = vec!["UPDATE gate_accounts SET updated_at = CURRENT_TIMESTAMP"];
        let mut params: Vec<String> = vec![];
        let mut param_count = 1;

        if email.is_some() {
            params.push(format!("email = ${}", param_count));
            param_count += 1;
        }
        if password.is_some() {
            params.push(format!("password = ${}", param_count));
            param_count += 1;
        }
        if balance.is_some() {
            params.push(format!("balance = ${}", param_count));
            param_count += 1;
        }
        if status.is_some() {
            params.push(format!("status = ${}", param_count));
            param_count += 1;
        }

        if params.is_empty() {
            return self.get_gate_account(id).await?
                .ok_or_else(|| AppError::NotFound("Gate account not found".to_string()));
        }

        let query = format!(
            "{}, {} WHERE id = ${} RETURNING *",
            query_parts.join(""),
            params.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query(&query);
        
        if let Some(e) = email {
            query_builder = query_builder.bind(e);
        }
        if let Some(p) = password {
            query_builder = query_builder.bind(p);
        }
        if let Some(b) = balance {
            query_builder = query_builder.bind(b);
        }
        if let Some(s) = status {
            query_builder = query_builder.bind(s);
        }
        query_builder = query_builder.bind(id);

        let row: PgRow = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let account = GateAccount {
            id: row.get("id"),
            email: row.get("email"),
            password: row.get("password"),
            balance: row.get("balance"),
            status: row.get("status"),
            cookies: row.get("cookies"),
            last_auth: row.get("last_auth"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        info!("Updated Gate account: {}", account.email);
        Ok(account)
    }

    pub async fn update_gate_cookies(&self, id: i32, cookies: serde_json::Value) -> Result<()> {
        sqlx::query!(
            "UPDATE gate_accounts SET cookies = $1, last_auth = CURRENT_TIMESTAMP WHERE id = $2",
            cookies,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        debug!("Updated cookies for Gate account ID: {}", id);
        Ok(())
    }

    pub async fn delete_gate_account(&self, id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM gate_accounts WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        info!("Deleted Gate account ID: {}", id);
        Ok(())
    }

    pub async fn count_gate_accounts(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM gate_accounts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(count)
    }

    // ========== Bybit Account Methods ==========

    pub async fn create_bybit_account(
        &self,
        account_name: &str,
        api_key: &str,
        api_secret: &str,
    ) -> Result<BybitAccount> {
        let account = sqlx::query_as!(
            BybitAccount,
            r#"
            INSERT INTO bybit_accounts (account_name, api_key, api_secret, active_ads, status)
            VALUES ($1, $2, $3, 0, 'available')
            RETURNING *
            "#,
            account_name,
            api_key,
            api_secret
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create Bybit account: {}", e);
            AppError::Database(e.to_string())
        })?;

        info!("Created Bybit account: {}", account_name);
        Ok(account)
    }

    pub async fn get_bybit_account(&self, id: i32) -> Result<Option<BybitAccount>> {
        let account = sqlx::query_as!(
            BybitAccount,
            "SELECT * FROM bybit_accounts WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(account)
    }

    pub async fn get_bybit_account_by_name(&self, account_name: &str) -> Result<Option<BybitAccount>> {
        let account = sqlx::query_as!(
            BybitAccount,
            "SELECT * FROM bybit_accounts WHERE account_name = $1",
            account_name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(account)
    }

    pub async fn list_bybit_accounts(&self) -> Result<Vec<BybitAccount>> {
        let accounts = sqlx::query_as!(
            BybitAccount,
            "SELECT * FROM bybit_accounts ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(accounts)
    }

    pub async fn list_available_bybit_accounts(&self) -> Result<Vec<BybitAccount>> {
        let accounts = sqlx::query_as!(
            BybitAccount,
            "SELECT * FROM bybit_accounts WHERE status = 'available' ORDER BY active_ads ASC, id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(accounts)
    }

    pub async fn get_least_used_bybit_account(&self) -> Result<Option<BybitAccount>> {
        let account = sqlx::query_as!(
            BybitAccount,
            r#"
            SELECT * FROM bybit_accounts 
            WHERE status = 'available' 
            ORDER BY active_ads ASC, last_used ASC NULLS FIRST
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(account)
    }

    pub async fn update_bybit_account(
        &self,
        id: i32,
        account_name: Option<&str>,
        api_key: Option<&str>,
        api_secret: Option<&str>,
        active_ads: Option<i32>,
        status: Option<&str>,
    ) -> Result<BybitAccount> {
        let mut query_parts = vec!["UPDATE bybit_accounts SET updated_at = CURRENT_TIMESTAMP"];
        let mut params: Vec<String> = vec![];
        let mut param_count = 1;

        if account_name.is_some() {
            params.push(format!("account_name = ${}", param_count));
            param_count += 1;
        }
        if api_key.is_some() {
            params.push(format!("api_key = ${}", param_count));
            param_count += 1;
        }
        if api_secret.is_some() {
            params.push(format!("api_secret = ${}", param_count));
            param_count += 1;
        }
        if active_ads.is_some() {
            params.push(format!("active_ads = ${}", param_count));
            param_count += 1;
        }
        if status.is_some() {
            params.push(format!("status = ${}", param_count));
            param_count += 1;
        }

        if params.is_empty() {
            return self.get_bybit_account(id).await?
                .ok_or_else(|| AppError::NotFound("Bybit account not found".to_string()));
        }

        let query = format!(
            "{}, {} WHERE id = ${} RETURNING *",
            query_parts.join(""),
            params.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query(&query);
        
        if let Some(n) = account_name {
            query_builder = query_builder.bind(n);
        }
        if let Some(k) = api_key {
            query_builder = query_builder.bind(k);
        }
        if let Some(s) = api_secret {
            query_builder = query_builder.bind(s);
        }
        if let Some(a) = active_ads {
            query_builder = query_builder.bind(a);
        }
        if let Some(s) = status {
            query_builder = query_builder.bind(s);
        }
        query_builder = query_builder.bind(id);

        let row: PgRow = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let account = BybitAccount {
            id: row.get("id"),
            account_name: row.get("account_name"),
            api_key: row.get("api_key"),
            api_secret: row.get("api_secret"),
            active_ads: row.get("active_ads"),
            status: row.get("status"),
            last_used: row.get("last_used"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        info!("Updated Bybit account: {}", account.account_name);
        Ok(account)
    }

    pub async fn update_bybit_last_used(&self, id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE bybit_accounts SET last_used = CURRENT_TIMESTAMP WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        debug!("Updated last_used for Bybit account ID: {}", id);
        Ok(())
    }

    pub async fn increment_bybit_active_ads(&self, id: i32) -> Result<i32> {
        let result: i32 = sqlx::query_scalar(
            "UPDATE bybit_accounts SET active_ads = active_ads + 1 WHERE id = $1 RETURNING active_ads"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        debug!("Incremented active_ads for Bybit account ID: {} to {}", id, result);
        Ok(result)
    }

    pub async fn decrement_bybit_active_ads(&self, id: i32) -> Result<i32> {
        let result: i32 = sqlx::query_scalar(
            "UPDATE bybit_accounts SET active_ads = GREATEST(active_ads - 1, 0) WHERE id = $1 RETURNING active_ads"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        debug!("Decremented active_ads for Bybit account ID: {} to {}", id, result);
        Ok(result)
    }

    pub async fn delete_bybit_account(&self, id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM bybit_accounts WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        info!("Deleted Bybit account ID: {}", id);
        Ok(())
    }

    pub async fn count_bybit_accounts(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bybit_accounts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(count)
    }

    // ========== Statistics Methods ==========

    pub async fn get_total_gate_balance(&self) -> Result<Decimal> {
        let balance: Option<Decimal> = sqlx::query_scalar("SELECT SUM(balance) FROM gate_accounts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(balance.unwrap_or_else(|| Decimal::new(0, 0)))
    }

    pub async fn get_total_active_ads(&self) -> Result<i64> {
        let count: Option<i64> = sqlx::query_scalar("SELECT SUM(active_ads) FROM bybit_accounts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(count.unwrap_or(0))
    }
}