use itrader_backend::bybit::{BybitRateFetcher, RateScenario, P2PTradeItem};
use chrono::{FixedOffset, TimeZone};

#[tokio::test]
async fn test_bybit_rate_fetcher() {
    println!("\n=== Testing Bybit Rate Fetcher ===\n");
    
    let fetcher = BybitRateFetcher::new();
    
    // Get current Moscow time
    let moscow_offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let moscow_time = moscow_offset.from_utc_datetime(&chrono::Utc::now().naive_utc());
    println!("Current Moscow time: {}", moscow_time.format("%Y-%m-%d %H:%M:%S"));
    
    // Test different amounts - can be overridden by TEST_AMOUNT env var
    let test_amounts = if let Ok(amount_str) = std::env::var("TEST_AMOUNT") {
        if let Ok(amount) = amount_str.parse::<f64>() {
            println!("Using custom amount from TEST_AMOUNT env var: {} RUB", amount);
            vec![amount]
        } else {
            vec![30_000.0, 50_000.0, 100_000.0, 500_000.0]
        }
    } else {
        vec![30_000.0, 50_000.0, 100_000.0, 500_000.0]
    };
    
    println!("\n--- Testing Current Rates for Different Amounts ---");
    for amount in test_amounts {
        println!("\nChecking rate for {} RUB...", amount);
        let scenario = RateScenario::determine(amount, moscow_time);
        let page = scenario.get_page_number();
        println!("  Scenario: {:?}", scenario);
        println!("  Will fetch from page: {}", page);
        
        match fetcher.get_current_rate(amount).await {
            Ok(rate) => {
                println!("  ✓ Success! Rate: {} RUB/USDT", rate);
                
                // Also fetch the raw data to show more details
                match fetcher.fetch_p2p_trades(page).await {
                    Ok(response) => {
                        if let Some(result) = response.result {
                            println!("  Total items on page: {}", result.items.len());
                            if result.items.len() >= 2 {
                                let penultimate = &result.items[result.items.len() - 2];
                                println!("  Penultimate trader: {}", penultimate.nickName);
                                println!("  Min-Max: {} - {} RUB", penultimate.min_amount, penultimate.max_amount);
                                println!("  Available: {} USDT", penultimate.quantity);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  ! Error fetching details: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("  ✗ Failed to get rate: {}", e);
            }
        }
    }
    
    // Test all scenarios
    println!("\n--- Testing All Rate Scenarios ---");
    println!("Fetching rates for all time/amount combinations...\n");
    
    match fetcher.get_all_rates().await {
        Ok(rates) => {
            println!("Successfully fetched rates:");
            println!("┌─────────────────────────────────────────┬──────────────┐");
            println!("│ Scenario                                │ Rate (RUB)   │");
            println!("├─────────────────────────────────────────┼──────────────┤");
            println!("│ Small amount + Day (≤50k, 7:00-1:00)    │ {:>12.2} │", 
                rates.get("small_amount_day").unwrap_or(&0.0));
            println!("│ Small amount + Night (≤50k, 1:00-7:00)  │ {:>12.2} │", 
                rates.get("small_amount_night").unwrap_or(&0.0));
            println!("│ Large amount + Day (>50k, 7:00-1:00)    │ {:>12.2} │", 
                rates.get("large_amount_day").unwrap_or(&0.0));
            println!("│ Large amount + Night (>50k, 1:00-7:00)  │ {:>12.2} │", 
                rates.get("large_amount_night").unwrap_or(&0.0));
            println!("└─────────────────────────────────────────┴──────────────┘");
        }
        Err(e) => {
            println!("Failed to get all rates: {}", e);
        }
    }
    
    // Demonstrate scenario determination logic
    println!("\n--- Scenario Determination Logic ---");
    println!("Testing time and amount combinations:\n");
    
    let test_times = vec![
        ("00:30", 30_000.0),  // Night, small
        ("02:00", 100_000.0), // Night, large
        ("07:30", 30_000.0),  // Day, small
        ("12:00", 100_000.0), // Day, large
        ("23:59", 50_000.0),  // Day, border amount
        ("01:00", 50_000.0),  // Night start, border amount
        ("06:59", 100_000.0), // Night end, large
        ("07:00", 100_000.0), // Day start, large
    ];
    
    println!("Time (MSK) | Amount (RUB) | Scenario         | Page");
    println!("---------- | ------------ | ---------------- | ----");
    
    for (time_str, amount) in test_times {
        let test_time = moscow_offset.datetime_from_str(
            &format!("2024-01-01 {}:00", time_str), 
            "%Y-%m-%d %H:%M:%S"
        ).unwrap();
        
        let scenario = RateScenario::determine(amount, test_time);
        let page = scenario.get_page_number();
        
        println!("{:10} | {:>12.2} | {:16?} | {:>4}", 
            time_str, amount, scenario, page);
    }
    
    println!("\n=== Test completed ===");
}

#[tokio::test]
async fn test_bybit_rate_pages() {
    println!("\n=== Testing Individual Bybit Pages ===\n");
    
    let fetcher = BybitRateFetcher::new();
    
    // Test fetching from specific pages
    let test_pages = vec![2, 3, 4, 5];
    
    println!("Testing pages used by different scenarios:");
    println!("- Page 2: Small amount + Night (≤50k RUB, 1:00-7:00 MSK)");
    println!("- Page 3: Large amount + Night (>50k RUB, 1:00-7:00 MSK)");
    println!("- Page 4: Small amount + Day (≤50k RUB, 7:00-1:00 MSK)");
    println!("- Page 5: Large amount + Day (>50k RUB, 7:00-1:00 MSK)");
    
    for page in test_pages {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║ Page {} Results                          ║", page);
        println!("╚══════════════════════════════════════════╝");
        
        match fetcher.fetch_p2p_trades(page).await {
            Ok(response) => {
                println!("API Response: ret_code={}, ret_msg={}", response.ret_code, response.ret_msg);
                
                if let Some(result) = response.result {
                    println!("\n📊 Summary:");
                    println!("  Total items found: {}", result.count);
                    println!("  Items on this page: {}", result.items.len());
                    
                    if result.items.is_empty() {
                        println!("  ⚠️  No items found on this page!");
                    } else {
                        // Show price range
                        let prices: Vec<f64> = result.items.iter()
                            .filter_map(|item| item.price.parse::<f64>().ok())
                            .collect();
                        
                        if !prices.is_empty() {
                            let min_price = prices.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                            let max_price = prices.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                            println!("  Price range: {:.2} - {:.2} RUB/USDT", min_price, max_price);
                        }
                        
                        // Show penultimate price (the one we use)
                        if result.items.len() >= 2 {
                            let penultimate = &result.items[result.items.len() - 2];
                            println!("\n🎯 Penultimate item (used for rate):");
                            println!("  Price: {} RUB/USDT", penultimate.price);
                            println!("  Trader: {} (ID: {})", penultimate.nickName, penultimate.user_id);
                            println!("  Min-Max: {} - {} RUB", penultimate.min_amount, penultimate.max_amount);
                            println!("  Available: {} USDT", penultimate.quantity);
                            println!("  Orders: {} completed, {} recent", penultimate.finish_num, penultimate.recent_order_num);
                            println!("  Success rate: {:.1}%", penultimate.recent_execute_rate);
                            println!("  Online: {}", if penultimate.is_online { "Yes ✓" } else { "No ✗" });
                        }
                        
                        // Show first 3 items for comparison
                        println!("\n📋 First 3 items on page:");
                        for (i, item) in result.items.iter().take(3).enumerate() {
                            println!("\n  #{} - Price: {} RUB/USDT", i + 1, item.price);
                            println!("      Trader: {}", item.nickName);
                            println!("      Range: {} - {} RUB", item.min_amount, item.max_amount);
                            println!("      Available: {} USDT", item.quantity);
                            println!("      Success: {:.1}%", item.recent_execute_rate);
                        }
                        
                        // Show last item
                        if result.items.len() > 3 {
                            let last = &result.items[result.items.len() - 1];
                            println!("\n  Last - Price: {} RUB/USDT", last.price);
                            println!("       Trader: {}", last.nickName);
                        }
                    }
                } else {
                    println!("❌ No result data in response");
                }
            }
            Err(e) => {
                println!("❌ Error fetching page {}: {}", page, e);
                println!("   Error type: {:?}", e);
            }
        }
    }
    
    println!("\n=== Page testing completed ===");
}

#[tokio::test]
async fn test_rate_scenario_determination() {
    println!("\n=== Testing Rate Scenario Determination ===\n");
    
    let moscow_offset = FixedOffset::east_opt(3 * 3600).unwrap();
    
    // Test boundary cases
    let test_cases = vec![
        // (time, amount, expected_scenario_name, expected_page)
        ("00:59:59", 50_000.0, "SmallAmountDay", 4),     // Just before night
        ("01:00:00", 50_000.0, "SmallAmountNight", 2),   // Start of night
        ("06:59:59", 50_000.0, "SmallAmountNight", 2),   // End of night
        ("07:00:00", 50_000.0, "SmallAmountDay", 4),     // Start of day
        ("00:59:59", 50_001.0, "LargeAmountDay", 5),     // Just before night, large
        ("01:00:00", 50_001.0, "LargeAmountNight", 3),   // Start of night, large
        ("12:00:00", 49_999.0, "SmallAmountDay", 4),     // Midday, small
        ("12:00:00", 50_001.0, "LargeAmountDay", 5),     // Midday, large
    ];
    
    println!("Time (MSK) | Amount (RUB) | Scenario | Page");
    println!("---------- | ------------ | -------- | ----");
    
    for (time_str, amount, expected_name, expected_page) in test_cases {
        let test_time = moscow_offset.datetime_from_str(
            &format!("2024-01-01 {}", time_str), 
            "%Y-%m-%d %H:%M:%S"
        ).unwrap();
        
        let scenario = RateScenario::determine(amount, test_time);
        let page = scenario.get_page_number();
        
        let scenario_name = format!("{:?}", scenario);
        let matches = scenario_name.contains(expected_name) && page == expected_page;
        
        println!("{:10} | {:>12.2} | {:16} | {:>4} {}", 
            time_str, amount, scenario_name, page,
            if matches { "✓" } else { "✗" }
        );
    }
}