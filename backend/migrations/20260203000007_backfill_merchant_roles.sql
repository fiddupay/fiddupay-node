-- Backfill NULL roles to 'MERCHANT' to ensure existing users can login
UPDATE merchants SET role = 'MERCHANT' WHERE role IS NULL;
