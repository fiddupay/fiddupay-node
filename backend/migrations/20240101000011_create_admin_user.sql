-- Create admin user for testing
-- This creates a default admin merchant for testing purposes

-- Insert admin merchant with ADMIN role
INSERT INTO merchants (
    email, 
    business_name, 
    api_key_hash, 
    fee_percentage, 
    customer_pays_fee, 
    is_active, 
    sandbox_mode, 
    role,
    created_at, 
    updated_at
) VALUES (
    'admin@fiddupay.com',
    'FidduPay Admin',
    -- SHA256 hash of 'sk_admin_test_key_12345'
    'a8b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5',
    2.5,
    true,
    true,
    true,
    'ADMIN',
    NOW(),
    NOW()
) ON CONFLICT (email) DO NOTHING;
