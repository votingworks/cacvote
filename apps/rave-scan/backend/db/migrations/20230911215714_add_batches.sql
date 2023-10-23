create table batches (
  id uuid primary key,
  scanned_ballot_ids uuid[] not null,
  error_message text,
  started_at timestamptz not null default current_timestamp,
  ended_at timestamptz
);
