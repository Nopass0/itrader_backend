use itrader_backend::gate::{GateClient, TransactionService};

mod common;
use common::create_rate_limiter;

#[tokio::test]
#[ignore]
async fn test_transaction_service_cache() {
    common::setup();
    
    let rate_limiter = create_rate_limiter();
    let client = GateClient::new("https://www.gate.io".to_string(), rate_limiter).unwrap();
    
    // Note: Make sure to run gate-login test first to save cookies
    
    let service = TransactionService::new(client);
    
    // Test single transaction fetch with caching
    println!("Fetching transaction 2450530...");
    let start = std::time::Instant::now();
    let tx1 = service.get_transaction("2450530").await.unwrap();
    let first_fetch_time = start.elapsed();
    println!("First fetch took: {:?}", first_fetch_time);
    
    assert!(tx1.is_some());
    
    // Second fetch should be from cache (much faster)
    let start = std::time::Instant::now();
    let tx2 = service.get_transaction("2450530").await.unwrap();
    let cached_fetch_time = start.elapsed();
    println!("Cached fetch took: {:?}", cached_fetch_time);
    
    assert!(tx2.is_some());
    assert!(cached_fetch_time < first_fetch_time);
    
    // Test transaction status
    let status = service.get_transaction_status("2450530").await.unwrap();
    assert_eq!(status, Some(5));
    
    // Test if transaction is completed
    let is_completed = service.is_transaction_completed("2450530").await.unwrap();
    assert!(is_completed);
}

#[tokio::test]
#[ignore]
async fn test_transaction_service_batch() {
    common::setup();
    
    let rate_limiter = create_rate_limiter();
    let client = GateClient::new("https://www.gate.io".to_string(), rate_limiter).unwrap();
    
    let service = TransactionService::new(client);
    
    // Test batch fetch
    let transaction_ids = vec!["2450530", "9999999"]; // One valid, one invalid
    let results = service.get_multiple_transactions(&transaction_ids).await.unwrap();
    
    assert_eq!(results.len(), 2);
    assert!(results.get("2450530").unwrap().is_some());
    assert!(results.get("9999999").unwrap().is_none());
    
    println!("Batch results:");
    for (id, tx) in results {
        match tx {
            Some(t) => println!("  {} - Found: {:?}", id, t.amount),
            None => println!("  {} - Not found", id),
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_transaction_service_cache_clear() {
    let rate_limiter = create_rate_limiter();
    let client = GateClient::new("https://www.gate.io".to_string(), rate_limiter).unwrap();
    
    let service = TransactionService::new(client);
    
    // Fetch to populate cache
    service.get_transaction("2450530").await.ok();
    
    // Clear cache
    service.clear_cache().await;
    
    // Remove specific item (should not error even if cache is empty)
    service.remove_from_cache("2450530").await;
}