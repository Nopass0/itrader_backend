use itrader_backend::gmail::GmailClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Gmail OAuth2 Authentication Setup ===\n");
    
    let credentials_path = "gmail/credentials.json";
    let token_path = "gmail/token.json";
    
    // Check if token already exists
    if std::path::Path::new(token_path).exists() {
        println!("Existing token found at {}!", token_path);
        print!("Do you want to re-authenticate? (y/n): ");
        io::stdout().flush()?;
        
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        
        if answer.trim().to_lowercase() != "y" {
            println!("Using existing token.");
            return Ok(());
        }
    }
    
    // Create OAuth client
    let mut client = GmailClient::new_for_oauth(credentials_path).await?;
    let auth_url = client.get_authorization_url();
    
    println!("Please visit this URL to authorize your Gmail account:\n");
    println!("{}\n", auth_url);
    println!("After authorization, you will receive a code.");
    print!("Enter the authorization code: ");
    io::stdout().flush()?;
    
    let mut auth_code = String::new();
    io::stdin().read_line(&mut auth_code)?;
    let auth_code = auth_code.trim();
    
    println!("\nExchanging authorization code for tokens...");
    client.exchange_code_for_token(auth_code).await?;
    
    // Save the token
    client.save_token(token_path).await?;
    
    println!("✓ Authentication successful!");
    println!("✓ Token saved to: {}", token_path);
    println!("✓ Authenticated as: {}", client.get_user_email_address());
    
    Ok(())
}