use crate::utils::error::{AppError, Result};
use super::models::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc, Duration};
use reqwest::{Client, Url};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tracing::{info, debug, error};

const GMAIL_API_BASE: &str = "https://www.googleapis.com/gmail/v1";
const OAUTH2_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const OAUTH2_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const SCOPES: &str = "https://www.googleapis.com/auth/gmail.readonly";

pub struct GmailClient {
    client: Client,
    pub credentials: GmailCredentials,
    user_email: String,
}

impl GmailClient {
    pub async fn new_for_oauth(credentials_path: &str) -> Result<Self> {
        let credentials_json = fs::read_to_string(credentials_path)
            .map_err(|e| AppError::FileSystem(format!("Failed to read credentials: {}", e)))?;
        
        let creds_value: Value = serde_json::from_str(&credentials_json)
            .map_err(|e| AppError::Serialization(format!("Failed to parse credentials: {}", e)))?;
        
        let installed = creds_value.get("installed")
            .ok_or_else(|| AppError::Configuration("Missing 'installed' section in credentials".to_string()))?;
        
        let client_id = installed.get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Configuration("Missing client_id".to_string()))?
            .to_string();
        
        let client_secret = installed.get("client_secret")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Configuration("Missing client_secret".to_string()))?
            .to_string();
        
        let credentials = GmailCredentials {
            client_id,
            client_secret,
            refresh_token: None,
            access_token: None,
            token_expiry: None,
        };
        
        let client = Client::new();
        Ok(Self {
            client,
            credentials,
            user_email: String::new(),
        })
    }
    
    pub async fn new(credentials_path: &str, token_path: Option<&str>) -> Result<Self> {
        let credentials_json = fs::read_to_string(credentials_path)
            .map_err(|e| AppError::FileSystem(format!("Failed to read credentials: {}", e)))?;
        
        let creds_value: Value = serde_json::from_str(&credentials_json)
            .map_err(|e| AppError::Serialization(format!("Failed to parse credentials: {}", e)))?;
        
        let installed = creds_value.get("installed")
            .ok_or_else(|| AppError::Configuration("Missing 'installed' section in credentials".to_string()))?;
        
        let client_id = installed.get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Configuration("Missing client_id".to_string()))?
            .to_string();
        
        let client_secret = installed.get("client_secret")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Configuration("Missing client_secret".to_string()))?
            .to_string();
        
        let mut credentials = GmailCredentials {
            client_id,
            client_secret,
            refresh_token: None,
            access_token: None,
            token_expiry: None,
        };
        
        // Load token if available
        if let Some(token_path) = token_path {
            if Path::new(token_path).exists() {
                let token_json = fs::read_to_string(token_path)
                    .map_err(|e| AppError::FileSystem(format!("Failed to read token: {}", e)))?;
                
                let token_value: Value = serde_json::from_str(&token_json)
                    .map_err(|e| AppError::Serialization(format!("Failed to parse token: {}", e)))?;
                
                if let Some(refresh_token) = token_value.get("refresh_token").and_then(|v| v.as_str()) {
                    credentials.refresh_token = Some(refresh_token.to_string());
                }
                if let Some(access_token) = token_value.get("access_token").and_then(|v| v.as_str()) {
                    credentials.access_token = Some(access_token.to_string());
                }
                if let Some(expiry) = token_value.get("token_expiry").and_then(|v| v.as_str()) {
                    credentials.token_expiry = DateTime::parse_from_rfc3339(expiry)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc));
                }
            }
        }
        
        let client = Client::new();
        let mut gmail_client = Self {
            client,
            credentials,
            user_email: String::new(),
        };
        
        // Get user profile to set email
        gmail_client.user_email = gmail_client.get_user_email().await?;
        
        Ok(gmail_client)
    }
    
    pub fn get_authorization_url(&self) -> String {
        let params = [
            ("client_id", &self.credentials.client_id),
            ("redirect_uri", &"urn:ietf:wg:oauth:2.0:oob".to_string()),
            ("response_type", &"code".to_string()),
            ("scope", &SCOPES.to_string()),
            ("access_type", &"offline".to_string()),
            ("prompt", &"consent".to_string()),
        ];
        
        let mut url = Url::parse(OAUTH2_AUTH_URL).unwrap();
        url.query_pairs_mut().extend_pairs(&params);
        url.to_string()
    }
    
    pub async fn exchange_code_for_token(&mut self, auth_code: &str) -> Result<()> {
        let params = json!({
            "code": auth_code,
            "client_id": self.credentials.client_id,
            "client_secret": self.credentials.client_secret,
            "redirect_uri": "urn:ietf:wg:oauth:2.0:oob",
            "grant_type": "authorization_code"
        });
        
        let response = self.client
            .post(OAUTH2_TOKEN_URL)
            .json(&params)
            .send()
            .await
            .map_err(|e| AppError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Authentication(format!("Failed to get token: {}", error_text)));
        }
        
        let token: OAuth2Token = response.json().await
            .map_err(|e| AppError::Serialization(format!("Failed to parse token response: {}", e)))?;
        
        self.credentials.access_token = Some(token.access_token);
        self.credentials.refresh_token = token.refresh_token.or(self.credentials.refresh_token.clone());
        self.credentials.token_expiry = Some(Utc::now() + Duration::seconds(token.expires_in));
        
        // Get user email after successful authorization
        self.user_email = self.get_user_email().await?;
        
        info!("Successfully exchanged authorization code for tokens");
        Ok(())
    }
    
    pub async fn refresh_access_token(&mut self) -> Result<()> {
        let refresh_token = self.credentials.refresh_token.as_ref()
            .ok_or_else(|| AppError::Authentication("No refresh token available".to_string()))?;
        
        let params = json!({
            "refresh_token": refresh_token,
            "client_id": self.credentials.client_id,
            "client_secret": self.credentials.client_secret,
            "grant_type": "refresh_token"
        });
        
        let response = self.client
            .post(OAUTH2_TOKEN_URL)
            .json(&params)
            .send()
            .await
            .map_err(|e| AppError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Authentication(format!("Failed to refresh token: {}", error_text)));
        }
        
        let token: OAuth2Token = response.json().await
            .map_err(|e| AppError::Serialization(format!("Failed to parse token response: {}", e)))?;
        
        self.credentials.access_token = Some(token.access_token);
        self.credentials.token_expiry = Some(Utc::now() + Duration::seconds(token.expires_in));
        
        debug!("Successfully refreshed access token");
        Ok(())
    }
    
    async fn ensure_valid_token(&mut self) -> Result<()> {
        if let Some(expiry) = self.credentials.token_expiry {
            if expiry > Utc::now() + Duration::minutes(5) {
                return Ok(());
            }
        }
        
        self.refresh_access_token().await
    }
    
    async fn get_user_email(&mut self) -> Result<String> {
        self.ensure_valid_token().await?;
        
        let access_token = self.credentials.access_token.as_ref()
            .ok_or_else(|| AppError::Authentication("No access token available".to_string()))?;
        
        let url = format!("{}/users/me/profile", GMAIL_API_BASE);
        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Gmail(format!("Failed to get user profile: {}", error_text)));
        }
        
        let profile: Value = response.json().await
            .map_err(|e| AppError::Serialization(format!("Failed to parse profile response: {}", e)))?;
        
        let email = profile.get("emailAddress")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Gmail("Email address not found in profile".to_string()))?;
        
        Ok(email.to_string())
    }
    
    pub fn get_user_email_address(&self) -> &str {
        &self.user_email
    }
    
    pub async fn list_messages(&mut self, filter: &EmailFilter) -> Result<Vec<GmailMessageId>> {
        self.ensure_valid_token().await?;
        
        let access_token = self.credentials.access_token.as_ref()
            .ok_or_else(|| AppError::Authentication("No access token available".to_string()))?;
        
        let mut query_parts = Vec::new();
        
        if let Some(from) = &filter.from {
            query_parts.push(format!("from:{}", from));
        }
        
        if let Some(subject) = &filter.subject {
            query_parts.push(format!("subject:{}", subject));
        }
        
        if let Some(after) = filter.after {
            query_parts.push(format!("after:{}", after.timestamp()));
        }
        
        if let Some(before) = filter.before {
            query_parts.push(format!("before:{}", before.timestamp()));
        }
        
        if let Some(true) = filter.has_attachment {
            query_parts.push("has:attachment".to_string());
        }
        
        let query = query_parts.join(" ");
        let url = format!("{}/users/me/messages", GMAIL_API_BASE);
        
        let mut all_messages = Vec::new();
        let mut page_token: Option<String> = None;
        
        loop {
            let mut params = vec![("q", query.as_str())];
            if let Some(token) = &page_token {
                params.push(("pageToken", token.as_str()));
            }
            
            let response = self.client
                .get(&url)
                .bearer_auth(access_token)
                .query(&params)
                .send()
                .await
                .map_err(|e| AppError::Network(e))?;
            
            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(AppError::Gmail(format!("Failed to list messages: {}", error_text)));
            }
            
            let message_list: GmailMessageList = response.json().await
                .map_err(|e| AppError::Serialization(format!("Failed to parse message list: {}", e)))?;
            
            if let Some(messages) = message_list.messages {
                all_messages.extend(messages);
            }
            
            page_token = message_list.next_page_token;
            if page_token.is_none() {
                break;
            }
        }
        
        Ok(all_messages)
    }
    
    pub async fn get_message(&mut self, message_id: &str) -> Result<EmailMessage> {
        self.ensure_valid_token().await?;
        
        let access_token = self.credentials.access_token.as_ref()
            .ok_or_else(|| AppError::Authentication("No access token available".to_string()))?;
        
        let url = format!("{}/users/me/messages/{}", GMAIL_API_BASE, message_id);
        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Gmail(format!("Failed to get message: {}", error_text)));
        }
        
        let gmail_message: GmailMessage = response.json().await
            .map_err(|e| AppError::Serialization(format!("Failed to parse message: {}", e)))?;
        
        let headers = &gmail_message.payload.headers;
        let subject = headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("subject"))
            .map(|h| h.value.clone());
        
        let from = headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("from"))
            .map(|h| h.value.clone())
            .unwrap_or_default();
        
        let to = headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("to"))
            .map(|h| h.value.clone())
            .unwrap_or_default();
        
        let date_str = headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case("date"))
            .map(|h| &h.value)
            .map_or("", |v| v);
        
        let date = DateTime::parse_from_rfc2822(date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let mut attachments = Vec::new();
        self.extract_attachments(&gmail_message.payload, &mut attachments);
        
        Ok(EmailMessage {
            id: gmail_message.id,
            thread_id: gmail_message.thread_id,
            subject,
            from,
            to,
            date,
            snippet: gmail_message.snippet,
            attachments,
        })
    }
    
    fn extract_attachments(&self, payload: &MessagePayload, attachments: &mut Vec<Attachment>) {
        if let Some(parts) = &payload.parts {
            for part in parts {
                if let Some(filename) = &part.filename {
                    if !filename.is_empty() {
                        if let Some(attachment_id) = &part.body.attachment_id {
                            attachments.push(Attachment {
                                id: attachment_id.clone(),
                                filename: filename.clone(),
                                mime_type: part.mime_type.clone(),
                                size: part.body.size,
                                data: None,
                            });
                        }
                    }
                }
                
                // Recursively check nested parts
                if let Some(nested_parts) = &part.parts {
                    let nested_payload = MessagePayload {
                        headers: vec![],
                        parts: Some(nested_parts.to_vec()),
                        body: None,
                    };
                    self.extract_attachments(&nested_payload, attachments);
                }
            }
        }
    }
    
    pub async fn get_attachment(&mut self, message_id: &str, attachment_id: &str) -> Result<Vec<u8>> {
        self.ensure_valid_token().await?;
        
        let access_token = self.credentials.access_token.as_ref()
            .ok_or_else(|| AppError::Authentication("No access token available".to_string()))?;
        
        let url = format!("{}/users/me/messages/{}/attachments/{}", 
                         GMAIL_API_BASE, message_id, attachment_id);
        
        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Gmail(format!("Failed to get attachment: {}", error_text)));
        }
        
        let attachment_data: AttachmentData = response.json().await
            .map_err(|e| AppError::Serialization(format!("Failed to parse attachment data: {}", e)))?;
        
        // Gmail might return base64 with different padding, try both variants
        let decoded = URL_SAFE_NO_PAD.decode(&attachment_data.data)
            .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(&attachment_data.data))
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(&attachment_data.data))
            .map_err(|e| AppError::Serialization(format!("Failed to decode attachment: {}", e)))?;
        
        Ok(decoded)
    }
    
    pub async fn download_pdf_attachments(&mut self, message: &EmailMessage, save_path: &Path) -> Result<Vec<String>> {
        let mut saved_files = Vec::new();
        
        for attachment in &message.attachments {
            if attachment.mime_type == "application/pdf" {
                info!("Downloading PDF attachment: {}", attachment.filename);
                
                let data = self.get_attachment(&message.id, &attachment.id).await?;
                
                let file_path = save_path.join(&attachment.filename);
                fs::write(&file_path, &data)
                    .map_err(|e| AppError::FileSystem(format!("Failed to save attachment: {}", e)))?;
                
                saved_files.push(file_path.to_string_lossy().to_string());
                info!("Saved PDF to: {}", file_path.display());
            }
        }
        
        Ok(saved_files)
    }
    
    pub async fn save_token(&self, token_path: &str) -> Result<()> {
        let token_data = json!({
            "refresh_token": self.credentials.refresh_token,
            "access_token": self.credentials.access_token,
            "token_expiry": self.credentials.token_expiry.map(|dt| dt.to_rfc3339()),
        });
        
        let token_json = serde_json::to_string_pretty(&token_data)
            .map_err(|e| AppError::Serialization(format!("Failed to serialize token: {}", e)))?;
        
        fs::write(token_path, token_json)
            .map_err(|e| AppError::FileSystem(format!("Failed to save token: {}", e)))?;
        
        Ok(())
    }
}