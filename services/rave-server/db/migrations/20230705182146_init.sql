create table elections (
  id uuid primary key,
  election json not null,
  created_at timestamp not null default current_timestamp
);

create table voters (
  id uuid primary key,
  common_access_card_id varchar(36) not null unique,
  is_admin boolean not null default false,
  created_at timestamp not null default current_timestamp
);

create table voter_registration_requests (
  id uuid primary key,
  voter_id uuid not null references voters(id) on update cascade on delete cascade,
  given_name varchar(255) not null,
  family_name varchar(255) not null,
  address_line_1 varchar(255) not null,
  address_line_2 varchar(255),
  city varchar(255) not null,
  state varchar(16) not null,
  postal_code varchar(255) not null,
  state_id varchar(255) not null,
  created_at timestamp not null default current_timestamp
);

create table voter_election_registrations (
  id uuid primary key,
  voter_id uuid not null references voters(id) on update cascade on delete cascade,
  voter_registration_request_id uuid not null references voter_registration_requests(id) on update cascade on delete cascade,
  election_id uuid not null references elections(id) on update cascade on delete cascade,
  precinct_id varchar(255) not null,
  ballot_style_id varchar(255) not null,
  created_at timestamp not null default current_timestamp
);

create table voter_election_selections (
  id uuid primary key,
  voter_id uuid not null references voters(id) on update cascade on delete cascade,
  voter_election_registration_id uuid not null references voter_election_registrations(id) on update cascade on delete cascade,
  cast_vote_record json not null,
  created_at timestamp not null default current_timestamp
);