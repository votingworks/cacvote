ALTER TABLE jurisdictions DROP CONSTRAINT IF EXISTS jurisdictions_code_unique;
ALTER TABLE jurisdictions DROP COLUMN IF EXISTS code;
