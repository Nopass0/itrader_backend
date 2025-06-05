use anyhow::Result;
use itrader_backend::bybit::rate_fetcher::{BybitRateFetcher, RateScenario};
use chrono::{FixedOffset, TimeZone};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env::set_var("RUST_LOG", "info");
    let _ = tracing_subscriber::fmt::try_init();
    
    let args: Vec<String> = env::args().collect();
    
    println!("=== Bybit P2P Rate Checker ===");
    println!();
    
    let fetcher = BybitRateFetcher::new();
    
    // Get current Moscow time
    let moscow_offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let moscow_time = moscow_offset.from_utc_datetime(&chrono::Utc::now().naive_utc());
    println!("Current Moscow time: {}", moscow_time.format("%Y-%m-%d %H:%M:%S"));
    
    if args.len() > 1 {
        // Check rate for specific amount
        let amount = args[1].parse::<f64>().unwrap_or(50_000.0);
        println!("Checking rate for amount: {} RUB", amount);
        
        let scenario = RateScenario::determine(amount, moscow_time);
        println!("\nScenario determined: {:?}", scenario);
        println!("Will fetch from page: {}", scenario.get_page_number());
        
        match fetcher.get_current_rate(amount).await {
            Ok(rate) => {
                println!("\n✅ Successfully fetched rate!");
                println!("📈 Current rate: {} RUB/USDT", rate);
                
                // Fetch additional details
                println!("\n🔍 Fetching additional details...");
                match fetcher.fetch_p2p_trades(scenario.get_page_number()).await {
                    Ok(response) => {
                        if let Some(result) = response.result {
                            println!("  Total items on page: {}", result.items.len());
                            
                            if result.items.len() >= 2 {
                                let penultimate = &result.items[result.items.len() - 2];
                                println!("\n📊 Rate source details:");
                                println!("  Trader: {}", penultimate.nickName);
                                println!("  Price: {} RUB/USDT", penultimate.price);
                                println!("  Min-Max: {} - {} RUB", penultimate.min_amount, penultimate.max_amount);
                                println!("  Available: {} USDT", penultimate.quantity);
                                println!("  Success rate: {:.1}%", penultimate.recent_execute_rate);
                                println!("  Online: {}", if penultimate.is_online { "Yes ✓" } else { "No ✗" });
                                
                                // Show price range on page
                                let prices: Vec<f64> = result.items.iter()
                                    .filter_map(|item| item.price.parse::<f64>().ok())
                                    .collect();
                                if !prices.is_empty() {
                                    let min_price = prices.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                                    let max_price = prices.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                                    println!("\n💹 Price range on page {}: {:.2} - {:.2} RUB/USDT", 
                                        scenario.get_page_number(), min_price, max_price);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("  ⚠️ Could not fetch additional details: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("\n❌ Error getting rate: {}", e);
                eprintln!("   Error type: {:?}", e);
                
                // Try to fetch raw data to debug
                println!("\n🔧 Debugging: Attempting raw API call to page {}...", scenario.get_page_number());
                match fetcher.fetch_p2p_trades(scenario.get_page_number()).await {
                    Ok(response) => {
                        println!("  ✓ Raw API call succeeded!");
                        println!("  Response code: {}", response.ret_code);
                        println!("  Response message: {}", response.ret_msg);
                        if let Some(result) = response.result {
                            println!("  Items found: {}", result.items.len());
                            if result.items.is_empty() {
                                println!("  ⚠️ No items returned - page might be empty");
                            }
                        } else {
                            println!("  ⚠️ No result data in response");
                        }
                    }
                    Err(fetch_err) => {
                        println!("  ✗ Raw API call also failed: {}", fetch_err);
                        println!("  This might be a network connectivity issue");
                    }
                }
            }
        }
    } else {
        // Show all rates
        println!("Fetching all rate scenarios...\n");
        
        match fetcher.get_all_rates().await {
            Ok(rates) => {
                println!("Current P2P rates (USDT/RUB):");
                println!("================================");
                println!("Small amount + Day   (≤50k, 07:00-01:00): {} RUB/USDT", 
                    rates.get("small_amount_day").unwrap_or(&0.0));
                println!("Small amount + Night (≤50k, 01:00-07:00): {} RUB/USDT", 
                    rates.get("small_amount_night").unwrap_or(&0.0));
                println!("Large amount + Day   (>50k, 07:00-01:00): {} RUB/USDT", 
                    rates.get("large_amount_day").unwrap_or(&0.0));
                println!("Large amount + Night (>50k, 01:00-07:00): {} RUB/USDT", 
                    rates.get("large_amount_night").unwrap_or(&0.0));
                
                // Show which rate applies now
                println!("\nCurrent scenario based on time:");
                let current_small = RateScenario::determine(30_000.0, moscow_time);
                let current_large = RateScenario::determine(100_000.0, moscow_time);
                println!("- For amounts ≤50k: {:?}", current_small);
                println!("- For amounts >50k: {:?}", current_large);
            }
            Err(e) => {
                eprintln!("Error getting rates: {}", e);
            }
        }
    }
    
    println!("\nUsage: {} [amount]", args[0]);
    println!("Example: {} 75000", args[0]);
    
    Ok(())
}