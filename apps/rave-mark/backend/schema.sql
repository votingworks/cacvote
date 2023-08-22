create table system_settings (
  -- enforce singleton table
  id integer primary key check (id = 1),
  are_poll_worker_card_pins_enabled boolean not null,
  inactive_session_time_limit_minutes integer not null,
  num_incorrect_pin_attempts_allowed_before_card_lockout integer not null,
  overall_session_time_limit_hours integer not null,
  starting_card_lockout_duration_seconds integer not null
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
  definition bytea not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);

create table admins (
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
  -- CAC ID of the person for this record
  common_access_card_id uuid not null unique,
  given_name text not null,
  family_name text not null,
  address_line_1 text not null,
  address_line_2 text,
  city text not null,
  state text not null,
  postal_code text not null,
  -- the state-issued id number of the person making the request,
  -- e.g. a driver's license number
  state_id text not null,
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
  -- CAC ID of the person for this record
  common_access_card_id uuid not null unique,
  registration_request_id uuid not null references registration_requests(id),
  election_id uuid not null references elections(id),
  precinct_id text not null,
  ballot_style_id text not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);

create table ballots_pending_print (
  id uuid primary key,
  -- CAC ID of the person for this record
  common_access_card_id uuid not null unique,
  common_access_card_certificate bytea not null,
  registration_id uuid not null references registrations(id),
  cast_vote_record bytea not null,
  cast_vote_record_signature bytea not null,
  created_at timestamptz not null default current_timestamp,

  unique (registration_id)
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

create table scanned_ballots (
  id uuid primary key,
  -- generated on the server, present only if the record has been synced
  server_id uuid unique,
  -- generated on a client machine
  client_id uuid not null,
  -- ID of the machine this record was originally created on
  machine_id varchar(255) not null,
  election_id uuid not null references elections(id) on update cascade on delete cascade,
  cast_vote_record bytea not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);