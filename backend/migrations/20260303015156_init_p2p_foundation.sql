-- Migration: Core P2P Architecture Requirements
-- This establishes the base tables needed for the P2P Exchange, entirely isolated from the Merchant logic.

-- 1. P2P Profiles Table
-- A retail user or vendor profile. A single email can have both a merchant account (merchants table)
-- and a P2P account (p2p_profiles table).
CREATE TABLE IF NOT EXISTS p2p_profiles (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    nickname VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    kyc_level INT NOT NULL DEFAULT 0, -- 0: Unverified, 1: Basic, 2: Advanced (Vendor eligible)
    is_vendor BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    total_trades INT NOT NULL DEFAULT 0,
    completion_rate DECIMAL(5, 2) NOT NULL DEFAULT 0.00,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_p2p_profiles_email ON p2p_profiles(email);
CREATE INDEX IF NOT EXISTS idx_p2p_profiles_kyc ON p2p_profiles(kyc_level);
CREATE INDEX IF NOT EXISTS idx_p2p_profiles_sandbox ON p2p_profiles(sandbox_mode);

-- 2. P2P Wallets Table
-- Strictly custodial wallets automatically generated for P2P users upon specific asset interaction.
-- These wallets hold funds temporarily for depositing/withdrawing. They NEVER export keys.
CREATE TABLE IF NOT EXISTS p2p_wallets (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,
    network VARCHAR(50) NOT NULL,
    address VARCHAR(255) NOT NULL,
    encrypted_private_key VARCHAR(1000) NOT NULL, -- Solely for platform-managed sweeping rules
    is_active BOOLEAN NOT NULL DEFAULT true,
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, crypto_type, sandbox_mode)
);

CREATE INDEX IF NOT EXISTS idx_p2p_wallets_user ON p2p_wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_p2p_wallets_sandbox ON p2p_wallets(sandbox_mode);
CREATE INDEX IF NOT EXISTS idx_p2p_wallets_address ON p2p_wallets(address);

-- 3. P2P Balances Table (The Internal Ledger)
-- Holds actual tracking of funds allocated to a user, irrespective of the physical wallet deposits.
CREATE TABLE IF NOT EXISTS p2p_balances (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,
    available_balance DECIMAL(36, 18) NOT NULL DEFAULT 0,
    locked_balance DECIMAL(36, 18) NOT NULL DEFAULT 0, -- Funds locked in an active trade escrow
    total_balance DECIMAL(36, 18) GENERATED ALWAYS AS (available_balance + locked_balance) STORED,
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, crypto_type, sandbox_mode)
);

CREATE INDEX IF NOT EXISTS idx_p2p_balances_user ON p2p_balances(user_id);
CREATE INDEX IF NOT EXISTS idx_p2p_balances_sandbox ON p2p_balances(sandbox_mode);

-- 4. P2P Balance History
-- Immutable log of internal transfers (trades, deposits, withdrawals, fee deductions).
CREATE TABLE IF NOT EXISTS p2p_balance_history (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    balance_type VARCHAR(20) NOT NULL, -- 'AVAILABLE' or 'LOCKED'
    change_type VARCHAR(20) NOT NULL, -- 'CREDIT' or 'DEBIT'
    reason VARCHAR(100) NOT NULL, -- 'TRADE_RELEASE', 'INTERNAL_TRANSFER_FROM_MERCHANT', 'DEPOSIT', 'WITHDRAWAL', 'ESCROW_LOCK'
    reference_id VARCHAR(100), -- trade_id, withdrawal_id, deposit_tx_hash
    balance_before DECIMAL(36, 18) NOT NULL,
    balance_after DECIMAL(36, 18) NOT NULL,
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_p2p_balance_history_user ON p2p_balance_history(user_id);
CREATE INDEX IF NOT EXISTS idx_p2p_balance_history_sandbox ON p2p_balance_history(sandbox_mode);
CREATE INDEX IF NOT EXISTS idx_p2p_balance_history_reference ON p2p_balance_history(reference_id);

-- Add Comments
COMMENT ON TABLE p2p_profiles IS 'Retail and Vendor user profiles strictly for P2P trading';
COMMENT ON TABLE p2p_wallets IS 'Custodial deposit addresses for retail P2P users';
COMMENT ON TABLE p2p_balances IS 'Internal ledger separating available funds from funds locked in active trade escrow';
COMMENT ON COLUMN p2p_balances.locked_balance IS 'Funds are moved here the moment a trade matches, preventing the seller from transferring them away.';
