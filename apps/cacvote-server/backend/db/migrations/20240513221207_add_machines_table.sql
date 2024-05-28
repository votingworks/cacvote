CREATE TABLE machines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    machine_identifier VARCHAR(255) NOT NULL,
    certificates BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);
