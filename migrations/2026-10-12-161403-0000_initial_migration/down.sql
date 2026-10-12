-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS update_user_updated_at_trigger ON users;
DROP TRIGGER IF EXISTS update_session_updated_at_trigger ON "session";
DROP FUNCTION IF EXISTS update_timestamp;
DROP INDEX IF EXISTS session_token_key;
DROP TABLE "session";
DROP TABLE users;
DROP EXTENSION IF EXISTS "uuid-ossp";