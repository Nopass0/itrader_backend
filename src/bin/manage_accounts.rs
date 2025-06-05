use anyhow::Result;
use clap::{Parser, Subcommand};
use itrader_backend::core::accounts::{AccountManager, AccountStatus, BybitAccountStatus};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Path to accounts.json file
    #[arg(short, long, default_value = "data/accounts.json")]
    file: String,
}

#[derive(Subcommand)]
enum Commands {
    /// List all accounts
    List,
    
    /// Add a new Gate.io account
    AddGate {
        /// Email address
        email: String,
        /// Password
        password: String,
    },
    
    /// Add a new Bybit account
    AddBybit {
        /// Account name
        name: String,
        /// API key
        api_key: String,
        /// API secret
        api_secret: String,
    },
    
    /// Show account statistics
    Stats,
    
    /// Update Gate account balance
    UpdateBalance {
        /// Account ID
        id: i32,
        /// New balance
        balance: f64,
    },
    
    /// Remove a Gate account
    RemoveGate {
        /// Account ID
        id: i32,
    },
    
    /// Remove a Bybit account
    RemoveBybit {
        /// Account ID
        id: i32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let manager = AccountManager::new(&cli.file).await?;
    
    match cli.command {
        Commands::List => {
            let stats = manager.get_stats().await?;
            println!("\n=== Account Statistics ===");
            println!("Gate.io accounts: {} active / {} total", stats.gate_active, stats.gate_total);
            println!("Bybit accounts: {} available / {} total", stats.bybit_available, stats.bybit_total);
            println!("Total active ads: {}", stats.total_active_ads);
            
            println!("\n=== Gate.io Accounts ===");
            let gate_accounts = manager.get_active_gate_accounts().await?;
            if gate_accounts.is_empty() {
                println!("No Gate.io accounts found");
            } else {
                for account in gate_accounts {
                    println!("ID: {}, Email: {}, Status: {:?}, Balance: {} RUB", 
                        account.id, account.email, account.status, account.balance);
                }
            }
            
            println!("\n=== Bybit Accounts ===");
            let bybit_accounts = manager.get_all_bybit_accounts().await?;
            if bybit_accounts.is_empty() {
                println!("No Bybit accounts found");
            } else {
                for account in bybit_accounts {
                    println!("ID: {}, Name: {}, Status: {:?}, Active Ads: {}/4", 
                        account.id, account.account_name, account.status, account.active_ads);
                }
            }
        }
        
        Commands::AddGate { email, password } => {
            let id = manager.add_gate_account(email.clone(), password).await?;
            println!("Added Gate.io account: {} with ID: {}", email, id);
        }
        
        Commands::AddBybit { name, api_key, api_secret } => {
            let id = manager.add_bybit_account(name.clone(), api_key, api_secret).await?;
            println!("Added Bybit account: {} with ID: {}", name, id);
        }
        
        Commands::Stats => {
            let stats = manager.get_stats().await?;
            println!("\n=== Account Statistics ===");
            println!("Gate.io:");
            println!("  Active accounts: {}", stats.gate_active);
            println!("  Total accounts: {}", stats.gate_total);
            println!("  Inactive accounts: {}", stats.gate_total - stats.gate_active);
            
            println!("\nBybit:");
            println!("  Available accounts: {}", stats.bybit_available);
            println!("  Total accounts: {}", stats.bybit_total);
            println!("  Busy accounts: {}", stats.bybit_total - stats.bybit_available);
            println!("  Total active ads: {}", stats.total_active_ads);
            println!("  Available ad slots: {}", (stats.bybit_total * 4) as i32 - stats.total_active_ads);
        }
        
        Commands::UpdateBalance { id, balance } => {
            manager.update_gate_balance(id, balance).await?;
            println!("Updated balance for Gate account {} to {} RUB", id, balance);
        }
        
        Commands::RemoveGate { id } => {
            // Note: This would need to be implemented in AccountManager
            println!("Remove Gate account {} - Not implemented yet", id);
        }
        
        Commands::RemoveBybit { id } => {
            // Note: This would need to be implemented in AccountManager
            println!("Remove Bybit account {} - Not implemented yet", id);
        }
    }
    
    Ok(())
}