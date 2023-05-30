use types_rs::election::ElectionDefinition;

pub mod encoded;

pub fn read_famous_names_election_definition() -> ElectionDefinition {
    include_str!("./electionFamousNames2021.json")
        .parse::<ElectionDefinition>()
        .unwrap()
}
