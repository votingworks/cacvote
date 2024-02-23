create table jurisdictions (
    id uuid primary key,
    name varchar(255) not null,
    created_at timestamptz not null default current_timestamp
);

alter table elections
add column jurisdiction_id uuid not null
references jurisdictions(id)
on update cascade
on delete cascade;

alter table registration_requests
add column jurisdiction_id uuid not null
references jurisdictions(id)
on update cascade
on delete cascade;

alter table registrations
add column jurisdiction_id uuid not null
references jurisdictions(id)
on update cascade
on delete cascade;
