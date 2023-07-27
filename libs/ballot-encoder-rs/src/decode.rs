use crate::{
    consts::{
        BITS_PER_WRITE_IN_CHAR, ELECTION_HASH_BYTE_LENGTH, ENCODING_VERSION,
        MAXIMUM_WRITE_IN_NAME_LENGTH, WRITE_IN_CHARS,
    },
    types::{BallotConfig, EncodableCvr, TestMode},
    util::sizeof,
};
use bitstream_io::{BigEndian, BitRead, BitReader, Endianness};
use std::io::Read;
use types_rs::{
    cdf::cvr::{
        CVRContest, CVRContestSelection, CVRSnapshot, CVRWriteIn, Cvr, IndicationStatus,
        SelectionPosition, VxBallotType,
    },
    election::{BallotStyleId, Contest, Election, PrecinctId},
};

/// Decodes a CVR from data encoded in a BMD ballot card.
pub fn decode(election: &Election, data: &[u8]) -> std::io::Result<EncodableCvr> {
    let mut reader = bitstream_io::BitReader::endian(data, BigEndian);
    let decoded = decode_from(election, &mut reader)?;

    while !reader.byte_aligned() {
        if reader.read_bit()? {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "unexpected non-zero padding bit",
            ));
        }
    }

    let mut remainder = vec![];
    let read_length = reader.into_reader().read_to_end(&mut remainder)?;

    if read_length > 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "unexpected trailing data",
        ));
    }

    Ok(decoded)
}

/// Decodes the encoded header from data encoded in a BMD ballot card. This
/// should be used to validate the version & election hash before decoding the
/// rest of the CVR.
pub fn decode_header(data: &[u8]) -> std::io::Result<(u8, String)> {
    let mut reader = bitstream_io::BitReader::endian(data, BigEndian);
    let version = decode_prelude_from(&mut reader)?;
    let election_hash = decode_election_hash_from(&mut reader)?;

    Ok((version, election_hash))
}

/// Decodes a CVR by reading data encoded in a BMD ballot card.
pub fn decode_from<E: Endianness>(
    election: &Election,
    reader: &mut BitReader<&[u8], E>,
) -> std::io::Result<EncodableCvr> {
    let version = decode_prelude_from(reader)?;

    if version != ENCODING_VERSION {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unsupported version: {}", version),
        ));
    }

    let election_hash = decode_election_hash_from(reader)?;
    let (precinct_count, ballot_style_count, contest_count) = decode_check_data_from(reader)?;

    if precinct_count as usize != election.precincts.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "precinct count mismatch: expected {}, but got {}",
                election.precincts.len(),
                precinct_count
            ),
        ));
    }

    if ballot_style_count as usize != election.ballot_styles.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "ballot style count mismatch: expected {}, but got {}",
                election.ballot_styles.len(),
                ballot_style_count
            ),
        ));
    }

    if contest_count as usize != election.contests.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "contest count mismatch: expected {}, but got {}",
                election.contests.len(),
                contest_count
            ),
        ));
    }

    let ballot_config = decode_ballot_config_from(election, reader)?;
    let ballot_style = election
        .ballot_styles
        .get(ballot_config.ballot_style_index as usize)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "cannot find ballot_style with index={}",
                    ballot_config.ballot_style_index
                ),
            )
        })?;
    let precinct = election
        .precincts
        .get(ballot_config.precinct_index as usize)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "cannot find precinct with index={}",
                    ballot_config.precinct_index
                ),
            )
        })?;
    let contests = election
        .get_contests(ballot_style.id.clone())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "cannot find contests for ballot_style_id={}",
                    ballot_style.id
                ),
            )
        })?;
    let ballot_votes = decode_ballot_votes_from(contests, &ballot_style.id, &precinct.id, reader)?;

    Ok(EncodableCvr::new(
        election_hash,
        ballot_votes,
        ballot_config.ballot_mode,
    ))
}

pub fn decode_prelude_from<E: Endianness>(reader: &mut BitReader<&[u8], E>) -> std::io::Result<u8> {
    let mut tag = [0u8; 2];
    reader.read_bytes(&mut tag)?;
    if &tag != b"VX" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid tag: {:?}", tag),
        ));
    }

    let version = reader.read(8)?;

    Ok(version)
}

pub fn decode_election_hash_from<E: Endianness>(
    reader: &mut BitReader<&[u8], E>,
) -> std::io::Result<String> {
    let mut election_hash = [0u8; ELECTION_HASH_BYTE_LENGTH];
    reader.read_bytes(&mut election_hash)?;

    Ok(hex::encode(election_hash))
}

pub fn decode_check_data_from<E: Endianness>(
    reader: &mut BitReader<&[u8], E>,
) -> std::io::Result<(u8, u8, u8)> {
    let precinct_count: u8 = reader.read(8)?;
    let ballot_style_count: u8 = reader.read(8)?;
    let contest_count: u8 = reader.read(8)?;

    Ok((precinct_count, ballot_style_count, contest_count))
}

pub fn decode_ballot_config_from<E: Endianness>(
    election: &Election,
    reader: &mut BitReader<&[u8], E>,
) -> std::io::Result<BallotConfig> {
    let precinct_index = reader.read::<u32>(sizeof(election.precincts.len() - 1))?;
    let ballot_style_index = reader.read::<u32>(sizeof(election.ballot_styles.len() - 1))?;
    let ballot_mode = if reader.read_bit()? {
        TestMode::Test
    } else {
        TestMode::Live
    };
    let ballot_type: VxBallotType = reader
        .read::<u32>(sizeof(VxBallotType::max() as usize))?
        .into();
    let ballot_id_present = reader.read_bit()?;
    let ballot_id = if ballot_id_present {
        let ballot_id_length = reader.read::<u8>(8)?;
        let mut ballot_id = vec![0u8; ballot_id_length as usize];
        reader.read_bytes(&mut ballot_id)?;
        Some(String::from_utf8(ballot_id).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "ballot_id is not a valid UTF-8 string",
            )
        })?)
    } else {
        None
    };

    Ok(BallotConfig {
        precinct_index,
        ballot_style_index,
        ballot_type,
        ballot_mode,
        ballot_id,
    })
}

pub fn decode_ballot_votes_from<E: Endianness>(
    contests: Vec<&Contest>,
    ballot_style_id: &BallotStyleId,
    precinct_id: &PrecinctId,
    reader: &mut BitReader<&[u8], E>,
) -> std::io::Result<Cvr> {
    let mut contests_with_votes = vec![];

    for contest in contests.iter() {
        if reader.read_bit()? {
            contests_with_votes.push(contest);
        }
    }

    let mut cvr_contests = vec![];

    for contest in contests.iter() {
        let contest_has_votes = contests_with_votes.contains(&contest);

        let cvr_contest_selections = match contest {
            Contest::YesNo(_) => {
                vec![
                    CVRContestSelection {
                        contest_selection_id: Some("yes".to_string()),
                        option_position: Some(0),
                        selection_position: vec![SelectionPosition {
                            has_indication: if contest_has_votes && reader.read_bit()? {
                                IndicationStatus::Yes
                            } else {
                                IndicationStatus::No
                            },
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    CVRContestSelection {
                        contest_selection_id: Some("no".to_string()),
                        option_position: Some(1),
                        selection_position: vec![SelectionPosition {
                            has_indication: if contest_has_votes && reader.read_bit()? {
                                IndicationStatus::Yes
                            } else {
                                IndicationStatus::No
                            },
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                ]
            }
            Contest::Candidate(candidate_contest) => {
                let mut cvr_contest_selections = vec![];
                let mut selected_candidate_count = 0usize;

                for (index, candidate) in candidate_contest.candidates.iter().enumerate() {
                    let has_vote = contest_has_votes && reader.read_bit()?;
                    if has_vote {
                        selected_candidate_count += 1;
                    }
                    cvr_contest_selections.push(CVRContestSelection {
                        contest_selection_id: Some(candidate.id.to_string()),
                        option_position: Some(index as i64),
                        selection_position: vec![SelectionPosition {
                            has_indication: if has_vote {
                                IndicationStatus::Yes
                            } else {
                                IndicationStatus::No
                            },
                            ..Default::default()
                        }],
                        ..Default::default()
                    });
                }

                if contest_has_votes && candidate_contest.allow_write_ins {
                    let maximum_possible_write_ins = (candidate_contest.seats as usize
                        - selected_candidate_count)
                        .clamp(0, candidate_contest.candidates.len());

                    if maximum_possible_write_ins > 0 {
                        let write_in_count =
                            reader.read::<u32>(sizeof(maximum_possible_write_ins))?;

                        for _ in 0..write_in_count {
                            let write_in_name = decode_write_in_name_from(reader)?;
                            cvr_contest_selections.push(CVRContestSelection {
                                contest_selection_id: None,
                                option_position: None,
                                selection_position: vec![SelectionPosition {
                                    has_indication: IndicationStatus::Yes,
                                    cvr_write_in: Some(CVRWriteIn {
                                        text: Some(write_in_name),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }],
                                ..Default::default()
                            });
                        }
                    }
                }

                cvr_contest_selections
            }
        };
        cvr_contests.push(CVRContest {
            contest_id: contest.id().to_string(),
            cvr_contest_selection: Some(cvr_contest_selections),
            ..Default::default()
        });
    }

    let cvr_snapshot = CVRSnapshot {
        cvr_contest: Some(cvr_contests),
        ..Default::default()
    };

    let cvr = Cvr {
        ballot_style_id: Some(ballot_style_id.to_string()),
        ballot_style_unit_id: Some(precinct_id.to_string()),
        current_snapshot_id: cvr_snapshot.id.clone(),
        cvr_snapshot: vec![cvr_snapshot],
        ..Default::default()
    };

    Ok(cvr)
}

pub fn decode_write_in_name_from<E: Endianness>(
    reader: &mut BitReader<&[u8], E>,
) -> std::io::Result<String> {
    let write_in_name_length = reader.read::<u32>(sizeof(MAXIMUM_WRITE_IN_NAME_LENGTH))?;

    if write_in_name_length > MAXIMUM_WRITE_IN_NAME_LENGTH as u32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("write-in name length is too long: {}", write_in_name_length),
        ));
    }

    let mut write_in_name = String::with_capacity(write_in_name_length as usize);

    for _ in 0..write_in_name_length {
        let index = reader.read::<u32>(BITS_PER_WRITE_IN_CHAR)?;
        let char = WRITE_IN_CHARS
            .get(index as usize..=index as usize)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("invalid write-in name char index: {}", index),
                )
            })?;
        write_in_name.push_str(char);
    }

    Ok(write_in_name)
}
