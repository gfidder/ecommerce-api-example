-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS update_products_updated_at_trigger ON products;
DROP TABLE products;