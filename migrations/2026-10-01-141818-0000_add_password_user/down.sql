-- This file should undo anything in `up.sql`
ALTER TABLE users DROP COLUMN salt;
ALTER TABLE users DROP COLUMN password_hash;
ALTER TABLE users DROP COLUMN email;
ALTER TABLE users DROP COLUMN first_name;
ALTER TABLE users DROP COLUMN last_name;
ALTER TABLE users
ADD COLUMN hair_color VARCHAR;