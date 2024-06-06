create table system_settings (
  -- enforce singleton table
  id integer primary key check (id = 1),
  data text not null -- JSON blob
);

create table objects (
  id uuid primary key,

  -- the ID of the election object this object is associated with or NULL
  election_id uuid,

  -- which jurisdiction owns the object. de-normalized out of certificates,
  -- e.g. "ca.alameda"
  jurisdiction varchar(255) not null,

  -- what type of object is this. de-normalized out of `payload`,
  -- e.g. "Election"
  object_type varchar(255) not null,

  -- raw object data, must be JSON with fields `object_type` and `data`
  payload bytea not null,

  -- certificates used to sign `payload` to get `signature`
  certificates bytea not null,

  -- signature of `data` using `certificates`
  signature bytea not null,

  -- server sync timestamp, NULL if not synced
  server_synced_at timestamptz,

  -- when the object was created
  created_at timestamptz not null default current_timestamp,

  -- when the object was deleted, NULL if not deleted
  deleted_at timestamptz
);

create table journal_entries (
  id uuid primary key,

  -- the object that was created or updated
  object_id uuid not null,

  -- the ID of the election object this object is associated with or NULL
  election_id uuid,

  -- which jurisdiction owns the object, must match `jurisdiction` in `objects`
  jurisdiction varchar(255) not null,

  -- the object's type, must match `object_type` in `objects`
  object_type varchar(255) not null,

  -- action that was taken on the object, e.g. "create" or "update"
  action varchar(255) not null,

  -- when the action was taken
  created_at timestamptz not null default current_timestamp
);

create table mail_label_print_jobs (
  id uuid primary key,

  -- the ID of the cast ballot this label is associated with
  cast_ballot_object_id uuid not null,

  -- the ID of the election object this object is associated with
  election_id uuid not null,

  -- when the action was taken
  created_at timestamptz not null default current_timestamp
);
