use crate::core::config::EmailConfig;
use crate::utils::error::{Result, AppError};
use crate::email::imap_client::ImapClient;
use crate::email::parser::EmailParser;
use std::future::Future;
use std::collections::HashSet;
use tokio::time::{interval, Duration};
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptEmail {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub received_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

pub struct EmailMonitor {
    config: EmailConfig,
    imap_client: ImapClient,
    parser: EmailParser,
    processed_ids: HashSet<String>,
}

impl EmailMonitor {
    pub fn new(config: EmailConfig) -> Result<Self> {
        let imap_client = ImapClient::new(
            config.imap_server.clone(),
            config.imap_port,
            config.email.clone(),
            config.password.clone(),
        )?;
        
        let parser = EmailParser::new();
        
        Ok(Self {
            config,
            imap_client,
            parser,
            processed_ids: HashSet::new(),
        })
    }

    pub async fn monitor_receipts<F, Fut>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(ReceiptEmail) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send,
    {
        info!("Starting email monitoring for receipts");
        
        // Connect to IMAP server
        self.imap_client.connect().await?;
        
        let mut check_interval = interval(Duration::from_secs(30)); // Check every 30 seconds
        
        loop {
            check_interval.tick().await;
            
            match self.check_for_new_receipts().await {
                Ok(receipts) => {
                    for receipt in receipts {
                        if !self.processed_ids.contains(&receipt.id) {
                            info!("Processing new receipt email: {}", receipt.subject);
                            
                            match callback(receipt.clone()).await {
                                Ok(_) => {
                                    self.processed_ids.insert(receipt.id.clone());
                                    info!("Successfully processed receipt: {}", receipt.id);
                                }
                                Err(e) => {
                                    error!("Failed to process receipt {}: {}", receipt.id, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to check for new receipts: {}", e);
                    // Try to reconnect
                    if let Err(reconnect_err) = self.imap_client.connect().await {
                        error!("Failed to reconnect to IMAP server: {}", reconnect_err);
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    }
                }
            }
        }
    }
    
    async fn check_for_new_receipts(&mut self) -> Result<Vec<ReceiptEmail>> {
        // Search for unread emails from known receipt senders
        let search_criteria = self.build_search_criteria();
        let messages = self.imap_client.search(&search_criteria).await?;
        
        let mut receipts = Vec::new();
        
        for msg_id in messages {
            match self.imap_client.fetch_message(msg_id).await {
                Ok(raw_email) => {
                    match self.parser.parse_email(&raw_email) {
                        Ok(email) => {
                            if self.is_receipt_email(&email) {
                                receipts.push(email);
                                // Mark as read
                                if let Err(e) = self.imap_client.mark_as_read(msg_id).await {
                                    warn!("Failed to mark email {} as read: {}", msg_id, e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse email {}: {}", msg_id, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch email {}: {}", msg_id, e);
                }
            }
        }
        
        Ok(receipts)
    }
    
    fn build_search_criteria(&self) -> String {
        // Search for unread emails from known bank/payment provider domains
        let domains = vec![
            "tbank.ru",
            "tinkoff.ru",
            "sberbank.ru",
            "alfa-bank.ru",
            "raiffeisen.ru",
            "qiwi.ru",
            "yoomoney.ru",
        ];
        
        let mut criteria = "UNSEEN ".to_string();
        for domain in domains {
            criteria.push_str(&format!("OR FROM \"@{}\" ", domain));
        }
        
        // Also search for keywords in subject
        criteria.push_str("OR SUBJECT \"чек\" OR SUBJECT \"квитанция\" OR SUBJECT \"receipt\" OR SUBJECT \"payment\"");
        
        criteria
    }
    
    fn is_receipt_email(&self, email: &ReceiptEmail) -> bool {
        // Check if email contains receipt indicators
        let subject_lower = email.subject.to_lowercase();
        let body_lower = email.body.to_lowercase();
        
        // Check for receipt keywords
        let receipt_keywords = vec![
            "чек", "квитанция", "receipt", "payment", "оплата",
            "перевод", "transaction", "confirmation"
        ];
        
        let has_receipt_keyword = receipt_keywords.iter().any(|kw| {
            subject_lower.contains(kw) || body_lower.contains(kw)
        });
        
        // Check for PDF attachments
        let has_pdf = email.attachments.iter().any(|att| {
            att.filename.to_lowercase().ends_with(".pdf") ||
            att.content_type.contains("pdf")
        });
        
        has_receipt_keyword && (has_pdf || body_lower.contains("руб") || body_lower.contains("rub"))
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        self.imap_client.disconnect().await
    }
}