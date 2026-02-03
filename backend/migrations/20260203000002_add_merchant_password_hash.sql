-- Add password_hash column to merchants table
ALTER TABLE merchants ADD COLUMN password_hash VARCHAR(255);

-- For existing users, we cannot recover the password.
-- They will need to reset their password or re-register.
-- We default it to NULL or a placeholder if needed, but NULL is safer.
