ALTER TABLE objects ADD COLUMN election_id UUID REFERENCES objects(id);
ALTER TABLE journal_entries ADD COLUMN election_id UUID REFERENCES objects(id);
