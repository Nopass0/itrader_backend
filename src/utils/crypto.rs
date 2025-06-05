use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{PasswordHash, SaltString};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use sha2::Digest;

use crate::utils::error::{AppError, Result};

const NONCE_SIZE: usize = 12;

pub fn encrypt(data: &str, key: &str) -> Result<String> {
    // Derive a 32-byte key from the provided key using SHA256
    let key_bytes = sha2::Sha256::digest(key.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    
    let cipher = Aes256Gcm::new(key);
    
    // Generate a random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt the data
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| AppError::Encryption(format!("Encryption failed: {}", e)))?;
    
    // Combine nonce + ciphertext
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    
    // Encode as base64
    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn decrypt(encrypted_data: &str, key: &str) -> Result<String> {
    // Decode from base64
    let combined = general_purpose::STANDARD
        .decode(encrypted_data)
        .map_err(|e| AppError::Encryption(format!("Invalid base64: {}", e)))?;
    
    if combined.len() < NONCE_SIZE {
        return Err(AppError::Encryption("Invalid encrypted data".to_string()));
    }
    
    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    // Derive key
    let key_bytes = sha2::Sha256::digest(key.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    
    let cipher = Aes256Gcm::new(key);
    
    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Encryption(format!("Decryption failed: {}", e)))?;
    
    String::from_utf8(plaintext)
        .map_err(|e| AppError::Encryption(format!("Invalid UTF-8: {}", e)))
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Encryption(format!("Password hashing failed: {}", e)))?
        .to_string();
    
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Encryption(format!("Invalid password hash: {}", e)))?;
    
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

// Aliases for compatibility
pub fn encrypt_string(data: &str, key: &str) -> Result<String> {
    encrypt(data, key)
}

pub fn decrypt_string(data: &str, key: &str) -> Result<String> {
    decrypt(data, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = "sensitive data";
        let key = "test-encryption-key";
        
        let encrypted = encrypt(data, key).unwrap();
        assert_ne!(encrypted, data);
        
        let decrypted = decrypt(&encrypted, key).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_wrong_key_fails() {
        let data = "sensitive data";
        let key = "test-encryption-key";
        let wrong_key = "wrong-key";
        
        let encrypted = encrypt(data, key).unwrap();
        assert!(decrypt(&encrypted, wrong_key).is_err());
    }

    #[test]
    fn test_password_hashing() {
        let password = "secure_password123";
        
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}