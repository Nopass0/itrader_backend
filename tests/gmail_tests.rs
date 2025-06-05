use itrader_backend::gmail::{GmailClient, EmailFilter};
use itrader_backend::utils::error::Result;
use std::path::Path;
use chrono::Utc;

const GMAIL_CREDENTIALS_PATH: &str = "gmail/credentials.json";
const GMAIL_TOKEN_PATH: &str = "gmail/token.json";

#[tokio::test]
#[ignore] // Run manually with: cargo test test_gmail_auth -- --ignored --nocapture
async fn test_gmail_auth() -> Result<()> {
    println!("=== Testing Gmail OAuth2 Authentication ===");
    
    // Try to create client with existing token
    match GmailClient::new(GMAIL_CREDENTIALS_PATH, Some(GMAIL_TOKEN_PATH)).await {
        Ok(client) => {
            println!("✓ Gmail authentication successful with existing token!");
            println!("✓ Authenticated as: {}", client.get_user_email_address());
        }
        Err(_) => {
            // No token exists, need to do OAuth flow
            println!("No existing token found. Starting OAuth2 authorization flow...");
            
            let mut client = GmailClient::new_for_oauth(GMAIL_CREDENTIALS_PATH).await?;
            let auth_url = client.get_authorization_url();
            
            println!("\n=================================================================");
            println!("Please visit this URL to authorize the application:");
            println!("{}", auth_url);
            println!("=================================================================");
            println!("\nAfter authorization, you will receive a code.");
            print!("Please enter the authorization code: ");
            
            use std::io::{self, Write};
            io::stdout().flush()
                .map_err(|e| itrader_backend::utils::error::AppError::FileSystem(e.to_string()))?;
            
            let mut auth_code = String::new();
            io::stdin().read_line(&mut auth_code)
                .map_err(|e| itrader_backend::utils::error::AppError::FileSystem(e.to_string()))?;
            let auth_code = auth_code.trim();
            
            println!("Exchanging authorization code for tokens...");
            client.exchange_code_for_token(auth_code).await?;
            
            // Ensure gmail directory exists
            std::fs::create_dir_all("gmail")
                .map_err(|e| itrader_backend::utils::error::AppError::FileSystem(e.to_string()))?;
            
            // Save the token for future use
            client.save_token(GMAIL_TOKEN_PATH).await?;
            println!("✓ Token saved to: {}", GMAIL_TOKEN_PATH);
            
            // Get user email
            println!("✓ Gmail authentication successful!");
            println!("✓ Authenticated as: {}", client.get_user_email_address());
        }
    }
    
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_gmail_list_emails_today() -> Result<()> {
    println!("=== Testing Gmail List Emails from Today ===");
    
    let mut client = GmailClient::new(GMAIL_CREDENTIALS_PATH, Some(GMAIL_TOKEN_PATH)).await?;
    
    // Create filter for today's emails
    let filter = EmailFilter::new().today();
    
    let messages = client.list_messages(&filter).await?;
    println!("✓ Found {} emails from today", messages.len());
    
    // Show first 5 emails
    for (i, msg_id) in messages.iter().take(5).enumerate() {
        let message = client.get_message(&msg_id.id).await?;
        println!("\n{}. Email ID: {}", i + 1, message.id);
        println!("   From: {}", message.from);
        println!("   Subject: {}", message.subject.unwrap_or_else(|| "(no subject)".to_string()));
        println!("   Date: {}", message.date.format("%Y-%m-%d %H:%M:%S"));
        println!("   Attachments: {}", message.attachments.len());
    }
    
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_gmail_list_emails_from_sender() -> Result<()> {
    println!("=== Testing Gmail List Emails from Specific Sender ===");
    println!("Enter sender email address to filter:");
    
    let mut sender = String::new();
    std::io::stdin().read_line(&mut sender)
        .map_err(|e| itrader_backend::utils::error::AppError::FileSystem(e.to_string()))?;
    let sender = sender.trim();
    
    let mut client = GmailClient::new(GMAIL_CREDENTIALS_PATH, Some(GMAIL_TOKEN_PATH)).await?;
    
    // Create filter for specific sender
    let filter = EmailFilter::new().from_sender(sender);
    
    let messages = client.list_messages(&filter).await?;
    println!("✓ Found {} emails from {}", messages.len(), sender);
    
    // Show first 5 emails
    for (i, msg_id) in messages.iter().take(5).enumerate() {
        let message = client.get_message(&msg_id.id).await?;
        println!("\n{}. Email ID: {}", i + 1, message.id);
        println!("   Subject: {}", message.subject.unwrap_or_else(|| "(no subject)".to_string()));
        println!("   Date: {}", message.date.format("%Y-%m-%d %H:%M:%S"));
        println!("   Snippet: {}", &message.snippet[..message.snippet.len().min(100)]);
    }
    
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_gmail_get_latest_email() -> Result<()> {
    println!("=== Testing Gmail Get Latest Email ===");
    
    let mut client = GmailClient::new(GMAIL_CREDENTIALS_PATH, Some(GMAIL_TOKEN_PATH)).await?;
    
    // Get emails from today, or last 7 days if no emails today
    let filter = EmailFilter::new();
    
    let messages = client.list_messages(&filter).await?;
    if messages.is_empty() {
        println!("No emails found");
        return Ok(());
    }
    
    // Get the first (latest) email
    let latest_msg_id = &messages[0];
    let message = client.get_message(&latest_msg_id.id).await?;
    
    println!("✓ Latest Email Details:");
    println!("  ID: {}", message.id);
    println!("  From: {}", message.from);
    println!("  To: {}", message.to);
    println!("  Subject: {}", message.subject.unwrap_or_else(|| "(no subject)".to_string()));
    println!("  Date: {}", message.date.format("%Y-%m-%d %H:%M:%S"));
    println!("  Attachments: {}", message.attachments.len());
    
    if !message.attachments.is_empty() {
        println!("\n  Attachment Details:");
        for (i, att) in message.attachments.iter().enumerate() {
            println!("    {}. {} ({}, {} bytes)", i + 1, att.filename, att.mime_type, att.size);
        }
    }
    
    println!("\n  Snippet:");
    println!("  {}", message.snippet);
    
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_gmail_get_latest_pdf() -> Result<()> {
    use itrader_backend::ocr::pdf::PdfReceiptParser;
    
    println!("=== Testing Gmail Get Latest Email with PDF ===");
    
    let mut client = GmailClient::new(GMAIL_CREDENTIALS_PATH, Some(GMAIL_TOKEN_PATH)).await?;
    
    // Create filter for emails with attachments
    let filter = EmailFilter::new().with_attachments();
    
    let messages = client.list_messages(&filter).await?;
    println!("Found {} emails with attachments", messages.len());
    
    // Find first email with PDF attachment
    let mut pdf_message = None;
    for msg_id in messages.iter().take(20) {  // Check first 20 emails
        let message = client.get_message(&msg_id.id).await?;
        
        if message.attachments.iter().any(|att| att.mime_type == "application/pdf") {
            pdf_message = Some(message);
            break;
        }
    }
    
    if let Some(message) = pdf_message {
        println!("\n✓ Found Email with PDF:");
        println!("  From: {}", message.from);
        println!("  Subject: {}", message.subject.clone().unwrap_or_else(|| "(no subject)".to_string()));
        println!("  Date: {}", message.date.format("%Y-%m-%d %H:%M:%S"));
        
        // Create test_data directory if it doesn't exist
        std::fs::create_dir_all("test_data")
            .map_err(|e| itrader_backend::utils::error::AppError::FileSystem(e.to_string()))?;
        
        // Download PDF attachments
        let saved_files = client.download_pdf_attachments(&message, Path::new("test_data")).await?;
        
        println!("\n✓ Downloaded {} PDF files:", saved_files.len());
        
        // Process each PDF with OCR
        let parser = PdfReceiptParser::new();
        for file_path in &saved_files {
            println!("\n  Processing: {}", file_path);
            
            match parser.parse_receipt(file_path).await {
                Ok(receipt_info) => {
                    println!("\n  ✓ Receipt Data Extracted:");
                    println!("    Amount: {} RUB", receipt_info.amount);
                    println!("    Date: {}", receipt_info.date_time.format("%Y-%m-%d %H:%M:%S"));
                    println!("    Transaction ID: {}", receipt_info.transaction_id.unwrap_or_else(|| "N/A".to_string()));
                    println!("    Bank: {}", receipt_info.bank_name.unwrap_or_else(|| "N/A".to_string()));
                    println!("    Recipient: {}", receipt_info.recipient.unwrap_or_else(|| "N/A".to_string()));
                    println!("    Sender: {}", receipt_info.sender.unwrap_or_else(|| "N/A".to_string()));
                    println!("    Status: {}", receipt_info.status.unwrap_or_else(|| "N/A".to_string()));
                    
                    if let Some(card) = receipt_info.card_number {
                        println!("    Card: {}", card);
                    }
                    if let Some(phone) = receipt_info.phone_number {
                        println!("    Phone: {}", phone);
                    }
                }
                Err(e) => {
                    println!("  ✗ Failed to parse receipt: {}", e);
                }
            }
        }
    } else {
        println!("No emails with PDF attachments found in recent messages");
    }
    
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_gmail_download_pdf_from_sender() -> Result<()> {
    println!("=== Testing Gmail Download PDF from Specific Sender ===");
    println!("Enter sender email address:");
    
    let mut sender = String::new();
    std::io::stdin().read_line(&mut sender)
        .map_err(|e| itrader_backend::utils::error::AppError::FileSystem(e.to_string()))?;
    let sender = sender.trim();
    
    let mut client = GmailClient::new(GMAIL_CREDENTIALS_PATH, Some(GMAIL_TOKEN_PATH)).await?;
    
    // Create filter for specific sender with attachments
    let filter = EmailFilter::new()
        .from_sender(sender)
        .with_attachments();
    
    let messages = client.list_messages(&filter).await?;
    println!("Found {} emails with attachments from {}", messages.len(), sender);
    
    // Find emails with PDF attachments
    let mut pdf_count = 0;
    for msg_id in messages.iter().take(10) {  // Check first 10 emails
        let message = client.get_message(&msg_id.id).await?;
        
        let pdf_attachments: Vec<_> = message.attachments.iter()
            .filter(|att| att.mime_type == "application/pdf")
            .collect();
        
        if !pdf_attachments.is_empty() {
            println!("\n  Email: {}", message.subject.clone().unwrap_or_else(|| "(no subject)".to_string()));
            println!("  Date: {}", message.date.format("%Y-%m-%d %H:%M:%S"));
            
            // Create test_data directory if it doesn't exist
            std::fs::create_dir_all("test_data")
                .map_err(|e| itrader_backend::utils::error::AppError::FileSystem(e.to_string()))?;
            
            // Download PDF attachments
            let saved_files = client.download_pdf_attachments(&message, Path::new("test_data")).await?;
            
            for file in &saved_files {
                println!("  ✓ Downloaded: {}", file);
                pdf_count += 1;
            }
        }
    }
    
    println!("\n✓ Total PDFs downloaded: {}", pdf_count);
    
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_gmail_get_sender_info() -> Result<()> {
    println!("=== Testing Gmail Get Sender Information ===");
    
    let mut client = GmailClient::new(GMAIL_CREDENTIALS_PATH, Some(GMAIL_TOKEN_PATH)).await?;
    
    println!("✓ Authenticated Account Email: {}", client.get_user_email_address());
    
    // Get recent emails to show different senders
    let filter = EmailFilter::new();
    let messages = client.list_messages(&filter).await?;
    
    let mut unique_senders = std::collections::HashSet::new();
    
    println!("\n✓ Recent Email Senders:");
    for msg_id in messages.iter().take(20) {
        let message = client.get_message(&msg_id.id).await?;
        
        // Extract email address from "From" field
        let from_email = if let Some(start) = message.from.find('<') {
            if let Some(end) = message.from.find('>') {
                message.from[start + 1..end].to_string()
            } else {
                message.from.clone()
            }
        } else {
            message.from.clone()
        };
        
        if unique_senders.insert(from_email.clone()) {
            println!("  - {} ({})", 
                from_email,
                message.subject.unwrap_or_else(|| "(no subject)".to_string())
            );
            
            if unique_senders.len() >= 10 {
                break;
            }
        }
    }
    
    Ok(())
}