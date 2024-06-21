DROP TABLE IF EXISTS tags;

ALTER TABLE passwords DROP CONSTRAINT IF EXISTS passwords_salt_check;
ALTER TABLE passwords ADD COLUMN tags text[] NOT NULL DEFAULT ARRAY[]::text[];
