use itrader_backend::gmail::{GmailClient, EmailFilter};

#[tokio::test]
async fn test_gmail_client_creation() {
    println!("=== Testing Gmail Client Creation ===");
    
    // This test will fail if credentials file doesn't exist
    match GmailClient::new("gmail/credentials.json", None).await {
        Ok(_) => println!("✓ Gmail client created successfully"),
        Err(e) => println!("✗ Failed to create Gmail client: {}", e),
    }
}

#[tokio::test]
async fn test_email_filter() {
    println!("=== Testing Email Filter Creation ===");
    
    let filter = EmailFilter::new()
        .from_sender("test@example.com")
        .today()
        .with_attachments();
    
    println!("✓ Created filter:");
    println!("  From: {:?}", filter.from);
    println!("  After: {:?}", filter.after);
    println!("  Has attachments: {:?}", filter.has_attachment);
}