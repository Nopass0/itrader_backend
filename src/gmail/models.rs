use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: String,
    pub thread_id: String,
    pub subject: Option<String>,
    pub from: String,
    pub to: String,
    pub date: DateTime<Utc>,
    pub snippet: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailFilter {
    pub from: Option<String>,
    pub subject: Option<String>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub has_attachment: Option<bool>,
}

impl EmailFilter {
    pub fn new() -> Self {
        Self {
            from: None,
            subject: None,
            after: None,
            before: None,
            has_attachment: None,
        }
    }

    pub fn from_sender(mut self, sender: &str) -> Self {
        self.from = Some(sender.to_string());
        self
    }

    pub fn today(mut self) -> Self {
        let today = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        self.after = Some(DateTime::from_naive_utc_and_offset(today, Utc));
        self
    }

    pub fn with_attachments(mut self) -> Self {
        self.has_attachment = Some(true);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub token_expiry: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GmailMessageList {
    pub messages: Option<Vec<GmailMessageId>>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GmailMessageId {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GmailMessage {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
    pub payload: MessagePayload,
    pub snippet: String,
}

#[derive(Debug, Deserialize)]
pub struct MessagePayload {
    pub headers: Vec<MessageHeader>,
    pub parts: Option<Vec<MessagePart>>,
    pub body: Option<MessageBody>,
}

#[derive(Debug, Deserialize)]
pub struct MessageHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagePart {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub filename: Option<String>,
    pub body: MessageBody,
    pub parts: Option<Vec<MessagePart>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageBody {
    #[serde(rename = "attachmentId")]
    pub attachment_id: Option<String>,
    pub size: i64,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AttachmentData {
    pub size: i64,
    pub data: String,
}