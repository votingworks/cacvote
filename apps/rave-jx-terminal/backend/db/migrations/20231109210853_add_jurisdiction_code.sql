ALTER TABLE jurisdictions ADD COLUMN code VARCHAR(255) NOT NULL;
ALTER TABLE jurisdictions ADD CONSTRAINT jurisdictions_code_unique UNIQUE (code);
