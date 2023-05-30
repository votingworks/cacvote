use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::layout::Layout;
use crate::pages::BallotsPage;
use crate::pages::ElectionsPage;
use crate::pages::VotersPage;

#[derive(Clone, Debug, PartialEq, Routable)]
#[allow(clippy::enum_variant_names)]
pub enum Route {
    #[layout(Layout)]
    #[redirect("/", || Route::ElectionsPage)]
    #[route("/elections")]
    ElectionsPage,
    #[route("/voters")]
    VotersPage,
    #[route("/ballots")]
    BallotsPage,
}

impl Route {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ElectionsPage => "Elections",
            Self::VotersPage => "Voters",
            Self::BallotsPage => "Ballots",
        }
    }
}
