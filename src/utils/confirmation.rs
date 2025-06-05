use std::io::{self, Write};
use colored::*;
use serde_json::Value;

pub struct ConfirmationHelper;

impl ConfirmationHelper {
    /// Ask for user confirmation with detailed information
    pub fn confirm_action(action: &str, details: Vec<(&str, String)>) -> bool {
        println!("\n{}", "="
            .repeat(80)
            .bright_yellow());
        println!("{}", format!("⚠️  ACTION REQUIRED: {}", action).bright_yellow().bold());
        println!("{}", "="
            .repeat(80)
            .bright_yellow());
        
        println!("\n{}", "📋 Details:".bright_cyan());
        for (key, value) in details {
            println!("  {} {}", 
                format!("{}:", key).bright_white().bold(),
                value.bright_green()
            );
        }
        
        println!("\n{}", "❓ Do you want to proceed with this action?".bright_white());
        
        loop {
            print!("{}", "   Enter your choice (yes/no): ".bright_cyan());
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            match input.as_str() {
                "yes" | "y" | "да" => {
                    println!("{}", "✅ Action confirmed!".bright_green());
                    return true;
                },
                "no" | "n" | "нет" => {
                    println!("{}", "❌ Action cancelled!".bright_red());
                    return false;
                },
                _ => {
                    println!("{}", "⚠️  Invalid input. Please enter 'yes' or 'no'.".bright_yellow());
                }
            }
        }
    }
    
    /// Confirm transaction creation
    pub fn confirm_transaction(
        gate_tx_id: &str,
        amount: f64,
        currency: &str,
        phone: &str,
        bank: &str,
    ) -> bool {
        let details = vec![
            ("Gate Transaction ID", gate_tx_id.to_string()),
            ("Amount", format!("{:.2} {}", amount, currency)),
            ("Phone Number", phone.to_string()),
            ("Bank", bank.to_string()),
            ("Action", "Accept transaction and create Bybit ad".to_string()),
        ];
        
        Self::confirm_action("Create Virtual Transaction", details)
    }
    
    /// Confirm Bybit ad creation
    pub fn confirm_bybit_ad(
        amount_rub: f64,
        amount_usdt: f64,
        rate: f64,
        payment_method: &str,
        bybit_account: &str,
    ) -> bool {
        let details = vec![
            ("Bybit Account", bybit_account.to_string()),
            ("Amount RUB", format!("{:.2} RUB", amount_rub)),
            ("Amount USDT", format!("{:.2} USDT", amount_usdt)),
            ("Rate", format!("{:.2} RUB/USDT", rate)),
            ("Payment Method", payment_method.to_string()),
            ("Ad Type", "SELL USDT".to_string()),
            ("Duration", "15 minutes".to_string()),
        ];
        
        Self::confirm_action("Create Bybit P2P Advertisement", details)
    }
    
    /// Confirm balance update
    pub fn confirm_balance_update(
        account_email: &str,
        current_balance: f64,
        new_balance: f64,
    ) -> bool {
        let details = vec![
            ("Account", account_email.to_string()),
            ("Current Balance", format!("{:.2} RUB", current_balance)),
            ("New Balance", format!("{:.2} RUB", new_balance)),
            ("Change", format!("{:+.2} RUB", new_balance - current_balance)),
        ];
        
        Self::confirm_action("Update Gate.io Balance", details)
    }
    
    /// Confirm order completion
    pub fn confirm_order_completion(
        order_id: uuid::Uuid,
        gate_tx_id: &str,
        bybit_order_id: &str,
        amount: f64,
        receipt_valid: bool,
    ) -> bool {
        let details = vec![
            ("Order ID", order_id.to_string()),
            ("Gate Transaction", gate_tx_id.to_string()),
            ("Bybit Order", bybit_order_id.to_string()),
            ("Amount", format!("{:.2} RUB", amount)),
            ("Receipt Validation", if receipt_valid { "✅ PASSED".to_string() } else { "❌ FAILED".to_string() }),
            ("Actions", "1. Release funds on Bybit\n               2. Approve transaction on Gate.io".to_string()),
        ];
        
        Self::confirm_action("Complete Order", details)
    }
    
    /// Confirm receipt validation result
    pub fn confirm_receipt_validation(
        expected_amount: f64,
        extracted_amount: f64,
        expected_phone: &str,
        extracted_phone: Option<&str>,
        bank_match: bool,
    ) -> bool {
        let phone_display = extracted_phone.unwrap_or("Not found");
        let phone_match = extracted_phone.map_or(false, |p| p.ends_with(&expected_phone[expected_phone.len()-4..]));
        
        let details = vec![
            ("Expected Amount", format!("{:.2} RUB", expected_amount)),
            ("Extracted Amount", format!("{:.2} RUB", extracted_amount)),
            ("Amount Match", if (expected_amount - extracted_amount).abs() < 0.01 { "✅ YES".to_string() } else { "❌ NO".to_string() }),
            ("Expected Phone (last 4)", expected_phone[expected_phone.len()-4..].to_string()),
            ("Extracted Phone", phone_display.to_string()),
            ("Phone Match", if phone_match { "✅ YES".to_string() } else { "❌ NO".to_string() }),
            ("Bank Match", if bank_match { "✅ YES".to_string() } else { "❌ NO".to_string() }),
        ];
        
        Self::confirm_action("Receipt Validation Result", details)
    }
    
    /// Show error and ask if should retry
    pub fn confirm_retry(action: &str, error: &str) -> bool {
        println!("\n{}", "❌ ERROR OCCURRED".bright_red().bold());
        println!("{}: {}", "Action".bright_white(), action);
        println!("{}: {}", "Error".bright_white(), error.bright_red());
        
        loop {
            print!("{}", "Retry this action? (yes/no): ".bright_cyan());
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            match input.as_str() {
                "yes" | "y" | "да" => return true,
                "no" | "n" | "нет" => return false,
                _ => println!("{}", "Invalid input. Please enter 'yes' or 'no'.".bright_yellow()),
            }
        }
    }
}