use crate::db::AccountRepository;
use crate::utils::error::Result;
use std::sync::Arc;
use tracing::{info, warn};
use dialoguer::{Input, Password, Confirm, theme::ColorfulTheme};
use rust_decimal::Decimal;
use std::str::FromStr;

pub struct AccountSetup {
    account_repository: Arc<AccountRepository>,
}

impl AccountSetup {
    pub fn new(account_repository: Arc<AccountRepository>) -> Self {
        Self { account_repository }
    }

    pub async fn ensure_accounts_exist(&self) -> Result<()> {
        // Check if we have any accounts
        let gate_count = self.account_repository.count_gate_accounts().await?;
        let bybit_count = self.account_repository.count_bybit_accounts().await?;

        info!("Found {} Gate.io accounts and {} Bybit accounts", gate_count, bybit_count);

        // If no accounts exist, prompt to add them
        if gate_count == 0 || bybit_count == 0 {
            println!("\n⚠️  No accounts configured!");
            println!("The system requires at least one Gate.io and one Bybit account to operate.\n");

            if gate_count == 0 {
                println!("📌 No Gate.io accounts found.");
                if self.prompt_add_gate_account().await? {
                    while self.prompt_add_another_gate_account().await? {
                        // Continue adding accounts
                    }
                }
            }

            if bybit_count == 0 {
                println!("\n📌 No Bybit accounts found.");
                if self.prompt_add_bybit_account().await? {
                    while self.prompt_add_another_bybit_account().await? {
                        // Continue adding accounts
                    }
                }
            }

            // Re-check counts
            let new_gate_count = self.account_repository.count_gate_accounts().await?;
            let new_bybit_count = self.account_repository.count_bybit_accounts().await?;

            if new_gate_count == 0 || new_bybit_count == 0 {
                warn!("No accounts configured. The system cannot operate without accounts.");
                return Err(crate::utils::error::AppError::Configuration(
                    "At least one Gate.io and one Bybit account is required".to_string()
                ));
            }
        }

        Ok(())
    }

    async fn prompt_add_gate_account(&self) -> Result<bool> {
        let theme = ColorfulTheme::default();
        
        let add = Confirm::with_theme(&theme)
            .with_prompt("Would you like to add a Gate.io account?")
            .default(true)
            .interact()
            .unwrap_or(false);

        if !add {
            return Ok(false);
        }

        println!("\n=== Add Gate.io Account ===");

        let email: String = Input::with_theme(&theme)
            .with_prompt("Email")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.contains('@') && input.len() > 5 {
                    Ok(())
                } else {
                    Err("Please enter a valid email address")
                }
            })
            .interact_text()
            .unwrap();

        let password: String = Password::with_theme(&theme)
            .with_prompt("Password")
            .interact()
            .unwrap();

        let balance_str: String = Input::with_theme(&theme)
            .with_prompt("Initial balance (RUB)")
            .default("10000000".to_string())
            .interact_text()
            .unwrap();

        let balance = Decimal::from_str(&balance_str)
            .unwrap_or_else(|_| Decimal::new(10000000, 0));

        match self.account_repository.create_gate_account(&email, &password, balance).await {
            Ok(account) => {
                println!("✅ Gate.io account added successfully: {}", account.email);
                Ok(true)
            }
            Err(e) => {
                println!("❌ Failed to add account: {}", e);
                Ok(false)
            }
        }
    }

    async fn prompt_add_another_gate_account(&self) -> Result<bool> {
        let theme = ColorfulTheme::default();
        
        let add_another = Confirm::with_theme(&theme)
            .with_prompt("Would you like to add another Gate.io account?")
            .default(false)
            .interact()
            .unwrap_or(false);

        if add_another {
            self.prompt_add_gate_account().await
        } else {
            Ok(false)
        }
    }

    async fn prompt_add_bybit_account(&self) -> Result<bool> {
        let theme = ColorfulTheme::default();
        
        let add = Confirm::with_theme(&theme)
            .with_prompt("Would you like to add a Bybit account?")
            .default(true)
            .interact()
            .unwrap_or(false);

        if !add {
            return Ok(false);
        }

        println!("\n=== Add Bybit Account ===");

        let account_name: String = Input::with_theme(&theme)
            .with_prompt("Account name")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.len() > 2 {
                    Ok(())
                } else {
                    Err("Account name must be at least 3 characters")
                }
            })
            .interact_text()
            .unwrap();

        let api_key: String = Input::with_theme(&theme)
            .with_prompt("API Key")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.len() > 10 {
                    Ok(())
                } else {
                    Err("API key seems too short")
                }
            })
            .interact_text()
            .unwrap();

        let api_secret: String = Password::with_theme(&theme)
            .with_prompt("API Secret")
            .interact()
            .unwrap();

        match self.account_repository.create_bybit_account(&account_name, &api_key, &api_secret).await {
            Ok(account) => {
                println!("✅ Bybit account added successfully: {}", account.account_name);
                Ok(true)
            }
            Err(e) => {
                println!("❌ Failed to add account: {}", e);
                Ok(false)
            }
        }
    }

    async fn prompt_add_another_bybit_account(&self) -> Result<bool> {
        let theme = ColorfulTheme::default();
        
        let add_another = Confirm::with_theme(&theme)
            .with_prompt("Would you like to add another Bybit account?")
            .default(false)
            .interact()
            .unwrap_or(false);

        if add_another {
            self.prompt_add_bybit_account().await
        } else {
            Ok(false)
        }
    }

    pub async fn show_account_summary(&self) -> Result<()> {
        let gate_accounts = self.account_repository.list_gate_accounts().await?;
        let bybit_accounts = self.account_repository.list_bybit_accounts().await?;

        println!("\n📊 Account Summary:");
        println!("─────────────────────────────────────");
        
        println!("\n🔐 Gate.io Accounts ({}):", gate_accounts.len());
        for account in &gate_accounts {
            println!("   • {} - Balance: {} RUB - Status: {}", 
                account.email, 
                account.balance,
                account.status
            );
        }

        println!("\n💱 Bybit Accounts ({}):", bybit_accounts.len());
        for account in &bybit_accounts {
            println!("   • {} - Active Ads: {} - Status: {}", 
                account.account_name,
                account.active_ads,
                account.status
            );
        }

        let total_balance = self.account_repository.get_total_gate_balance().await?;
        let total_ads = self.account_repository.get_total_active_ads().await?;

        println!("\n📈 Statistics:");
        println!("   Total Gate Balance: {} RUB", total_balance);
        println!("   Total Active Ads: {}", total_ads);
        println!("─────────────────────────────────────\n");

        Ok(())
    }
}