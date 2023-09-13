use dioxus::prelude::*;
use types_rs::{cdf::cvr::Cvr, rave::jx};
use ui_rs::DateOrDateTimeCell;

use crate::components::ElectionConfigurationCell;

#[derive(PartialEq, Props)]
struct VotersProps<'a> {
    app_data: &'a jx::AppData,
}

pub fn BallotsPage(cx: Scope) -> Element {
    let app_data = use_shared_state::<jx::AppData>(cx).unwrap();
    let app_data = app_data.read();
    let elections = app_data.elections.clone();
    let printed_ballots = app_data.printed_ballots.clone();
    let scanned_ballots = app_data.scanned_ballots.clone();

    render!(
        h1 { class: "text-2xl font-bold mb-4", "Printed Ballots" }
        if printed_ballots.is_empty() {
            rsx!("No printed ballots")
        } else {
            to_owned![elections, printed_ballots];
            rsx!(PrintedBallotsTable {
                elections: elections,
                printed_ballots: printed_ballots,
            })
        }

        h1 { class: "text-2xl font-bold mt-4 mb-4", "Scanned Ballots" }
        if scanned_ballots.is_empty() {
            rsx!("No scanned ballots")
        } else {
            to_owned![elections, scanned_ballots];
            rsx!(ScannedBallotsTable {
                elections: elections,
                scanned_ballots: scanned_ballots,
            })
        }
    )
}

#[derive(PartialEq, Props)]
struct PendingRegistrationsTableProps {
    elections: Vec<jx::Election>,
    printed_ballots: Vec<jx::PrintedBallot>,
}

fn summarize_cast_vote_record(cvr: Cvr) -> String {
    let mut summary = String::new();

    for snapshot in cvr.cvr_snapshot {
        if let Some(contests) = snapshot.cvr_contest {
            for contest in contests {
                if let Some(contest_selections) = contest.cvr_contest_selection {
                    for contest_selection in contest_selections {
                        if let Some(contest_selection_id) = contest_selection.contest_selection_id {
                            summary.push_str(&format!(
                                "{}: {}\n",
                                contest.contest_id, contest_selection_id
                            ));
                        }
                    }
                }
            }
        }
    }

    summary
}

fn PrintedBallotsTable(cx: Scope<PendingRegistrationsTableProps>) -> Element {
    let elections = &cx.props.elections;

    let get_election_by_id = |election_id| {
        elections
            .iter()
            .find(|election| election.id() == election_id)
    };

    render!(
        div {
            rsx!(
                table { class: "table-auto w-full",
                    thead {
                        tr {
                            th { class: "px-4 py-2 text-left", "Election Configuration" }
                            th { class: "px-4 py-2 text-left", "Cast Vote Record" }
                            th { class: "px-4 py-2 text-left", "Created At" }
                        }
                    }
                    tbody {
                        for printed_ballot in cx.props.printed_ballots.iter() {
                            {
                                let election = get_election_by_id(printed_ballot.election_id()).unwrap();

                                rsx!(tr {
                                    ElectionConfigurationCell {
                                        election_title: election.title.clone(),
                                        election_hash: election.election_hash.clone(),
                                        precinct_id: printed_ballot.precinct_id().clone(),
                                        ballot_style_id: printed_ballot.ballot_style_id().clone(),
                                    }
                                    td {
                                        class: "border px-4 py-2 whitespace-nowrap",
                                        match printed_ballot.cast_vote_record() {
                                            Ok(cvr) => {
                                                rsx!(
                                                    match &printed_ballot.verification_status {
                                                        jx::VerificationStatus::Success { common_access_card_id, display_name } => {
                                                            rsx!(span {
                                                                class: "text-sm p-1 ps-0 pe-2 text-green-800 bg-green-300 font-semibold rounded-md",
                                                                title: "{display_name}",
                                                                span {
                                                                    class: "text-sm p-1 ps-2 pe-2 text-white bg-gray-400 font-semibold rounded-l-md",
                                                                    "CAC #{common_access_card_id}"
                                                                }
                                                                span {
                                                                    class: "ps-2",
                                                                    "Verified"
                                                                }
                                                            })
                                                        }
                                                        jx::VerificationStatus::Failure => {
                                                            rsx!(span {
                                                                class: "text-sm p-1 ps-0 pe-2 text-red-800 bg-red-300 font-semibold rounded-md",
                                                                span {
                                                                    class: "text-sm p-1 ps-2 pe-2 text-white bg-gray-400 font-semibold rounded-l-md",
                                                                    "CAC"
                                                                }
                                                                span {
                                                                    class: "ps-2",
                                                                    "Unverified"
                                                                }
                                                            })
                                                        }
                                                        jx::VerificationStatus::Error(err) => {
                                                            rsx!(span {
                                                                class: "text-sm p-1 ps-0 pe-2 text-orange-800 bg-orange-300 font-semibold rounded-md",
                                                                span {
                                                                    class: "text-sm p-1 ps-2 pe-2 text-white bg-gray-400 font-semibold rounded-l-md",
                                                                    "CAC"
                                                                }
                                                                span {
                                                                    class: "ps-2",
                                                                    title: "{err}",
                                                                    "Error"
                                                                }
                                                            })
                                                        }
                                                        jx::VerificationStatus::Unknown => {
                                                            rsx!(span {
                                                                class: "text-sm p-1 ps-0 pe-2 text-yellow-800 bg-yellow-300 font-semibold rounded-md",
                                                                span {
                                                                    class: "text-sm p-1 ps-2 pe-2 text-white bg-gray-400 font-semibold rounded-l-md",
                                                                    "CAC"
                                                                }
                                                                span {
                                                                    class: "ps-2",
                                                                    "Unknown"
                                                                }
                                                            })
                                                        }
                                                    }
                                                    details {
                                                        rsx!(summary {
                                                            class: "text-gray-200",
                                                            "DEBUG"
                                                        })
                                                        {
                                                            let summary = summarize_cast_vote_record(cvr);
                                                            rsx!(pre {
                                                                summary
                                                            })
                                                        }
                                                    }
                                                )
                                            }
                                            Err(e) => {
                                                rsx!(p {
                                                    class: "text-sm text-red-500",
                                                    "Cast vote record is invalid: {e}"
                                                })
                                            }
                                        }

                                    }
                                    DateOrDateTimeCell {
                                        date_or_datetime: printed_ballot.created_at(),
                                    }
                                })
                            }
                        }
                    }
                }
            )
        }
    )
}

#[derive(Debug, PartialEq, Props)]
struct RegistrationsTableProps {
    elections: Vec<jx::Election>,
    scanned_ballots: Vec<jx::ScannedBallot>,
}

fn ScannedBallotsTable(cx: Scope<RegistrationsTableProps>) -> Element {
    render!(
        table { class: "table-auto w-full",
            thead {
                tr {
                    th { class: "px-4 py-2 text-left", "Election" }
                    th { class: "px-4 py-2 text-left", "Created At" }
                }
            }
            tbody {
                for scanned_ballot in cx.props.scanned_ballots.iter() {
                    tr {
                        {
                            let election = cx
                                .props
                                .elections
                                .iter()
                                .find(|election| *election.id() == scanned_ballot.election_id)
                                .unwrap();
                            rsx!(ElectionConfigurationCell {
                                election_title: election.title.clone(),
                                election_hash: election.election_hash.clone(),
                                precinct_id: scanned_ballot.precinct_id.clone(),
                                ballot_style_id: scanned_ballot.ballot_style_id.clone(),
                            })
                        }
                        DateOrDateTimeCell {
                            date_or_datetime: scanned_ballot.created_at()
                        }
                    }
                }
            }
        }
    )
}
