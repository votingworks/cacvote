create table elections (
  id uuid primary key,
  election_data json not null,
  created_at timestamp not null default current_timestamp
);

create table system_settings (
  -- enforce singleton table
  id integer primary key check (id = 1),
  are_poll_worker_card_pins_enabled boolean not null,
  inactive_session_time_limit_minutes integer not null,
  num_incorrect_pin_attempts_allowed_before_card_lockout integer not null,
  overall_session_time_limit_hours integer not null,
  starting_card_lockout_duration_seconds integer not null
);

create table voters (
  id uuid primary key,
  common_access_card_id uuid not null unique,
  is_admin boolean not null default false,
  created_at timestamp not null default current_timestamp
);

create table voter_registration_requests (
  id uuid primary key,
  voter_id uuid not null references voters(id),
  given_name text not null,
  family_name text not null,
  address_line_1 text not null,
  address_line_2 text,
  city text not null,
  state text not null,
  postal_code text not null,
  state_id text not null,
  created_at timestamp not null default current_timestamp
);

create table voter_election_registrations (
  id uuid primary key,
  voter_id uuid not null references voters(id),
  voter_registration_request_id uuid not null references voter_registration_requests(id),
  election_id uuid not null references elections(id),
  precinct_id text not null,
  ballot_style_id text not null,
  created_at timestamp not null default current_timestamp
);

create table voter_election_selections (
  id uuid primary key,
  voter_id uuid not null references voters(id),
  voter_election_registration_id uuid not null references voter_election_registrations(id),
  cast_vote_record json not null,
  created_at timestamp not null default current_timestamp
);
