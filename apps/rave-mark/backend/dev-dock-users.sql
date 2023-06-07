insert or ignore into elections (
    id,
    election_data,
    jurisdiction
) values (
    'election-famous-names-2021',
    readfile('../../../libs/fixtures/data/electionFamousNames2021/election.json'),
    'st.dev-jurisdiction'
);

insert or ignore into elections (
    id,
    election_data,
    jurisdiction
) values (
    'election-minimal-exhaustive',
    readfile('../../../libs/fixtures/data/electionMinimalExhaustiveSample/electionMinimalExhaustiveSample.json'),
    'st.dev-jurisdiction'
);

--
--
-- `voter-unregistered` is a voter who has not registered to vote

insert or ignore into voters (
    id,
    common_access_card_id,
    is_admin
) values (
    'voter-unregistered',
    'voter-unregistered',
    0
);

-- no voter registration


--
--
-- `voter-registration-pending` is a voter whose registration is pending

insert or ignore into voters (
    id,
    common_access_card_id,
    is_admin
) values (
    'voter-registration-pending',
    'voter-registration-pending',
    0
);

insert or ignore into voter_registrations (
    id,
    voter_id,
    given_name,
    family_name,
    election_id,
    voted_at
) values (
    'voter-registration-pending',
    'voter-registration-pending',
    'Nathan',
    'Shelley',
    null,
    null
);

--
--
-- `voter-registered` is a voter who has registered

insert or ignore into voters (
    id,
    common_access_card_id,
    is_admin
) values (
    'voter-registered',
    'voter-registered',
    0
);

insert or ignore into voter_registrations (
    id,
    voter_id,
    given_name,
    family_name,
    election_id,
    voted_at
) values (
    'voter-registered',
    'voter-registered',
    'Nathan',
    'Shelley',
    'election-famous-names-2021',
    null
);

--
--
-- `voter-voted` is a voter who has registered to vote and has voted

insert or ignore into voters (
    id,
    common_access_card_id,
    is_admin
) values (
    'voter-voted',
    'voter-voted',
    0
);

insert or ignore into voter_registrations (
    id,
    voter_id,
    given_name,
    family_name,
    election_id,
    voted_at
) values (
    'voter-voted',
    'voter-voted',
    'Nathan',
    'Shelley',
    null,
    '2020-11-03 00:00:00'
);


--
--
-- `admin-unregistered` is an admin voter who has not registered to vote

insert or ignore into voters (
    id,
    common_access_card_id,
    is_admin
) values (
    'admin-unregistered',
    'admin-unregistered',
    1
);

-- no voter registration


--
--
-- `admin-registration-pending` is an admin voter whose registration is pending

insert or ignore into voters (
    id,
    common_access_card_id,
    is_admin
) values (
    'admin-registration-pending',
    'admin-registration-pending',
    1
);

insert or ignore into voter_registrations (
    id,
    voter_id,
    given_name,
    family_name,
    election_id,
    voted_at
) values (
    'admin-registration-pending',
    'admin-registration-pending',
    'Rebecca',
    'Welton',
    null,
    null
);

--
--
-- `admin-registered` is an admin voter who has registered to vote but has not voted

insert or ignore into voters (
    id,
    common_access_card_id,
    is_admin
) values (
    'admin-registered',
    'admin-registered',
    1
);

insert or ignore into voter_registrations (
    id,
    voter_id,
    given_name,
    family_name,
    election_id,
    voted_at
) values (
    'admin-registered',
    'admin-registered',
    'Rebecca',
    'Welton',
    'election-minimal-exhaustive',
    null
);

--
--
-- `admin-voted` is an admin voter who has voted

insert or ignore into voters (
    id,
    common_access_card_id,
    is_admin
) values (
    'admin-voted',
    'admin-voted',
    1
);

insert or ignore into voter_registrations (
    id,
    voter_id,
    given_name,
    family_name,
    election_id,
    voted_at
) values (
    'admin-voted',
    'admin-voted',
    'Rebecca',
    'Welton',
    'election-minimal-exhaustive',
    '2020-11-03 00:00:00'
);
