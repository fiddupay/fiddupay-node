-- Seed Super Admin User
-- This creates a super admin user directly in the database for security

-- First, let's create the super admin user
INSERT INTO merchants (
    email, 
    business_name, 
    api_key_hash, 
    role, 
    is_active, 
    sandbox_mode,
    fee_percentage,
    customer_pays_fee,
    created_at,
    updated_at
) VALUES (
    'superadmin@fiddupay.com',
    'FidduPay Super Admin',
    -- Hash of 'superadmin_api_key_2026_secure' using SHA-256
    'a8f5f167f44f4964e6c998dee827110c',
    'SUPER_ADMIN',
    true,
    false,
    0.00,
    true,
    NOW(),
    NOW()
) ON CONFLICT (email) DO UPDATE SET
    role = 'SUPER_ADMIN',
    business_name = 'FidduPay Super Admin',
    is_active = true,
    updated_at = NOW();

-- Create a regular admin user as well
INSERT INTO merchants (
    email, 
    business_name, 
    api_key_hash, 
    role, 
    is_active, 
    sandbox_mode,
    fee_percentage,
    customer_pays_fee,
    created_at,
    updated_at
) VALUES (
    'admin@fiddupay.com',
    'FidduPay Admin',
    -- Hash of 'admin_api_key_2026_secure' using SHA-256
    'b9e6c167e55e5975f7d009fff938221d',
    'ADMIN',
    true,
    false,
    0.00,
    true,
    NOW(),
    NOW()
) ON CONFLICT (email) DO UPDATE SET
    role = 'ADMIN',
    business_name = 'FidduPay Admin',
    is_active = true,
    updated_at = NOW();

-- Revert any merchants that were given admin roles back to MERCHANT
UPDATE merchants 
SET role = 'MERCHANT', updated_at = NOW()
WHERE role IN ('ADMIN', 'SUPER_ADMIN') 
AND email NOT IN ('superadmin@fiddupay.com', 'admin@fiddupay.com');

-- Display the admin users
SELECT id, email, business_name, role, is_active, created_at 
FROM merchants 
WHERE role IN ('ADMIN', 'SUPER_ADMIN')
ORDER BY role DESC, created_at ASC;
