use std::{
    fs::{read_dir, DirBuilder, File},
    io,
    path::PathBuf,
};

use serde::Serialize;
use types_rs::election as vx_election;

#[derive(Debug, Serialize)]
pub(crate) struct Manifest {
    pub(crate) election_scope_id: String,
    pub(crate) spec_version: String,
    pub(crate) r#type: ElectionType,
    pub(crate) start_date: String,
    pub(crate) end_date: String,
    pub(crate) geopolitical_units: Vec<GeopoliticalUnit>,
    pub(crate) parties: Vec<Party>,
    pub(crate) candidates: Vec<Candidate>,
    pub(crate) contests: Vec<Contest>,
    pub(crate) ballot_styles: Vec<BallotStyle>,
    pub(crate) name: Vec<String>,
    pub(crate) contact_information: ContactInformation,
}

impl From<vx_election::Election> for Manifest {
    fn from(value: vx_election::Election) -> Self {
        Self {
            election_scope_id: "TestManifest".to_owned(),
            spec_version: "v2.0.0".to_owned(),
            r#type: if value.title.contains("Primary") {
                ElectionType::Primary
            } else {
                ElectionType::General
            },
            start_date: "start".to_owned(),
            end_date: "end".to_owned(),
            geopolitical_units: value.districts.into_iter().map(Into::into).collect(),
            parties: value.parties.into_iter().map(Into::into).collect(),
            candidates: value
                .contests
                .clone()
                .into_iter()
                .filter_map(|contest| match contest {
                    vx_election::Contest::Candidate(contest) => Some(contest.candidates),
                    vx_election::Contest::YesNo(_) => None,
                })
                .flat_map(|candidates| candidates.into_iter().map(Into::into))
                .collect(),
            contests: value
                .contests
                .into_iter()
                .enumerate()
                .map(|(i, contest)| Contest {
                    sequence_order: i as u32 + 1,
                    ..contest.into()
                })
                .collect(),
            ballot_styles: value.ballot_styles.into_iter().map(Into::into).collect(),
            name: Vec::new(),
            contact_information: ContactInformation::default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum ElectionType {
    #[serde(rename = "primary")]
    Primary,

    #[serde(rename = "general")]
    General,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ObjectId(String);

impl From<vx_election::DistrictId> for ObjectId {
    fn from(district_id: vx_election::DistrictId) -> Self {
        Self(format!("district-{district_id}"))
    }
}

impl From<vx_election::ContestId> for ObjectId {
    fn from(contest_id: vx_election::ContestId) -> Self {
        Self(format!("contest-{contest_id}"))
    }
}

impl From<vx_election::CandidateId> for ObjectId {
    fn from(candidate_id: vx_election::CandidateId) -> Self {
        Self(format!("cand-{candidate_id}"))
    }
}

impl From<vx_election::PartyId> for ObjectId {
    fn from(party_id: vx_election::PartyId) -> Self {
        Self(format!("party-{party_id}"))
    }
}

impl From<vx_election::BallotStyleId> for ObjectId {
    fn from(ballot_style_id: vx_election::BallotStyleId) -> Self {
        Self(format!("ballot-style-{ballot_style_id}"))
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct GeopoliticalUnit {
    pub(crate) object_id: ObjectId,
    pub(crate) name: String,
    pub(crate) r#type: GeopoliticalUnitType,
    pub(crate) contact_information: Option<Vec<ContactInformation>>,
}

impl From<vx_election::District> for GeopoliticalUnit {
    fn from(district: vx_election::District) -> Self {
        GeopoliticalUnit {
            object_id: district.id.into(),
            name: district.name,
            r#type: GeopoliticalUnitType::District,
            contact_information: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum GeopoliticalUnitType {
    #[serde(rename = "district")]
    District,
}

#[derive(Debug, Serialize)]
pub(crate) struct Party {
    pub(crate) object_id: ObjectId,
    pub(crate) name: String,
    pub(crate) abbreviation: String,
    pub(crate) color: Option<String>,
    pub(crate) logo_uri: Option<String>,
}

impl From<types_rs::election::Party> for Party {
    fn from(party: types_rs::election::Party) -> Self {
        Party {
            object_id: party.id.into(),
            name: party.name,
            abbreviation: party.abbrev,
            color: None,
            logo_uri: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Candidate {
    pub(crate) object_id: ObjectId,
    pub(crate) name: String,
    pub(crate) party_id: Option<ObjectId>,
    pub(crate) image_url: Option<String>,
    pub(crate) is_write_in: bool,
}

impl From<vx_election::Candidate> for Candidate {
    fn from(candidate: vx_election::Candidate) -> Self {
        Candidate {
            object_id: candidate.id.into(),
            name: candidate.name,
            party_id: candidate
                .party_ids
                .and_then(|party_ids| party_ids.first().cloned())
                .map(Into::into),
            image_url: None,
            is_write_in: false,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Contest {
    pub(crate) object_id: ObjectId,
    pub(crate) sequence_order: u32,
    pub(crate) electoral_district_id: ObjectId,
    pub(crate) vote_variation: VoteVariation,
    pub(crate) number_elected: u32,
    pub(crate) votes_allowed: u32,
    pub(crate) name: String,
    pub(crate) ballot_selections: Vec<BallotSelection>,
    pub(crate) ballot_title: Option<String>,
    pub(crate) ballot_subtitle: Option<String>,
}

impl From<vx_election::Contest> for Contest {
    fn from(contest: vx_election::Contest) -> Self {
        match contest {
            vx_election::Contest::Candidate(contest) => Contest {
                object_id: contest.id.clone().into(),
                sequence_order: 0,
                electoral_district_id: contest.district_id.into(),
                vote_variation: if contest.seats == 1 {
                    VoteVariation::OneOfM
                } else {
                    VoteVariation::NofM
                },
                votes_allowed: contest.seats,
                number_elected: 1,
                name: ObjectId::from(contest.id).0,
                ballot_selections: contest
                    .candidates
                    .into_iter()
                    .enumerate()
                    .map(|(i, candidate)| BallotSelection {
                        sequence_order: i as u32 + 1,
                        ..candidate.into()
                    })
                    .collect(),
                ballot_title: Some(contest.title),
                ballot_subtitle: None,
            },
            vx_election::Contest::YesNo(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct BallotSelection {
    pub(crate) object_id: ObjectId,
    pub(crate) sequence_order: u32,
    pub(crate) candidate_id: ObjectId,
}

impl From<vx_election::Candidate> for BallotSelection {
    fn from(value: vx_election::Candidate) -> Self {
        let candidate_id: ObjectId = value.id.into();
        BallotSelection {
            object_id: candidate_id.clone(),
            sequence_order: 0,
            candidate_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum VoteVariation {
    #[serde(rename = "one_of_m")]
    OneOfM,

    #[serde(rename = "n_of_m")]
    NofM,
}

#[derive(Debug, Serialize)]
pub(crate) struct BallotStyle {
    pub(crate) object_id: ObjectId,
    pub(crate) geopolitical_unit_ids: Vec<ObjectId>,
    pub(crate) party_ids: Vec<ObjectId>,
    pub(crate) image_uri: Option<String>,
}

impl From<vx_election::BallotStyle> for BallotStyle {
    fn from(ballot_style: vx_election::BallotStyle) -> Self {
        BallotStyle {
            object_id: ballot_style.id.into(),
            geopolitical_unit_ids: ballot_style.districts.into_iter().map(Into::into).collect(),
            party_ids: ballot_style
                .party_id
                .map(|party_id| vec![party_id.into()])
                .unwrap_or_default(),
            image_uri: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ContactInformation {
    pub(crate) name: String,
    pub(crate) address_line: Vec<String>,
    pub(crate) email: Option<String>,
    pub(crate) phone: Option<String>,
}

impl Default for ContactInformation {
    fn default() -> Self {
        Self {
            name: "contact".to_owned(),
            address_line: Vec::new(),
            email: None,
            phone: None,
        }
    }
}

pub(crate) struct ElectionConfig {
    pub(crate) public_metadata_blob: Vec<u8>,
    pub(crate) private_metadata_blob: Vec<u8>,
}

/// Generate ElectionGuard metadata for an election.
pub(crate) fn generate_election_config(
    classpath: &PathBuf,
    election: impl Into<Manifest>,
) -> io::Result<ElectionConfig> {
    let manifest: Manifest = election.into();

    // create a temporary working directory securely
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    // write the manifest to a file
    let manifest_path = temp_dir_path.join("manifest.json");
    let manifest_file = File::create(&manifest_path)?;
    serde_json::to_writer(manifest_file, &manifest)?;

    // create a temporary output directory securely
    let output_directory = temp_dir_path.join("output");
    DirBuilder::new().create(&output_directory)?;

    // run the Java ElectionGuard CLI to create the election configuration
    let trustees_directory = output_directory.join("trustees");
    run_create_election_config(classpath, &manifest_path, &output_directory)?;
    run_trusted_key_ceremony(
        classpath,
        &output_directory,
        &trustees_directory,
        &output_directory,
    )?;

    // at this point, the output directory should contain the public election
    // configuration & key and the private keys in the `trustees` directory
    let public_metadata_blob = zip_files_in_directory_to_buffer(&output_directory)?;
    let private_metadata_blob = zip_files_in_directory_to_buffer(&trustees_directory)?;

    Ok(ElectionConfig {
        public_metadata_blob,
        private_metadata_blob,
    })
}

/// Zip all files directly within a directory into a zip archive. This function
/// does not recursively zip files in subdirectories, and it does not include
/// the directory itself in the archive.
fn zip_files_in_directory_to_buffer(directory: &PathBuf) -> io::Result<Vec<u8>> {
    let mut zip_buffer = Vec::new();
    let writer = std::io::Cursor::new(&mut zip_buffer);
    let mut zip = zip::ZipWriter::new(writer);

    zip_files_in_directory(&mut zip, directory)?;

    zip.finish()?;

    // `zip` is holding on to `writer` which is holding on to `zip_buffer`,
    // so we need to drop `zip` to release the borrow on `zip_buffer`.
    drop(zip);

    Ok(zip_buffer)
}

/// Zip all files directly within a directory into a zip archive. This function
/// does not recursively zip files in subdirectories, and it does not include
/// the directory itself in the archive.
fn zip_files_in_directory<W>(zip: &mut zip::ZipWriter<W>, directory: &PathBuf) -> io::Result<()>
where
    W: io::Write + io::Seek,
{
    for entry in read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(&directory)
            .expect("entry must be in output directory");

        if path.is_file() {
            zip.start_file(
                name.to_str().expect("entry must have valid UTF-8 name"),
                Default::default(),
            )?;
            let mut file = File::open(&path)?;
            std::io::copy(&mut file, zip)?;
        }
    }

    Ok(())
}

/// Run the Java ElectionGuard CLI to create an election configuration. Expects
/// to read and write files because the Java ElectionGuard implementation
/// expects to work with files.
pub(crate) fn run_create_election_config(
    classpath: &PathBuf,
    manifest_path: &PathBuf,
    output_directory: &PathBuf,
) -> io::Result<()> {
    std::process::Command::new("java")
        .arg("-classpath")
        .arg(classpath)
        .arg("electionguard.cli.RunCreateElectionConfig")
        .arg("-manifest")
        .arg(manifest_path)
        .arg("-nguardians")
        .arg("1")
        .arg("-quorum")
        .arg("1")
        .arg("-out")
        .arg(output_directory)
        .arg("--baux0")
        .arg("device42")
        .output()
        .map(|output| {
            if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                if !stdout.is_empty() {
                    tracing::debug!("electionguard.cli.RunCreateElectionConfig stdout: {stdout}");
                }
            }

            if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
                if !stderr.is_empty() {
                    tracing::debug!("electionguard.cli.RunCreateElectionConfig stderr: {stderr}");
                }
            }
        })
}

/// Run the Java ElectionGuard CLI to create a trustee (private) election key.
/// Expects to read and write files because the Java ElectionGuard
/// implementation expects to work with files.
pub(crate) fn run_trusted_key_ceremony(
    classpath: &PathBuf,
    input_directory: &PathBuf,
    trustees_directory: &PathBuf,
    output_directory: &PathBuf,
) -> io::Result<()> {
    std::process::Command::new("java")
        .arg("-classpath")
        .arg(classpath)
        .arg("electionguard.cli.RunTrustedKeyCeremony")
        .arg("-in")
        .arg(input_directory)
        .arg("-trustees")
        .arg(trustees_directory)
        .arg("-out")
        .arg(output_directory)
        .output()
        .map(|output| {
            if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                if !stdout.is_empty() {
                    tracing::debug!("electionguard.cli.RunTrustedKeyCeremony stdout: {stdout}");
                }
            }

            if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
                if !stderr.is_empty() {
                    tracing::debug!("electionguard.cli.RunTrustedKeyCeremony stderr: {stderr}");
                }
            }
        })
}

#[cfg(test)]
mod tests {
    use types_rs::election::ElectionDefinition;

    use super::*;

    fn load_election_definition() -> color_eyre::Result<ElectionDefinition> {
        Ok(ElectionDefinition::try_from(
            &include_bytes!("../tests/fixtures/electionFamousNames2021.json")[..],
        )?)
    }

    #[test]
    fn test_generate_election_config() {
        if let Ok(classpath) = std::env::var("EG_CLASSPATH") {
            let election_definition = load_election_definition().unwrap();
            let election_config =
                generate_election_config(&PathBuf::from(classpath), election_definition.election)
                    .unwrap();

            let public_metadata_zip =
                zip::ZipArchive::new(std::io::Cursor::new(election_config.public_metadata_blob))
                    .unwrap();
            let mut file_names = public_metadata_zip.file_names().collect::<Vec<_>>();
            file_names.sort();
            assert_eq!(
                file_names,
                vec![
                    "constants.json",
                    "election_config.json",
                    "election_initialized.json",
                    "manifest.json",
                ]
            );

            let private_metadata_zip =
                zip::ZipArchive::new(std::io::Cursor::new(election_config.private_metadata_blob))
                    .unwrap();
            let mut file_names = private_metadata_zip.file_names().collect::<Vec<_>>();
            file_names.sort();
            assert_eq!(file_names, vec!["decryptingTrustee-trustee1.json"]);
        } else {
            eprintln!("EG_CLASSPATH environment variable not set");
        }
    }
}
