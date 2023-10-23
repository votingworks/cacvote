use crate::{
    consts::{
        BITS_PER_WRITE_IN_CHAR, ENCODING_VERSION, MAXIMUM_WRITE_IN_NAME_LENGTH, WRITE_IN_CHARS,
    },
    types::{EncodableCvr, TestMode},
    util::sizeof,
};
use bitstream_io::{BigEndian, BitWrite, BitWriter, Endianness};
use types_rs::{
    cdf::cvr::{CVRContestSelection, Cvr, IndicationStatus, VxBallotType},
    election::{
        BallotStyleId, CandidateContest, Contest, Election, PartialElectionHash, PrecinctId,
        YesNoContest,
    },
};

pub fn encode(election: &Election, encodable_cvr: &EncodableCvr) -> std::io::Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = BitWriter::endian(&mut data, BigEndian);

    encode_into(election, encodable_cvr, &mut writer)?;
    writer.byte_align()?;

    Ok(data)
}

pub fn encode_into<E: Endianness>(
    election: &Election,
    encodable_cvr: &EncodableCvr,
    writer: &mut BitWriter<&mut Vec<u8>, E>,
) -> std::io::Result<()> {
    encode_prelude_into(writer)?;

    // TODO: validate votes

    let cvr = &encodable_cvr.cvr;
    let test_mode = &encodable_cvr.test_mode;
    let partial_election_hash = encodable_cvr.partial_election_hash.clone();

    let precinct_id = PrecinctId::from(cvr.ballot_style_unit_id.clone().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "ballot_style_unit_id is missing from CVR",
        )
    })?);
    let ballot_style_id = BallotStyleId::from(cvr.ballot_style_id.clone().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "ballot_style_id is missing from CVR",
        )
    })?);
    let contests = election
        .get_contests(ballot_style_id.clone())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("cannot find contests for ballot_style_id={ballot_style_id}"),
            )
        })?;

    encode_election_hash_into(&partial_election_hash, writer)?;
    encode_ballot_config_into(
        election,
        ballot_style_id,
        precinct_id,
        *test_mode,
        cvr.vx_ballot_type.clone(),
        cvr.unique_id.as_deref(),
        writer,
    )?;
    encode_ballot_votes_into(contests, cvr, writer)?;

    Ok(())
}

pub fn encode_prelude_into<E: Endianness>(
    writer: &mut BitWriter<&mut Vec<u8>, E>,
) -> std::io::Result<()> {
    // write prelude
    writer.write_bytes(b"VX")?; // tag
    writer.write(8, ENCODING_VERSION)?; // version

    Ok(())
}

pub fn encode_election_hash_into<E: Endianness>(
    partial_election_hash: &PartialElectionHash,
    writer: &mut BitWriter<&mut Vec<u8>, E>,
) -> std::io::Result<()> {
    writer.write_bytes(&partial_election_hash.to_bytes())?;

    Ok(())
}

pub fn encode_ballot_config_into<E: Endianness>(
    election: &Election,
    ballot_style_id: BallotStyleId,
    precinct_id: PrecinctId,
    test_mode: TestMode,
    ballot_type: VxBallotType,
    ballot_id: Option<&str>,
    writer: &mut BitWriter<&mut Vec<u8>, E>,
) -> std::io::Result<()> {
    let precinct_id = precinct_id.to_string();
    let ballot_style_id = ballot_style_id.to_string();
    let precinct_index = election
        .precincts
        .iter()
        .position(|precinct| precinct.id.to_string() == precinct_id)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("cannot find precinct with id={}", precinct_id),
            )
        })?;
    let ballot_style_index = election
        .ballot_styles
        .iter()
        .position(|ballot_style| ballot_style.id.to_string() == ballot_style_id)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("cannot find ballot_style with id={}", ballot_style_id),
            )
        })?;

    writer.write(8, election.precincts.len() as u8)?;
    writer.write(8, election.ballot_styles.len() as u8)?;
    writer.write(8, election.contests.len() as u8)?;
    writer.write(sizeof(election.precincts.len() - 1), precinct_index as u32)?;
    writer.write(
        sizeof(election.ballot_styles.len() - 1),
        ballot_style_index as u32,
    )?;
    writer.write_bit(test_mode == TestMode::Test)?;
    writer.write(sizeof(VxBallotType::max() as usize), u32::from(ballot_type))?;
    if let Some(ballot_id) = ballot_id {
        writer.write_bit(true)?;
        writer.write(8, ballot_id.len() as u8)?;
        writer.write_bytes(ballot_id.as_bytes())?;
    } else {
        writer.write_bit(false)?;
    }

    Ok(())
}

pub fn encode_ballot_votes_into<E: Endianness>(
    contests: Vec<&types_rs::election::Contest>,
    cvr: &Cvr,
    writer: &mut BitWriter<&mut Vec<u8>, E>,
) -> std::io::Result<()> {
    let snapshot = cvr
        .cvr_snapshot
        .iter()
        .find(|snapshot| snapshot.id == cvr.current_snapshot_id)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "cannot find CVR snapshot with id={}",
                    cvr.current_snapshot_id
                ),
            )
        })?;

    let all_cvr_contests = snapshot.cvr_contest.as_ref().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "cvr_contest is missing from CVR snapshot",
        )
    })?;

    let all_cvr_contests_paired_with_contest_definition = contests
        .iter()
        .map(|contest| {
            let contest_id = contest.id();
            let cvr_contest = all_cvr_contests
                .iter()
                .find(|cvr_contest| contest_id.to_string() == cvr_contest.contest_id)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("cannot find contest with id={}", contest_id),
                    )
                })?;
            Ok((cvr_contest, contest))
        })
        .collect::<std::io::Result<Vec<_>>>()?;

    let cvr_contests_with_votes = all_cvr_contests
        .iter()
        .filter(|cvr_contest| {
            cvr_contest
                .cvr_contest_selection
                .as_ref()
                .map(|cvr_contest_selection| {
                    cvr_contest_selection
                        .iter()
                        .flat_map(|selection| &selection.selection_position)
                        .any(|selection_position| {
                            selection_position.has_indication == IndicationStatus::Yes
                        })
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    // write roll call
    for (cvr_contest, _) in all_cvr_contests_paired_with_contest_definition.iter() {
        let has_votes = cvr_contests_with_votes.contains(cvr_contest);
        writer.write_bit(has_votes)?;
    }

    for (cvr_contest, contest) in all_cvr_contests_paired_with_contest_definition.iter() {
        if !cvr_contests_with_votes.contains(cvr_contest) {
            continue;
        }

        let selections = cvr_contest.cvr_contest_selection.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cvr_contest_selection is missing from CVR contest",
            )
        })?;

        match contest {
            Contest::YesNo(yesno_contest) => {
                encode_yesno_selections(yesno_contest, selections, writer)?;
            }
            Contest::Candidate(candidate_contest) => {
                encode_candidate_selections(candidate_contest, selections, writer)?;
            }
        }
    }

    Ok(())
}

fn encode_candidate_selections<E: Endianness>(
    candidate_contest: &CandidateContest,
    selections: &[CVRContestSelection],
    writer: &mut BitWriter<&mut Vec<u8>, E>,
) -> std::io::Result<()> {
    // candidate choices get one bit per candidate
    let mut non_write_in_count = 0;
    for candidate in candidate_contest.candidates.iter() {
        let selection = selections
            .iter()
            .find(|selection| {
                selection
                    .contest_selection_id
                    .as_ref()
                    .map(|contest_selection_id| contest_selection_id == &candidate.id.to_string())
                    .unwrap_or(false)
            })
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "cannot find selection for candidate with id={}",
                        candidate.id
                    ),
                )
            })?;
        let is_selected = selection
            .selection_position
            .iter()
            .any(|selection_position| selection_position.has_indication == IndicationStatus::Yes);
        writer.write_bit(is_selected)?;

        if is_selected {
            non_write_in_count += 1;
        }
    }

    if candidate_contest.allow_write_ins {
        // write write-in data
        let write_in_count = selections
            .iter()
            .filter(|selection| {
                selection
                    .selection_position
                    .iter()
                    .any(|selection_position| {
                        selection_position.has_indication == IndicationStatus::Yes
                            && selection_position.cvr_write_in.is_some()
                    })
            })
            .count();
        let maximum_write_ins = (candidate_contest.seats - non_write_in_count).max(0);

        if maximum_write_ins > 0 {
            writer.write(sizeof(maximum_write_ins as usize), write_in_count as u32)?;

            for selection in selections.iter() {
                if selection
                    .selection_position
                    .iter()
                    .any(|selection_position| {
                        selection_position.has_indication == IndicationStatus::Yes
                            && selection_position.cvr_write_in.is_some()
                    })
                {
                    let write_in_name = selection
                        .selection_position
                        .iter()
                        .find_map(|selection_position| {
                            selection_position
                                .cvr_write_in
                                .as_ref()
                                .map(|cvr_write_in| cvr_write_in.text.as_ref())
                        })
                        .flatten()
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "write-in name is missing from selection",
                            )
                        })?;
                    encode_write_in_name(write_in_name, writer)?;
                }
            }
        }
    }

    Ok(())
}

pub fn encode_write_in_name<E: Endianness>(
    write_in_name: &str,
    writer: &mut BitWriter<&mut Vec<u8>, E>,
) -> std::io::Result<()> {
    if write_in_name.len() > MAXIMUM_WRITE_IN_NAME_LENGTH {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("write-in name is too long: {}", write_in_name.len()),
        ));
    }

    writer.write(
        sizeof(MAXIMUM_WRITE_IN_NAME_LENGTH),
        write_in_name.len() as u8,
    )?;

    for char in write_in_name.chars() {
        let index = WRITE_IN_CHARS.find(char).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid write-in name char: {char}"),
            )
        })?;
        writer.write(BITS_PER_WRITE_IN_CHAR, index as u32)?;
    }

    Ok(())
}

pub fn encode_yesno_selections<E: Endianness>(
    contest: &YesNoContest,
    selections: &Vec<CVRContestSelection>,
    writer: &mut BitWriter<&mut Vec<u8>, E>,
) -> std::io::Result<()> {
    if selections.len() != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "expected 1 selection for contest with id={}, but got {}",
                contest.id,
                selections.len()
            ),
        ));
    }

    let selection = &selections[0];
    let is_yes_vote = selection
        .contest_selection_id
        .as_ref()
        .map(|contest_selection_id| contest_selection_id == "yes")
        .unwrap_or(false);

    writer.write_bit(is_yes_vote)?;

    Ok(())
}
