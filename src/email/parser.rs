use crate::utils::error::{Result, AppError};
use crate::email::monitor::{ReceiptEmail, Attachment};
use mail_parser::{MessageParser, Message, MimeHeaders};
use tracing::{debug, warn};

pub struct EmailParser;

impl EmailParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse_email(&self, raw_email: &[u8]) -> Result<ReceiptEmail> {
        let message = mail_parser::MessageParser::default()
            .parse(raw_email)
            .ok_or_else(|| AppError::EmailError("Failed to parse email".to_string()))?;
        
        // Extract basic info
        let id = message.message_id()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("no-id-{}", chrono::Utc::now().timestamp()));
        
        let from = message.from()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.address())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let subject = message.subject()
            .unwrap_or("No Subject")
            .to_string();
        
        let body = self.extract_body(&message);
        let attachments = self.extract_attachments(&message);
        
        let received_at = message.date()
            .and_then(|d| chrono::DateTime::from_timestamp(d.to_timestamp(), 0))
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);
        
        Ok(ReceiptEmail {
            id,
            from,
            subject,
            body,
            attachments,
            received_at,
        })
    }
    
    fn extract_body(&self, message: &Message) -> String {
        // Try to get text body first
        if let Some(body) = message.body_text(0) {
            return body.to_string();
        }
        
        // Fall back to HTML body
        if let Some(html) = message.body_html(0) {
            // Simple HTML to text conversion
            return self.html_to_text(&html);
        }
        
        // If no body found
        String::new()
    }
    
    fn extract_attachments(&self, message: &Message) -> Vec<Attachment> {
        let mut attachments = Vec::new();
        
        for attachment in message.attachments() {
            if let Some(filename) = attachment.attachment_name() {
                debug!("Found attachment: {}", filename);
                
                let content_type = attachment.content_type()
                    .map(|ct| ct.ctype())
                    .unwrap_or("application/octet-stream")
                    .to_string();
                
                let data = attachment.contents().to_vec();
                
                attachments.push(Attachment {
                    filename: filename.to_string(),
                    content_type,
                    data,
                });
            }
        }
        
        attachments
    }
    
    fn html_to_text(&self, html: &str) -> String {
        // Simple HTML stripping - in production, use a proper HTML parser
        html.chars()
            .fold((String::new(), false), |(mut acc, in_tag), ch| {
                match ch {
                    '<' => (acc, true),
                    '>' => (acc, false),
                    _ if !in_tag => {
                        acc.push(ch);
                        (acc, false)
                    }
                    _ => (acc, in_tag),
                }
            })
            .0
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Legacy function for compatibility
pub fn parse_email_body(body: &str) -> Option<Vec<u8>> {
    warn!("Using deprecated parse_email_body function");
    None
}