-- This file should undo anything in `up.sql`
ALTER TABLE users 
DROP COLUMN pid;

DROP INDEX IF EXISTS idx_user_pid;
DROP INDEX IF EXISTS idx_user_email;