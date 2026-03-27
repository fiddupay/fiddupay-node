-- Update Daily Volume Limit and Withdrawal Auto-Approval Limit
-- Requirement: Increase limits to $100,000 as per merchant request
UPDATE system_settings 
SET value = '100000.00', updated_at = NOW()
WHERE key = 'DAILY_VOLUME_LIMIT_NON_KYC_USD';

UPDATE system_settings 
SET value = '100000.00', updated_at = NOW()
WHERE key = 'WITHDRAWAL_AUTO_APPROVAL_LIMIT_USD';
