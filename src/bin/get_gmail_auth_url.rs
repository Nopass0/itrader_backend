use itrader_backend::gmail::GmailClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Gmail Authorization URL Generator ===\n");
    
    // Try to create a client for OAuth
    let mut client = GmailClient::new_for_oauth("gmail/credentials.json").await?;
    let auth_url = client.get_authorization_url();
    
    println!("Please visit this URL to authorize your Gmail account:\n");
    println!("{}\n", auth_url);
    println!("After authorization, you'll receive a code.");
    println!("You can then run one of these commands:");
    println!("1. Use the interactive test: cargo test test_gmail_auth -- --ignored --nocapture");
    println!("2. Or save the code and run the setup_gmail_token tool\n");
    
    Ok(())
}