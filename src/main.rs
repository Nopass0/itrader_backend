use anyhow::Result;
use dotenv::dotenv;
use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;

mod api;
mod core;
mod utils;
mod gate;
mod bybit;
mod ai;
mod ocr;
// mod email;
mod db;

use crate::core::{app::Application, config::Config, account_setup::AccountSetup};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,itrader_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting iTrader Backend...");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let auto_mode = args.contains(&"--auto".to_string());
    
    // Display mode information
    if auto_mode {
        warn!("🤖 Starting in AUTOMATIC mode - all actions will be auto-confirmed!");
        warn!("⚠️  Transactions will be processed WITHOUT manual confirmation!");
    } else {
        info!("👤 Starting in MANUAL mode - actions require confirmation");
        info!("💡 To run in automatic mode, use: cargo run -- --auto");
    }

    // Load configuration and override auto_confirm based on command line
    let mut config = Config::load()?;
    config.auto_trader.auto_confirm = auto_mode;
    
    // Create application state first to get access to repositories
    let state = core::state::AppState::new(config.clone()).await?;
    
    // Check and ensure accounts exist
    let account_setup = AccountSetup::new(state.account_repository.clone());
    account_setup.ensure_accounts_exist().await?;
    account_setup.show_account_summary().await?;
    
    // Create and start application with the state
    let app = Application::from_state(state).await?;
    
    // Spawn the application
    let app_handle = tokio::spawn(async move {
        if let Err(e) = app.run().await {
            tracing::error!("Application error: {}", e);
        }
    });

    // Wait for shutdown signal
    shutdown_signal().await;
    
    info!("Shutting down gracefully...");
    
    // Cancel the application task
    app_handle.abort();
    
    // Wait a bit for graceful shutdown
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    info!("Shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C signal");
        },
        _ = terminate => {
            warn!("Received terminate signal");
        },
    }
}