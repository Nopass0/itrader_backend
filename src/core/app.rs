use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{info, error, warn};

use crate::utils::error::{Result, AppError};
use crate::core::config::Config;
use crate::core::state::AppState;
use crate::core::orchestrator::Orchestrator;
use crate::api::server::ApiServer;
use crate::gate::models::Cookie;

pub struct Application {
    state: Arc<AppState>,
    tasks: Vec<JoinHandle<()>>,
}

impl Application {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing application");
        
        let state = AppState::new(config).await?;
        
        Ok(Self {
            state,
            tasks: Vec::new(),
        })
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Starting application");
        
        // Initialize Gate accounts
        self.initialize_gate_accounts().await?;
        
        // Initialize Bybit accounts
        self.initialize_bybit_accounts().await?;
        
        // Start background tasks
        self.start_background_tasks().await?;
        
        // Start API server
        self.start_api_server().await?;
        
        // Start orchestrator or auto-trader based on config
        if self.state.config.auto_trader.enabled {
            self.start_auto_trader().await?;
        } else {
            self.start_orchestrator().await?;
        }
        
        // Wait for shutdown signal
        self.state.wait_for_shutdown().await;
        
        info!("Shutdown signal received, cleaning up...");
        
        // Shutdown tasks
        self.shutdown().await?;
        
        Ok(())
    }

    async fn initialize_gate_accounts(&self) -> Result<()> {
        info!("Initializing Gate.io accounts");
        
        // Get all Gate accounts from account storage
        let accounts = self.state.account_storage.list_gate_accounts().await?;
        
        if accounts.is_empty() {
            warn!("No active Gate.io accounts found");
            
            // Check if we have test cookies for development
            if let Ok(cookie_data) = tokio::fs::read_to_string("test_data/gate_cookie.json").await {
                if let Ok(cookies) = serde_json::from_str::<Vec<Cookie>>(&cookie_data) {
                    info!("Loading Gate.io cookies from test data");
                    
                    // Add test account to account manager
                    let account_id = self.state.account_manager.add_gate_account(
                        "test@example.com".to_string(),
                        "test_password".to_string()
                    ).await?;
                    
                    // Update cookies
                    self.state.account_manager.update_gate_account_cookies(
                        account_id,
                        serde_json::to_value(&cookies)?
                    ).await?;
                    
                    // Load cookies to Gate manager
                    self.state.gate_manager.load_cookies_from_file(cookies).await?;
                    
                    // Set initial balance
                    self.state.gate_manager.set_account_balance(
                        "test@example.com", 
                        self.state.config.gate.target_balance
                    ).await?;
                }
            }
        } else {
            // Authenticate all accounts from account storage
            for account in accounts {
                if let Some(cookies) = account.cookies {
                    if let Ok(cookie_vec) = serde_json::from_value::<Vec<Cookie>>(cookies) {
                        info!("Loading cookies for account: {}", account.login);
                        self.state.gate_manager.load_cookies_from_file(cookie_vec).await?;
                    }
                } else {
                    info!("Authenticating account: {}", account.login);
                    self.state.gate_manager.authenticate_account(&account.id).await?;
                }
            }
        }
        
        Ok(())
    }

    async fn initialize_bybit_accounts(&self) -> Result<()> {
        info!("Initializing Bybit P2P accounts");
        
        // Get all Bybit accounts from account storage
        let accounts = self.state.account_storage.list_bybit_accounts().await?;
        
        if accounts.is_empty() {
            warn!("No Bybit accounts found");
            
            // Check if we have test credentials for development
            if let Ok(cred_data) = tokio::fs::read_to_string("test_data/bybit_creditials.json").await {
                if let Ok(creds) = serde_json::from_str::<serde_json::Value>(&cred_data) {
                    if let (Some(api_key), Some(api_secret)) = (
                        creds["api_key"].as_str(),
                        creds["api_secret"].as_str()
                    ) {
                        info!("Loading Bybit credentials from test data");
                        
                        // Add test account to account manager
                        let account_id = self.state.account_manager.add_bybit_account(
                            "test_bybit".to_string(),
                            api_key.to_string(),
                            api_secret.to_string()
                        ).await?;
                        
                        let client = Arc::new(crate::bybit::BybitP2PClient::new(
                            self.state.config.bybit.rest_url.clone(),
                            api_key.to_string(),
                            api_secret.to_string(),
                            self.state.rate_limiter.clone(),
                            self.state.config.bybit.max_ads_per_account,
                        ).await?);
                        
                        // Sync server time for accurate signatures
                        if let Err(e) = client.get_server_time().await {
                            warn!("Failed to sync Bybit server time: {}", e);
                        }
                        
                        self.state.add_bybit_client(account_id, client).await;
                    }
                }
            }
        } else {
            // Initialize all Bybit accounts
            for account in accounts {
                info!("Initializing Bybit account: {}", account.api_key);
                
                let api_secret = account.api_secret
                    .ok_or_else(|| AppError::Config("Bybit API secret not found".to_string()))?;
                
                let client = Arc::new(crate::bybit::BybitP2PClient::new(
                    self.state.config.bybit.rest_url.clone(),
                    account.api_key.clone(),
                    api_secret,
                    self.state.rate_limiter.clone(),
                    self.state.config.bybit.max_ads_per_account,
                ).await?);
                
                // Sync server time for accurate signatures
                if let Err(e) = client.get_server_time().await {
                    warn!("Failed to sync Bybit server time for {}: {}", account.api_key, e);
                }
                
                // Use a simple hash of the ID string as i32
                let account_id = account.id.bytes().fold(0i32, |acc, b| acc.wrapping_add(b as i32));
                self.state.add_bybit_client(account_id, client).await;
            }
        }
        
        Ok(())
    }

    async fn start_background_tasks(&mut self) -> Result<()> {
        info!("Starting background tasks");
        
        // Session refresh task
        let gate_manager = self.state.gate_manager.clone();
        let session_task = tokio::spawn(async move {
            gate_manager.start_session_refresh_task().await;
        });
        self.tasks.push(session_task);
        
        // Balance monitor task
        let gate_manager = self.state.gate_manager.clone();
        let balance_task = tokio::spawn(async move {
            gate_manager.start_balance_monitor_task().await;
        });
        self.tasks.push(balance_task);
        
        info!("Background tasks started");
        Ok(())
    }

    async fn start_api_server(&mut self) -> Result<()> {
        info!("Starting API server");
        
        let api_server = ApiServer::new(self.state.clone());
        let server_task = tokio::spawn(async move {
            if let Err(e) = api_server.run().await {
                error!("API server error: {}", e);
            }
        });
        self.tasks.push(server_task);
        
        info!("API server started on {}:{}", 
            self.state.config.server.host, 
            self.state.config.server.port
        );
        
        Ok(())
    }

    async fn start_orchestrator(&mut self) -> Result<()> {
        info!("Starting order orchestrator");
        
        let orchestrator = Arc::new(Orchestrator::new(self.state.clone())?);
        let orchestrator_task = tokio::spawn(async move {
            if let Err(e) = orchestrator.start_processing().await {
                error!("Orchestrator error: {}", e);
            }
        });
        self.tasks.push(orchestrator_task);
        
        info!("Order orchestrator started");
        Ok(())
    }

    async fn start_auto_trader(&mut self) -> Result<()> {
        info!("Starting auto-trader");
        
        let trader_state = self.state.clone();
        let trader_config = self.state.config.auto_trader.clone();
        
        let trader_task = tokio::spawn(async move {
            let mut auto_trader = crate::core::auto_trader::AutoTrader::new(
                trader_state,
                trader_config
            );
            if let Err(e) = auto_trader.run().await {
                error!("Auto-trader error: {}", e);
            }
        });
        self.tasks.push(trader_task);
        
        info!("Auto-trader started");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down application");
        
        // Set all Gate balances to 0
        if let Err(e) = self.state.gate_manager.shutdown().await {
            error!("Error during Gate manager shutdown: {}", e);
        }
        
        // Cancel all tasks
        for task in &self.tasks {
            task.abort();
        }
        
        // Wait a bit for tasks to finish
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        info!("Application shutdown complete");
        Ok(())
    }
}