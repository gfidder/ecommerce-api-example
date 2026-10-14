-- Your SQL goes here
ALTER TABLE users 
ADD COLUMN pid UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4();

CREATE INDEX idx_user_email ON users(email);
CREATE INDEX idx_user_pid ON users(pid);