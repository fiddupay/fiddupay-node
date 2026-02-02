-- Add partial_payment_received to address_only_status enum
ALTER TYPE address_only_status ADD VALUE 'partial_payment_received' AFTER 'payment_received';
