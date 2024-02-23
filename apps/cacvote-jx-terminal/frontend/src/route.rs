use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::layouts::AppLayout;
use crate::layouts::LoggedInLayout;
use crate::pages::BallotsPage;
use crate::pages::ElectionsPage;
use crate::pages::MachineLockedPage;
use crate::pages::NotFound;
use crate::pages::VotersPage;

#[derive(Clone, Debug, PartialEq, Routable)]
#[allow(clippy::enum_variant_names)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[layout(LoggedInLayout)]
            #[route("/elections")]
            ElectionsPage,
            #[route("/voters")]
            VotersPage,
            #[route("/ballots")]
            BallotsPage,
        #[end_layout]
        #[route("/")]
        MachineLockedPage,
    #[end_layout]
	#[route("/:..segments")]
	NotFound { segments: Vec<String> },
}

impl Route {
    pub fn label(&self) -> &'static str {
        match self {
            Self::MachineLockedPage => "Machine Locked",
            Self::ElectionsPage => "Elections",
            Self::VotersPage => "Voters",
            Self::BallotsPage => "Ballots",
            Self::NotFound { .. } => "Not Found",
        }
    }
}
