use crate::utils::error::{Result, AppError};
use async_imap::{Session, Client};
use async_native_tls::{TlsConnector, TlsStream};
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{info, debug, error};

pub struct ImapClient {
    server: String,
    port: u16,
    username: String,
    password: String,
    session: Option<Session<TlsStream<TcpStream>>>,
}

impl ImapClient {
    pub fn new(server: String, port: u16, username: String, password: String) -> Result<Self> {
        Ok(Self {
            server,
            port,
            username,
            password,
            session: None,
        })
    }
    
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to IMAP server {}:{}", self.server, self.port);
        
        // Connect to the server
        let tcp_stream = TcpStream::connect((self.server.as_str(), self.port)).await
            .map_err(|e| AppError::EmailError(format!("Failed to connect to IMAP server: {}", e)))?;
        
        // Setup TLS
        let tls = TlsConnector::new();
        let tls_stream = tls.connect(&self.server, tcp_stream.compat()).await
            .map_err(|e| AppError::EmailError(format!("TLS connection failed: {}", e)))?;
        
        // Create IMAP client
        let client = Client::new(tls_stream);
        
        // Login
        let session = client.login(&self.username, &self.password).await
            .map_err(|e| AppError::EmailError(format!("IMAP login failed: {}", e.0)))?;
        
        // Select INBOX
        let mut session = session.select("INBOX").await
            .map_err(|e| AppError::EmailError(format!("Failed to select INBOX: {}", e)))?;
        
        info!("Successfully connected to IMAP server");
        self.session = Some(session);
        Ok(())
    }
    
    pub async fn search(&mut self, criteria: &str) -> Result<Vec<u32>> {
        let session = self.session.as_mut()
            .ok_or_else(|| AppError::EmailError("Not connected to IMAP server".to_string()))?;
        
        debug!("Searching emails with criteria: {}", criteria);
        
        let messages = session.search(criteria).await
            .map_err(|e| AppError::EmailError(format!("Search failed: {}", e)))?;
        
        Ok(messages)
    }
    
    pub async fn fetch_message(&mut self, msg_id: u32) -> Result<Vec<u8>> {
        let session = self.session.as_mut()
            .ok_or_else(|| AppError::EmailError("Not connected to IMAP server".to_string()))?;
        
        debug!("Fetching message {}", msg_id);
        
        let messages = session.fetch(msg_id.to_string(), "RFC822").await
            .map_err(|e| AppError::EmailError(format!("Fetch failed: {}", e)))?;
        
        let mut raw_email = Vec::new();
        
        let messages: Vec<_> = messages.collect().await;
        for message in messages {
            if let Some(body) = message.body() {
                raw_email.extend_from_slice(body);
            }
        }
        
        if raw_email.is_empty() {
            return Err(AppError::EmailError(format!("No data for message {}", msg_id)));
        }
        
        Ok(raw_email)
    }
    
    pub async fn mark_as_read(&mut self, msg_id: u32) -> Result<()> {
        let session = self.session.as_mut()
            .ok_or_else(|| AppError::EmailError("Not connected to IMAP server".to_string()))?;
        
        debug!("Marking message {} as read", msg_id);
        
        session.store(msg_id.to_string(), "+FLAGS (\\Seen)").await
            .map_err(|e| AppError::EmailError(format!("Failed to mark as read: {}", e)))?;
        
        Ok(())
    }
    
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut session) = self.session.take() {
            info!("Disconnecting from IMAP server");
            session.logout().await
                .map_err(|e| AppError::EmailError(format!("Logout failed: {}", e)))?;
        }
        Ok(())
    }
}

impl Drop for ImapClient {
    fn drop(&mut self) {
        if self.session.is_some() {
            error!("ImapClient dropped without proper disconnect");
        }
    }
}