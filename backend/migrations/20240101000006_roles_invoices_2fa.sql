-- Add roles and permissions
CREATE TYPE user_role AS ENUM ('SUPER_ADMIN', 'ADMIN', 'MODERATOR', 'MERCHANT', 'USER');

-- Add role to merchants table
ALTER TABLE merchants ADD COLUMN role user_role NOT NULL DEFAULT 'MERCHANT';

-- Create users table for multi-user support
CREATE TABLE merchant_users (
    id SERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'USER',
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create temporary deposit addresses table (BitPay model)
CREATE TABLE deposit_addresses (
    id SERIAL PRIMARY KEY,
    payment_id VARCHAR(100) NOT NULL REFERENCES payment_transactions(payment_id),
    crypto_type VARCHAR(50) NOT NULL,
    deposit_address VARCHAR(255) NOT NULL UNIQUE,
    private_key_encrypted TEXT NOT NULL, -- Encrypted, used to forward funds
    merchant_destination VARCHAR(255) NOT NULL, -- Merchant's actual wallet
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE', -- ACTIVE, USED, EXPIRED
    expires_at TIMESTAMPTZ NOT NULL,
    forwarded_at TIMESTAMPTZ,
    forward_tx_hash VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create invoices table
CREATE TABLE invoices (
    id SERIAL PRIMARY KEY,
    invoice_id VARCHAR(100) UNIQUE NOT NULL,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    customer_email VARCHAR(255),
    customer_name VARCHAR(255),
    status VARCHAR(20) NOT NULL DEFAULT 'UNPAID', -- UNPAID, PAID, CANCELLED, OVERDUE
    items JSONB NOT NULL, -- Array of line items
    subtotal DECIMAL(20, 2) NOT NULL,
    tax DECIMAL(20, 2) NOT NULL DEFAULT 0,
    total DECIMAL(20, 2) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    payment_id VARCHAR(100) REFERENCES payment_transactions(payment_id),
    due_date DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at TIMESTAMPTZ
);

-- Create 2FA table
CREATE TABLE two_factor_auth (
    id SERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) UNIQUE,
    secret_encrypted TEXT NOT NULL,
    recovery_codes_encrypted TEXT NOT NULL, -- JSON array of codes
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    enabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_merchant_users_merchant ON merchant_users(merchant_id);
CREATE INDEX idx_merchant_users_email ON merchant_users(email);
CREATE INDEX idx_deposit_addresses_payment ON deposit_addresses(payment_id);
CREATE INDEX idx_deposit_addresses_address ON deposit_addresses(deposit_address);
CREATE INDEX idx_deposit_addresses_status ON deposit_addresses(status);
CREATE INDEX idx_invoices_merchant ON invoices(merchant_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_invoice_id ON invoices(invoice_id);
