use ballot_encoder_rs::EncodableCvr;
use nanoid::nanoid;
use pretty_assertions::assert_eq;
use types_rs::cdf::cvr::{
    CVRContest, CVRContestSelection, CVRSnapshot, CVRWriteIn, Cvr, IndicationStatus,
    SelectionPosition,
};
use types_rs::election::{BallotStyleId, Contest, ContestId, Election, PrecinctId};

pub fn build_cvr(
    election: &Election,
    ballot_style_id: BallotStyleId,
    precinct_id: PrecinctId,
    get_indication_status: impl Fn(&ContestId, &str) -> (Option<String>, IndicationStatus),
) -> Cvr {
    let id = nanoid!();
    let contests = election
        .get_contests(ballot_style_id.clone())
        .unwrap()
        .iter()
        .map(|contest| {
            let contest_id = contest.id().to_string();
            let contest_selection_ids = contest
                .option_ids()
                .iter()
                .map(|option_id| option_id.to_string())
                .collect::<Vec<_>>();
            build_cvr_contest(
                ContestId::from(contest_id),
                contest_selection_ids.iter().map(|s| s.as_str()).collect(),
                match contest {
                    Contest::YesNo(_) => 0,
                    Contest::Candidate(candidate_contest) => {
                        if candidate_contest.allow_write_ins {
                            candidate_contest.seats as usize
                        } else {
                            0
                        }
                    }
                },
                &get_indication_status,
            )
        })
        .collect();
    Cvr {
        ballot_style_id: Some(ballot_style_id.to_string()),
        ballot_style_unit_id: Some(precinct_id.to_string()),
        current_snapshot_id: id.clone(),
        cvr_snapshot: vec![CVRSnapshot {
            id,
            cvr_contest: Some(contests),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn assert_cvrs_equivalent(left: &EncodableCvr, right: &EncodableCvr) {
    let mut left: EncodableCvr = (*left).clone();

    // check consistency
    assert_eq!(left.cvr.current_snapshot_id, left.cvr.cvr_snapshot[0].id);
    assert_eq!(right.cvr.current_snapshot_id, right.cvr.cvr_snapshot[0].id);

    // update the snapshot IDs to match
    left.cvr.current_snapshot_id = right.cvr.current_snapshot_id.clone();
    left.cvr.cvr_snapshot[0].id = right.cvr.cvr_snapshot[0].id.clone();

    assert_eq!(left, *right);
}

pub fn assert_bytes_equal(left: &[u8], right: &[u8]) {
    assert_eq!(pretty_bytes(left), pretty_bytes(right));
}

fn pretty_bytes(bytes: &[u8]) -> String {
    let mut binary_string = String::new();
    for byte in bytes.iter() {
        let char = char::from(*byte);
        binary_string.push_str(&format!(
            "{:08b} {:02x?} {}\n",
            byte,
            byte,
            if char.is_alphanumeric() { char } else { ' ' },
        ));
    }
    binary_string
}

fn build_cvr_contest(
    contest_id: ContestId,
    contest_selection_ids: Vec<&str>,
    possible_write_in_count: usize,
    get_indication_status: impl Fn(&ContestId, &str) -> (Option<String>, IndicationStatus),
) -> CVRContest {
    let write_in_contest_selections = (0..possible_write_in_count).flat_map(|index| {
        let (write_in_name, has_indication) =
            get_indication_status(&contest_id, &format!("write-in-{}", index));

        write_in_name.map(|write_in_name| CVRContestSelection {
            contest_selection_id: None,
            option_position: None,
            selection_position: vec![SelectionPosition {
                has_indication,
                cvr_write_in: Some(CVRWriteIn {
                    text: Some(write_in_name),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        })
    });
    CVRContest {
        contest_id: contest_id.to_string(),
        cvr_contest_selection: Some(
            contest_selection_ids
                .iter()
                .enumerate()
                .map(|(option_position, contest_selection_id)| {
                    let (write_in_name, has_indication) =
                        get_indication_status(&contest_id, contest_selection_id);
                    CVRContestSelection {
                        contest_selection_id: Some(contest_selection_id.to_string()),
                        option_position: Some(option_position as i64),
                        selection_position: vec![SelectionPosition {
                            has_indication,
                            cvr_write_in: write_in_name.map(|text| CVRWriteIn {
                                text: Some(text),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }
                })
                .chain(write_in_contest_selections)
                .collect(),
        ),
        ..Default::default()
    }
}

pub fn sizeof(number: usize) -> u32 {
    let mut size = 0;
    let mut n = number;
    while n > 0 {
        size += 1;
        n >>= 1;
    }
    size.max(1)
}
