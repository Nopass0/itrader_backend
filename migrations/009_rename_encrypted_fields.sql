-- Rename encrypted fields to plain fields since we don't need encryption

-- Rename api_secret_encrypted to api_secret in bybit_accounts
ALTER TABLE bybit_accounts RENAME COLUMN api_secret_encrypted TO api_secret;

-- Rename password_encrypted to password in gate_accounts
ALTER TABLE gate_accounts RENAME COLUMN password_encrypted TO password;

-- Rename password_encrypted to password in gate_cookies
ALTER TABLE gate_cookies RENAME COLUMN password_encrypted TO password;