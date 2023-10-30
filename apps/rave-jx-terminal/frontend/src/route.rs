use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::logged_in_layout::LoggedInLayout;
use crate::pages::BallotsPage;
use crate::pages::ChooseJurisdictionPage;
use crate::pages::ElectionsPage;
use crate::pages::VotersPage;

#[derive(Clone, Debug, PartialEq, Routable)]
#[allow(clippy::enum_variant_names)]
#[rustfmt::skip]
pub enum Route {
    #[nest("/jurisdictions/:jurisdiction_id")]
        #[layout(LoggedInLayout)]
            #[route("/elections")]
            ElectionsPage { jurisdiction_id: String },
            #[route("/voters")]
            VotersPage { jurisdiction_id: String },
            #[route("/ballots")]
            BallotsPage { jurisdiction_id: String },
        #[end_layout]
    #[end_nest]
    #[route("/")]
    ChooseJurisdictionPage,
}

impl Route {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ChooseJurisdictionPage => "Choose Jurisdiction",
            Self::ElectionsPage { .. } => "Elections",
            Self::VotersPage { .. } => "Voters",
            Self::BallotsPage { .. } => "Ballots",
        }
    }
}
