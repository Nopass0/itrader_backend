// Stub for Python rate fetcher - disabled when python-sdk feature is not enabled
use std::sync::Arc;
use anyhow::Result;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::{info, debug, warn};
use tokio::sync::OnceCell;

static PYTHON_INITIALIZED: OnceCell<()> = OnceCell::const_new();

#[derive(Clone)]
pub struct PythonRateFetcher {
    _api_key: String,
    _api_secret: String,
}

impl PythonRateFetcher {
    pub async fn new(api_key: String, api_secret: String) -> Result<Self> {
        Ok(Self {
            _api_key: api_key,
            _api_secret: api_secret,
        })
    }

    pub async fn get_current_rate(
        &self,
        _fiat: &str,
        _trade_type: &str,
        _payment_methods: Vec<String>
    ) -> Result<Decimal> {
        // Return a default rate for now
        warn!("Python rate fetcher is disabled, returning default rate");
        Ok(Decimal::from_str("100.0")?)
    }
}