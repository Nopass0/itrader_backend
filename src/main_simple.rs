use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use colored::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("{}", "==================================".bright_blue());
    println!("{}", "    iTrader Auto-Trader System    ".bright_blue().bold());
    println!("{}", "==================================".bright_blue());
    println!();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let auto_mode = args.contains(&"--auto".to_string());
    
    // Display mode information
    if auto_mode {
        println!("{}", "🤖 Starting in AUTOMATIC mode".bright_red().bold());
        println!("{}", "⚠️  All actions will be auto-confirmed!".bright_yellow());
        println!();
    } else {
        println!("{}", "👤 Starting in MANUAL mode".bright_green().bold());
        println!("{}", "✅ Actions require confirmation".bright_green());
        println!("{}", "💡 To run in automatic mode, use: --auto".bright_cyan());
        println!();
    }

    // Simple demo loop
    info!("System initialized. Starting demo loop...");
    
    let mut cycle = 0;
    loop {
        cycle += 1;
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        
        println!("\n{}", format!("=== Cycle {} ===", cycle).bright_white().bold());
        
        // Simulate finding a transaction
        if cycle % 3 == 0 {
            println!("{}", "📊 Found new pending transaction!".bright_yellow());
            
            if !auto_mode {
                // Show confirmation dialog
                println!("\n{}", "================================================================================".bright_yellow());
                println!("{}", "⚠️  ACTION REQUIRED: Create Virtual Transaction".bright_yellow().bold());
                println!("{}", "================================================================================".bright_yellow());
                
                println!("\n{}", "📋 Details:".bright_cyan());
                println!("  {} {}", "Transaction ID:".bright_white().bold(), "DEMO-12345".bright_green());
                println!("  {} {}", "Amount:".bright_white().bold(), "75000.00 RUB".bright_green());
                println!("  {} {}", "Phone:".bright_white().bold(), "+7900******67".bright_green());
                println!("  {} {}", "Bank:".bright_white().bold(), "Tinkoff".bright_green());
                println!("  {} {}", "Action:".bright_white().bold(), "Accept and create Bybit ad".bright_green());
                
                println!("\n{}", "❓ Do you want to proceed? (yes/no):".bright_white());
                print!("   {} ", "Enter your choice:".bright_cyan());
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_lowercase();
                
                if input == "yes" || input == "y" || input == "да" {
                    println!("{}", "✅ Action confirmed!".bright_green());
                    info!("Transaction DEMO-12345 processed");
                } else {
                    println!("{}", "❌ Action cancelled!".bright_red());
                    info!("Transaction DEMO-12345 skipped");
                }
            } else {
                println!("{}", "🤖 Auto-processing transaction...".bright_blue());
                info!("Transaction DEMO-12345 auto-processed");
            }
        }
        
        println!("{}", "💤 Waiting for next check...".dim());
    }
}