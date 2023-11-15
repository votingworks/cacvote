ALTER TABLE registration_requests ADD COLUMN address_line_1 varchar(255) not null;
ALTER TABLE registration_requests ADD COLUMN address_line_2 varchar(255);
ALTER TABLE registration_requests ADD COLUMN city varchar(255) not null;
ALTER TABLE registration_requests ADD COLUMN state varchar(16) not null;
ALTER TABLE registration_requests ADD COLUMN postal_code varchar(255) not null;
ALTER TABLE registration_requests ADD COLUMN state_id varchar(255) not null;