#![allow(non_snake_case)]

use dioxus::prelude::*;
use types_rs::election::{BallotStyleId, ElectionHash, PrecinctId};
use ui_rs::TableCell;

#[derive(Debug, Props, PartialEq)]
pub struct Props {
    #[props(into)]
    election_title: String,
    election_hash: ElectionHash,
    precinct_id: PrecinctId,
    ballot_style_id: BallotStyleId,
}

pub fn ElectionConfigurationCell(cx: Scope<Props>) -> Element {
    let election_title = &cx.props.election_title;
    let election_hash = &cx.props.election_hash;
    let precinct_id = &cx.props.precinct_id;
    let ballot_style_id = &cx.props.ballot_style_id;

    render!(
        TableCell {
            p {
                "{election_title}"
                span {
                    class: "italic text-gray-400",
                    " ({election_hash.to_partial()})"
                }
            }
            p {
                "{ballot_style_id} / {precinct_id}"
                span {
                    class: "italic text-gray-400",
                    " (Ballot Style / Precinct)"
                }
            }
        }
    )
}
