CREATE TABLE objects (
    id UUID PRIMARY KEY,

    -- which jurisdiction owns the object. de-normalized out of certificate,
    -- e.g. "ca.alameda"
    jurisdiction varchar(255) NOT NULL,

    -- what type of object is this. de-normalized out of `payload`,
    -- e.g. "election"
    object_type varchar(255) NOT NULL,

    -- raw object data, must be JSON with fields `object_type` and `data`
    payload BYTEA NOT NULL,

    -- certificate used to sign `payload` to get `signature`
    certificate BYTEA NOT NULL,

    -- signature of `data` using `certificate`
    signature BYTEA NOT NULL,

    -- server sync timestamp, NULL if not synced
    server_synced_at timestamptz,

    -- when the object was created
    created_at timestamptz NOT NULL DEFAULT current_timestamp,

    -- when the object was deleted, NULL if not deleted
    deleted_at timestamptz
);

CREATE TABLE journal_entries (
    id UUID PRIMARY KEY,

    -- the object that was created or updated
    object_id UUID NOT NULL REFERENCES objects(id),

    -- which jurisdiction owns the object, must match `jurisdiction` in `objects`
    jurisdiction varchar(255) NOT NULL,

    -- the object's type, must match `object_type` in `objects`
    object_type varchar(255) NOT NULL,

    -- action that was taken on the object, e.g. "create" or "update"
    action varchar(255) NOT NULL,

    -- when the action was taken
    created_at timestamptz NOT NULL DEFAULT current_timestamp
);
