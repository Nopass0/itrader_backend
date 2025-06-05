use itrader_backend::bybit::python_rate_fetcher::PythonRateFetcher;
use std::env;
use dotenv::dotenv;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment and logging
    dotenv().ok();
    tracing_subscriber::fmt::init();
    
    println!("Bybit P2P Rate Checker - Python SDK Version");
    println!("==========================================");
    
    info!("Initializing Python rate fetcher...");
    
    // Create rate fetcher (no API keys needed for public data)
    let fetcher = PythonRateFetcher::new(
        "dummy_key".to_string(),
        "dummy_secret".to_string(),
        false
    ).await?;
    
    println!("Rate fetcher initialized successfully");
    
    // Get command line argument for amount
    let args: Vec<String> = env::args().collect();
    let amount = if args.len() > 1 {
        args[1].parse::<f64>().unwrap_or(30_000.0)
    } else {
        30_000.0
    };
    
    println!("Checking rate for {} RUB", amount);
    info!("Checking rate for {} RUB", amount);
    
    // Get current rate
    println!("Calling get_current_rate...");
    match fetcher.get_current_rate(amount).await {
        Ok(rate) => {
            println!("Current rate: {} RUB/USDT", rate);
            println!("For {} RUB you would get approximately {:.2} USDT", amount, amount / rate);
            info!("Current rate: {} RUB/USDT", rate);
            info!("For {} RUB you would get approximately {:.2} USDT", amount, amount / rate);
        }
        Err(e) => {
            eprintln!("Error getting rate: {}", e);
        }
    }
    
    // Also show all scenario rates
    info!("\nAll scenario rates:");
    match fetcher.get_all_rates().await {
        Ok(rates) => {
            for (scenario, rate) in rates.iter() {
                info!("  {}: {} RUB/USDT", scenario, rate);
            }
        }
        Err(e) => {
            eprintln!("Error getting all rates: {}", e);
        }
    }
    
    Ok(())
}