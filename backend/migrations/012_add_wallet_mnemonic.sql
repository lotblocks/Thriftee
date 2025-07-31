-- Add mnemonic field to users table for HD wallet backup
ALTER TABLE users ADD COLUMN internal_wallet_mnemonic_encrypted TEXT;

-- Add index for wallet address lookups
CREATE INDEX IF NOT EXISTS idx_users_wallet_address ON users(internal_wallet_address);

-- Add comment for documentation
COMMENT ON COLUMN users.internal_wallet_mnemonic_encrypted IS 'Encrypted BIP39 mnemonic phrase for HD wallet recovery';