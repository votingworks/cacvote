CREATE TABLE scanned_mailing_label_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- denormalized out of `original_payload` ▶︎ `election_object_id` as-is
    election_id UUID NOT NULL REFERENCES objects(id),

    -- denormalized out of `original_payload` ▶︎ `machine_id` via `machine_identifier`
    machine_id UUID NOT NULL REFERENCES machines(id),

    -- denormalized out of `original_payload` ▶︎ `common_access_card_id` as-is
    common_access_card_id VARCHAR(255) NOT NULL,

    -- denormalized out of `original_payload` ▶︎ `encrypted_ballot_signature_hash` as-is
    encrypted_ballot_signature_hash BYTEA NOT NULL,

    -- original payload data from the scanned mailing label
    original_payload BYTEA NOT NULL,

    -- when this record was created
    created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);