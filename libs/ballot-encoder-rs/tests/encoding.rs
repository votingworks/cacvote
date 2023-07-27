use ballot_encoder_rs::{decode, encode, EncodableCvr, TestMode};
use bitstream_io::{BigEndian, BitWrite, BitWriter};
use fixtures::encoded::{
    FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_VOTES_IN_ALL_CONTESTS,
    FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_WRITE_INS,
};
use types_rs::{
    cdf::cvr::{IndicationStatus, VxBallotType},
    election::{BallotStyleId, PrecinctId},
};

use crate::{
    fixtures::{
        encoded::FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_NO_VOTES, read_famous_names_election_definition,
    },
    util::{assert_bytes_equal, assert_cvrs_equivalent, build_cvr, sizeof},
};

mod fixtures;
mod util;

#[test]
fn test_compare_to_manually_encoded_empty_votes() {
    let election_definition = read_famous_names_election_definition();
    let election = &election_definition.election;
    let election_hash = &election_definition.election_hash;
    let ballot_style_id = election.ballot_styles[0].id.clone();
    let contests = election.get_contests(ballot_style_id).unwrap();
    let mut data = vec![];
    let mut writer = BitWriter::endian(&mut data, BigEndian);
    // prelude + version number
    writer.write_bytes(b"VX").unwrap();
    writer.write(8, 2).unwrap();
    // election hash
    let truncated_election_hash = election_hash
        .get(0..20)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("election_hash is too short: {election_hash}"),
            )
        })
        .unwrap();
    let truncated_election_hash_bytes = hex::decode(truncated_election_hash)
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("election_hash is not a valid hex string: {election_hash}"),
            )
        })
        .unwrap();
    writer.write_bytes(&truncated_election_hash_bytes).unwrap();
    // check data
    writer.write(8, election.precincts.len() as u8).unwrap();
    writer.write(8, election.ballot_styles.len() as u8).unwrap();
    writer.write(8, election.contests.len() as u8).unwrap();
    // precinct index
    writer
        .write(sizeof(election.precincts.len() - 1), 2)
        .unwrap();
    // ballot style index
    writer
        .write(sizeof(election.ballot_styles.len() - 1), 0)
        .unwrap();
    // test ballot?
    writer.write_bit(true).unwrap();
    // ballot type
    writer
        .write(
            sizeof(VxBallotType::max() as usize),
            u32::from(VxBallotType::Precinct),
        )
        .unwrap();
    // ballot id?
    writer.write_bit(false).unwrap();
    // vote roll call only, no vote data
    for _ in contests {
        writer.write_bit(false).unwrap();
    }
    writer.byte_align().unwrap();

    let cvr_with_no_votes = build_cvr(
        &election_definition.election,
        BallotStyleId::from("1".to_string()),
        PrecinctId::from("21".to_string()),
        |_, _| (None, IndicationStatus::No),
    );
    let encodable_cvr = EncodableCvr::new(
        election_definition.election_hash,
        cvr_with_no_votes.clone(),
        TestMode::Test,
    );
    assert_bytes_equal(
        data.as_slice(),
        FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_NO_VOTES.as_slice(),
    );
    assert_eq!(
        encode(&election_definition.election, &encodable_cvr).unwrap(),
        data
    );
    let decoded_cvr = decode(&election_definition.election, data.as_slice()).unwrap();

    assert_cvrs_equivalent(&encodable_cvr, &decoded_cvr);

    let round_trip_encoded_cvr = encode(&election_definition.election, &decoded_cvr).unwrap();
    assert_bytes_equal(
        round_trip_encoded_cvr.as_slice(),
        FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_NO_VOTES.as_slice(),
    );
}

#[test]
fn test_empty_votes_from_machine_encoded() {
    let election_definition = read_famous_names_election_definition();
    let cvr_with_no_votes = build_cvr(
        &election_definition.election,
        BallotStyleId::from("1".to_string()),
        PrecinctId::from("21".to_string()),
        |_, _| (None, IndicationStatus::No),
    );
    let encodable_cvr = EncodableCvr::new(
        election_definition.election_hash,
        cvr_with_no_votes,
        TestMode::Test,
    );
    let encoded_cvr = encode(&election_definition.election, &encodable_cvr).unwrap();
    assert_bytes_equal(
        encoded_cvr.as_slice(),
        FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_NO_VOTES.as_slice(),
    );

    let decoded_cvr = decode(&election_definition.election, encoded_cvr.as_slice()).unwrap();
    assert_cvrs_equivalent(&encodable_cvr, &decoded_cvr);
}

#[test]
fn test_votes_from_machine_encoded() {
    let election_definition = read_famous_names_election_definition();
    let cvr_with_votes = build_cvr(
        &election_definition.election,
        BallotStyleId::from("1".to_string()),
        PrecinctId::from("21".to_string()),
        |_, option_id| match option_id {
            "thomas-edison"
            | "winston-churchill"
            | "mark-twain"
            | "bill-nye"
            | "alfred-hitchcock"
            | "johan-sebastian-bach"
            | "nikola-tesla"
            | "jackie-chan"
            | "tim-allen"
            | "harriet-tubman"
            | "marilyn-monroe" => (None, IndicationStatus::Yes),
            _ => (None, IndicationStatus::No),
        },
    );
    let encodable_cvr = EncodableCvr::new(
        election_definition.election_hash,
        cvr_with_votes,
        TestMode::Test,
    );
    assert_bytes_equal(
        encode(&election_definition.election, &encodable_cvr)
            .unwrap()
            .as_slice(),
        FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_VOTES_IN_ALL_CONTESTS.as_slice(),
    );
    assert_cvrs_equivalent(
        &decode(
            &election_definition.election,
            &FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_VOTES_IN_ALL_CONTESTS,
        )
        .unwrap(),
        &encodable_cvr,
    );
}

#[test]
fn test_votes_from_machine_encoded_with_write_ins() {
    let election_definition = read_famous_names_election_definition();
    let cvr_with_votes = build_cvr(
        &election_definition.election,
        BallotStyleId::from("1".to_string()),
        PrecinctId::from("22".to_string()),
        |contest_id, option_id| match (contest_id.to_string().as_str(), option_id) {
            (_, "thomas-edison")
            | (_, "oprah-winfrey")
            | (_, "john-snow")
            | (_, "bill-nye")
            | (_, "vincent-van-gogh")
            | (_, "wolfgang-amadeus-mozart")
            | (_, "tim-allen")
            | (_, "harriet-tubman") => (None, IndicationStatus::Yes),
            ("chief-of-police", "write-in-0") => {
                (Some("MERLIN".to_string()), IndicationStatus::Yes)
            }
            ("parks-and-recreation-director", "write-in-0") => (
                Some(r#"QWERTYUIOPASDFGHJKL'"ZXCVBNM,.- "'.,-POI"#.to_string()),
                IndicationStatus::Yes,
            ),
            ("board-of-alderman", "write-in-0") => {
                (Some("JOHN".to_string()), IndicationStatus::Yes)
            }
            ("board-of-alderman", "write-in-1") => {
                (Some("ALICE".to_string()), IndicationStatus::Yes)
            }
            _ => (None, IndicationStatus::No),
        },
    );
    let encodable_cvr = EncodableCvr::new(
        election_definition.election_hash,
        cvr_with_votes,
        TestMode::Test,
    );
    let decoded = decode(
        &election_definition.election,
        &FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_WRITE_INS,
    )
    .unwrap();

    assert_cvrs_equivalent(&decoded, &encodable_cvr);
    assert_bytes_equal(
        encode(&election_definition.election, &encodable_cvr)
            .unwrap()
            .as_slice(),
        FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_WRITE_INS.as_slice(),
    );
}
