use itrader_backend::bybit::BybitRateFetcher;
use chrono::{FixedOffset, TimeZone};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n=== Simple Bybit Rate Test ===\n");
    
    let fetcher = BybitRateFetcher::new();
    
    // Get current Moscow time
    let moscow_offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let moscow_time = moscow_offset.from_utc_datetime(&chrono::Utc::now().naive_utc());
    println!("Current Moscow time: {}", moscow_time.format("%Y-%m-%d %H:%M:%S"));
    
    // Test a single amount
    let amount = 50_000.0;
    println!("\nTesting rate for {} RUB...", amount);
    
    match fetcher.get_current_rate(amount).await {
        Ok(rate) => {
            println!("✓ Success! Current rate: {} RUB/USDT", rate);
            
            // Test fetching raw data from page 4 (day small amount)
            println!("\nFetching raw data from page 4...");
            match fetcher.fetch_p2p_trades(4).await {
                Ok(response) => {
                    println!("API Response: ret_code={}, ret_msg={}", response.ret_code, response.ret_msg);
                    if let Some(result) = response.result {
                        println!("Total items: {}", result.count);
                        println!("Items on page: {}", result.items.len());
                        
                        if result.items.len() >= 2 {
                            let penultimate = &result.items[result.items.len() - 2];
                            println!("\nPenultimate item (used for rate):");
                            println!("  Price: {} RUB/USDT", penultimate.price);
                            println!("  Trader: {}", penultimate.nickName);
                            println!("  Range: {} - {} RUB", penultimate.min_amount, penultimate.max_amount);
                        }
                    }
                }
                Err(e) => println!("✗ Error fetching page: {}", e),
            }
        }
        Err(e) => {
            println!("✗ Failed to get rate: {}", e);
            println!("Error details: {:?}", e);
        }
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}