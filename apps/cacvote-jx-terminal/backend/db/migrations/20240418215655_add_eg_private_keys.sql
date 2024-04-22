CREATE TABLE eg_private_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    election_object_id UUID NOT NULL,
    private_key BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (election_object_id) REFERENCES objects(id)
);
