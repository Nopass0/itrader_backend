#[cfg(test)]
mod tests {
    use itrader_backend::bybit::python_rate_fetcher::{PythonRateFetcher, RateScenario};
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_python_rate_fetcher_basic() {
        dotenv().ok();
        let _ = tracing_subscriber::fmt::try_init();
        
        // Create fetcher with dummy keys (rates are public)
        let fetcher = PythonRateFetcher::new(
            "dummy_key".to_string(),
            "dummy_secret".to_string(), 
            false
        ).await.expect("Failed to create Python rate fetcher");
        
        // Test fetching a single page
        let response = fetcher.fetch_p2p_rates(1).await.expect("Failed to fetch rates");
        assert!(response.success, "Response should be successful");
        assert!(response.result.is_some(), "Result should be present");
        
        let result = response.result.unwrap();
        assert!(result.items.len() > 0, "Should have at least one item");
        
        // Verify first item has expected fields
        let first_item = &result.items[0];
        assert!(!first_item.price.is_empty(), "Price should not be empty");
        assert!(!first_item.nick_name.is_empty(), "Nickname should not be empty");
        
        println!("Successfully fetched {} P2P listings", result.items.len());
        println!("First listing: {} RUB/USDT by {}", first_item.price, first_item.nick_name);
    }
    
    #[tokio::test]
    async fn test_rate_scenarios() {
        dotenv().ok();
        
        let fetcher = PythonRateFetcher::new(
            "dummy_key".to_string(),
            "dummy_secret".to_string(),
            false
        ).await.expect("Failed to create Python rate fetcher");
        
        // Test all scenarios
        let scenarios = vec![
            (RateScenario::SmallAmountDay, "Small Amount Day"),
            (RateScenario::SmallAmountNight, "Small Amount Night"),
            (RateScenario::LargeAmountDay, "Large Amount Day"),
            (RateScenario::LargeAmountNight, "Large Amount Night"),
        ];
        
        for (scenario, name) in scenarios {
            match fetcher.get_rate_for_scenario(scenario).await {
                Ok(rate) => {
                    println!("{}: {} RUB/USDT", name, rate);
                    assert!(rate > 0.0, "Rate should be positive");
                }
                Err(e) => {
                    eprintln!("Failed to get rate for {}: {}", name, e);
                }
            }
        }
    }
    
    #[tokio::test]
    async fn test_get_current_rate() {
        dotenv().ok();
        
        let fetcher = PythonRateFetcher::new(
            "dummy_key".to_string(),
            "dummy_secret".to_string(),
            false
        ).await.expect("Failed to create Python rate fetcher");
        
        // Test with different amounts
        let test_amounts = vec![
            (30_000.0, "30k RUB"),
            (100_000.0, "100k RUB"),
        ];
        
        for (amount, description) in test_amounts {
            match fetcher.get_current_rate(amount).await {
                Ok(rate) => {
                    println!("Current rate for {}: {} RUB/USDT", description, rate);
                    assert!(rate > 0.0, "Rate should be positive");
                }
                Err(e) => {
                    eprintln!("Failed to get rate for {}: {}", description, e);
                }
            }
        }
    }
    
    #[tokio::test]
    async fn test_get_all_rates() {
        dotenv().ok();
        
        let fetcher = PythonRateFetcher::new(
            "dummy_key".to_string(),
            "dummy_secret".to_string(),
            false
        ).await.expect("Failed to create Python rate fetcher");
        
        match fetcher.get_all_rates().await {
            Ok(rates) => {
                println!("All rates:");
                for (scenario, rate) in rates.iter() {
                    println!("  {}: {} RUB/USDT", scenario, rate);
                }
                assert!(!rates.is_empty(), "Should have at least one rate");
            }
            Err(e) => {
                eprintln!("Failed to get all rates: {}", e);
            }
        }
    }
}