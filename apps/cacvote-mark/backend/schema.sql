create table system_settings (
  -- enforce singleton table
  id integer primary key check (id = 1),
  data text not null -- JSON blob
);

create table server_sync_attempts (
  id uuid primary key,
  creator text not null,
  trigger text not null,
  status_message text not null,
  success boolean,
  created_at timestamptz not null default current_timestamp,
  completed_at timestamp
);

create table jurisdictions (
  id uuid primary key,
  name text not null,
  created_at timestamptz not null default current_timestamp
);

create table elections (
  -- generated on this machine
  id uuid primary key,
  -- generated on the server, present only if the record has been synced
  server_id uuid,
  -- generated on a client machine; should match `id` if this record was
  -- generated on this machine
  client_id uuid not null,
  -- ID of the machine this record was originally created on
  machine_id text not null,
  jurisdiction_id uuid not null references jurisdictions(id),
  definition bytea not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);

create table admins (
  machine_id varchar(255) not null,
  -- CAC ID of the admin user
  common_access_card_id uuid not null unique,
  created_at timestamptz not null default current_timestamp
);

create table registration_requests (
  -- generated on this machine
  id uuid primary key,
  -- generated on the server, present only if the record has been synced
  server_id uuid,
  -- generated on a client machine; should match `id` if this record was
  -- generated on this machine
  client_id uuid not null unique,
  -- ID of the machine this record was originally created on
  machine_id text not null,
  jurisdiction_id uuid not null references jurisdictions(id),
  -- CAC ID of the person for this record
  common_access_card_id uuid not null unique,
  given_name text not null,
  family_name text not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);

create table registrations (
  -- generated on this machine
  id uuid primary key,
  -- generated on the server, present only if the record has been synced
  server_id uuid,
  -- generated on a client machine; should match `id` if this record was
  -- generated on this machine
  client_id uuid not null,
  -- ID of the machine this record was originally created on
  machine_id text not null,
  jurisdiction_id uuid not null references jurisdictions(id),
  -- CAC ID of the person for this record
  common_access_card_id uuid not null unique,
  registration_request_id uuid not null references registration_requests(id),
  election_id uuid not null references elections(id),
  precinct_id text not null,
  ballot_style_id text not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);

create table printed_ballots (
  -- generated on this machine
  id uuid primary key,
  -- generated on the server, present only if the record has been synced
  server_id uuid,
  -- generated on a client machine; should match `id` if this record was
  -- generated on this machine
  client_id uuid not null,
  -- ID of the machine this record was originally created on
  machine_id text not null,
  -- CAC ID of the person for this record
  common_access_card_id uuid not null unique,
  common_access_card_certificate bytea not null,
  registration_id uuid not null references registrations(id),
  cast_vote_record bytea not null,
  cast_vote_record_signature bytea not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);
