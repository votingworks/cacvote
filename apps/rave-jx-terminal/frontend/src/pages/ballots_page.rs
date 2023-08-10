use dioxus::{html::span, prelude::*};
use types_rs::{cdf::cvr::Cvr, rave::jx};

use crate::components::{DateOrDateTimeColumn, ElectionConfigurationColumn};

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
            rsx!(PrintedBallotsTable {
                elections: elections,
                printed_ballots: printed_ballots,
            })
        }

        h1 { class: "text-2xl font-bold mt-4 mb-4", "Scanned Ballots" }
        if scanned_ballots.is_empty() {
            rsx!("No scanned ballots")
        } else {
            rsx!(ScannedBallotsTable {
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
                                    ElectionConfigurationColumn {
                                        election_title: election.title.clone(),
                                        election_hash: election.election_hash.clone(),
                                        precinct_id: printed_ballot.precinct_id().clone(),
                                        ballot_style_id: printed_ballot.ballot_style_id().clone(),
                                    }
                                    td {
                                        class: "border px-4 py-2",
                                        match printed_ballot.cast_vote_record() {
                                            Ok(cvr) => {
                                                rsx!(
                                                    p {
                                                        class: "text-sm text-green-500",
                                                        "Cast vote record is valid"
                                                    }
                                                    details {
                                                        rsx!(summary {
                                                            "Click to view"
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
                                    DateOrDateTimeColumn {
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
                        td { class: "border px-4 py-2", "TODO" }
                        DateOrDateTimeColumn {
                            date_or_datetime: scanned_ballot.created_at()
                        }
                    }
                }
            }
        }
    )
}
