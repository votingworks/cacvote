use dioxus::prelude::*;
use types_rs::cacvote;
use ui_rs::DateOrDateTimeCell;

use crate::components::ElectionConfigurationCell;

pub fn BallotsPage(cx: Scope) -> Element {
    let session_data = use_shared_state::<cacvote::SessionData>(cx).unwrap();
    let cacvote::SessionData::Authenticated {
        elections,
        cast_ballots,
        ..
    } = &*session_data.read()
    else {
        return render!(h1 { class: "text-2xl font-bold", "Please log in to view this page" });
    };

    render!(
        h1 { class: "text-2xl font-bold mb-4", "Cast Ballots" }
        if cast_ballots.is_empty() {
            rsx!("No cast ballots")
        } else {
            rsx!(CastBallotsTable {
                elections: elections.clone(),
                cast_ballots: cast_ballots.clone(),
            })
        }
    )
}

#[derive(PartialEq, Props)]
struct CastBallotsTableProps {
    elections: Vec<cacvote::ElectionPresenter>,
    cast_ballots: Vec<cacvote::CastBallotPresenter>,
}

fn CastBallotsTable(cx: Scope<CastBallotsTableProps>) -> Element {
    let elections = &cx.props.elections;

    let get_election_by_id =
        |election_id| elections.iter().find(|election| election.id == election_id);

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
                        for cast_ballot in cx.props.cast_ballots.iter() {
                            {
                                let election = get_election_by_id(cast_ballot.election_object_id).unwrap();

                                rsx!(tr {
                                    ElectionConfigurationCell {
                                        election_title: election.election_definition.election.title.clone(),
                                        election_hash: election.election_hash.clone(),
                                        precinct_id: cast_ballot.registration().precinct_id.clone(),
                                        ballot_style_id: cast_ballot.registration().ballot_style_id.clone(),
                                    }
                                    td {
                                        class: "border px-4 py-2 whitespace-nowrap",
                                        rsx!(
                                            match cast_ballot.verification_status() {
                                                cacvote::VerificationStatus::Success { common_access_card_id, display_name } => {
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
                                                cacvote::VerificationStatus::Failure => {
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
                                                cacvote::VerificationStatus::Error(err) => {
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
                                                cacvote::VerificationStatus::Unknown => {
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
                                        )
                                    }
                                    DateOrDateTimeCell {
                                        date_or_datetime: cast_ballot.created_at(),
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
