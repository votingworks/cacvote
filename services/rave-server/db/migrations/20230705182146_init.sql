create table elections (
  id uuid primary key,
  -- generated on a client machine
  client_id uuid not null,
  -- ID of the machine this record was originally created on
  machine_id varchar(255) not null,
  election json not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);

create table admins (
  -- CAC ID of the admin user
  common_access_card_id varchar(36) not null primary key,
  created_at timestamptz not null default current_timestamp
);

create table registration_requests (
  id uuid primary key,
  -- generated on a client machine
  client_id uuid not null,
  -- ID of the machine this record was originally created on
  machine_id varchar(255) not null,
  -- CAC ID of this record's voter
  common_access_card_id varchar(36) not null,
  given_name varchar(255) not null,
  family_name varchar(255) not null,
  address_line_1 varchar(255) not null,
  address_line_2 varchar(255),
  city varchar(255) not null,
  state varchar(16) not null,
  postal_code varchar(255) not null,
  state_id varchar(255) not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);

create table registrations (
  id uuid primary key,
  -- generated on a client machine
  client_id uuid not null,
  -- ID of the machine this record was originally created on
  machine_id varchar(255) not null,
  -- CAC ID of this record's voter
  common_access_card_id varchar(36) not null,
  registration_request_id uuid not null references registration_requests(id) on update cascade on delete cascade,
  election_id uuid not null references elections(id) on update cascade on delete cascade,
  precinct_id varchar(255) not null,
  ballot_style_id varchar(255) not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);

create table ballots (
  id uuid primary key,
  -- generated on a client machine
  client_id uuid not null,
  -- ID of the machine this record was originally created on
  machine_id varchar(255) not null,
  -- CAC ID of this record's voter
  common_access_card_id varchar(36) not null,
  registration_id uuid not null references registrations(id) on update cascade on delete cascade,
  cast_vote_record json not null,
  created_at timestamptz not null default current_timestamp,

  unique (client_id, machine_id)
);