use std::{
    fs::{read_dir, DirBuilder, File},
    io::{self, Read},
    ops::Range,
    path::PathBuf,
};

use color_eyre::eyre::{bail, ensure};
use rand::Rng;
use serde::{Deserialize, Serialize};
use types_rs::{
    cdf::cvr::{AllocationStatus, Cvr},
    election,
};

use crate::{
    command::run_electionguard_command,
    manifest::{Manifest, ObjectId},
    zip::{unzip_into_directory, UnzipLimits},
};

/// The range of serial numbers for ballots. Max is `Number.MAX_SAFE_INTEGER`
/// from JS.
const SERIAL_NUMBER_RANGE: Range<u64> = 1..(2 ^ 53 - 1);

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PlaintextBallot {
    pub ballot_id: ObjectId, // a unique ballot ID created by the external system
    pub ballot_style: ObjectId, // matches a Manifest.BallotStyle
    pub contests: Vec<Contest>,
    pub sn: Option<u64>,        // must be > 0
    pub errors: Option<String>, // error messages from processing, eg when invalid
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Contest {
    pub contest_id: ObjectId, // matches ContestDescription.contestId
    pub sequence_order: u32,  // matches ContestDescription.sequenceOrder
    pub selections: Vec<Selection>,
    pub write_ins: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Selection {
    pub selection_id: ObjectId, // matches SelectionDescription.selectionId
    pub sequence_order: u32,    // matches SelectionDescription.sequenceOrder
    pub vote: u32,
}

pub fn convert_vx_cvr_to_eg_plaintext_ballot(
    cvr: Cvr,
    manifest: Manifest,
    election: election::Election,
) -> color_eyre::Result<PlaintextBallot> {
    let converted_manifest: Manifest = election.clone().into();

    ensure!(
        manifest == converted_manifest,
        "VX election and EG manifest do not match"
    );

    let Some(unique_id) = cvr.unique_id else {
        bail!("Missing unique ID in CVR");
    };

    let Some(vx_ballot_style_id) = cvr.ballot_style_id else {
        bail!("Missing ballot style ID in CVR");
    };

    let Some(ballot_style_index) = election
        .ballot_styles
        .iter()
        .position(|bs| bs.id.to_string() == vx_ballot_style_id)
    else {
        bail!("Ballot style ID not found in election: {vx_ballot_style_id}");
    };

    let Some(eg_ballot_style) = manifest.ballot_styles.get(ballot_style_index) else {
        bail!("Ballot style ID not found in manifest: {vx_ballot_style_id}");
    };

    let mut contests = Vec::new();

    let [cvr_snapshot] = cvr.cvr_snapshot.as_slice() else {
        bail!("Expected exactly one CVR snapshot in CVR");
    };

    let Some(ref cvr_contests) = cvr_snapshot.cvr_contest else {
        bail!("Missing contests in CVR snapshot");
    };

    for cvr_contest in cvr_contests {
        let contest_id = cvr_contest.contest_id.clone();
        let Some((contest_index, contest)) = election
            .contests
            .iter()
            .enumerate()
            .find(|(_, c)| c.id().to_string() == contest_id)
        else {
            bail!("Contest ID not found in election: {contest_id}");
        };
        let Some(eg_contest) = manifest.contests.get(contest_index) else {
            bail!("Contest not found in manifest: {contest_id}");
        };
        let mut selections = Vec::new();

        // TODO: handle write-ins
        let write_ins = Vec::new();

        let Some(ref cvr_contest_selection) = cvr_contest.cvr_contest_selection else {
            bail!("Missing selections in CVR contest");
        };

        let election::Contest::Candidate(contest) = contest else {
            bail!("Unsupported contest type: {contest:?}");
        };

        for selection in cvr_contest_selection {
            let Some(selection_id) = selection.contest_selection_id.clone() else {
                bail!("Missing selection ID in CVR contest selection (contest ID: {contest_id})")
            };
            let Some(candidate_index) = contest
                .candidates
                .iter()
                .position(|c| c.id.to_string() == selection_id)
            else {
                bail!("Selection ID not found in election: {selection_id}");
            };
            let Some(eg_ballot_selection) = eg_contest.ballot_selections.get(candidate_index)
            else {
                bail!("Selection not found in manifest: {selection_id}");
            };

            let Some(selection_position) = selection
                .selection_position
                .iter()
                .find(|p| matches!(p.is_allocable, Some(AllocationStatus::Yes)))
            else {
                bail!("Selection position not allocable: {selection_id}");
            };

            let vote = selection_position.number_votes as u32;
            selections.push(Selection {
                selection_id: eg_ballot_selection.object_id.clone(),
                sequence_order: candidate_index as u32 + 1,
                vote,
            });
        }

        contests.push(Contest {
            contest_id: eg_contest.object_id.clone(),
            sequence_order: contest_index as u32 + 1,
            selections,
            write_ins,
        });
    }

    let plaintext_ballot = PlaintextBallot {
        ballot_id: ObjectId(unique_id),
        ballot_style: eg_ballot_style.object_id.clone(),
        contests,
        sn: Some(rand::thread_rng().gen_range(SERIAL_NUMBER_RANGE)),
        errors: None,
    };

    Ok(plaintext_ballot)
}

pub fn encrypt(
    classpath: &PathBuf,
    public_metadata_blob: &[u8],
    plaintext_ballot: &PlaintextBallot,
) -> io::Result<Vec<u8>> {
    // create a temporary working directory securely
    let temp_directory = tempfile::tempdir()?;
    let temp_path = temp_directory.path();

    let config_directory = temp_path.join("config");
    DirBuilder::new().create(&config_directory)?;

    let input_directory = temp_path.join("input");
    DirBuilder::new().create(&input_directory)?;

    let output_directory = temp_path.join("output");
    DirBuilder::new().create(&output_directory)?;

    // unzip the public metadata blob into the config directory
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(public_metadata_blob))?;
    unzip_into_directory(&mut zip, &config_directory, UnzipLimits::default())?;

    // write the plaintext ballot to a file
    let plaintext_ballot_path = input_directory.join("ballot.json");
    let plaintext_ballot_file = File::create(&plaintext_ballot_path)?;
    serde_json::to_writer(plaintext_ballot_file, plaintext_ballot)?;

    // run the Java ElectionGuard CLI to encrypt the ballot
    run_encrypt_ballot(
        classpath,
        &config_directory,
        &plaintext_ballot_path,
        &output_directory,
    )?;

    let encrypted_ballots_directory = output_directory.join("encrypted_ballots");
    let output_directory = if encrypted_ballots_directory.exists() {
        encrypted_ballots_directory
    } else {
        output_directory
    };

    // at this point, the output directory should contain a single encrypted ballot file
    let output_paths: Vec<_> = read_dir(&output_directory)?
        .flatten()
        .filter(|e| matches!(e.file_type(), Ok(file_type) if file_type.is_file()))
        .take(2)
        .collect();

    match output_paths.as_slice() {
        [encrypted_ballot_path] => File::open(encrypted_ballot_path.path())?.bytes().collect(),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected exactly one encrypted ballot file, found {output_paths:?}"),
            ));
        }
    }
}

/// Run the Java ElectionGuard CLI to encrypt a ballot. Expects to read and
/// write files because the Java ElectionGuard implementation expects to work
/// with files.
pub fn run_encrypt_ballot(
    classpath: &PathBuf,
    config_directory: &PathBuf,
    plaintext_ballot_path: &PathBuf,
    output_directory: &PathBuf,
) -> io::Result<()> {
    DirBuilder::new().recursive(true).create(output_directory)?;

    run_electionguard_command(
        std::process::Command::new("java")
            .arg("-classpath")
            .arg(classpath)
            .arg("org.cryptobiotic.eg.cli.RunEncryptBallot")
            .arg("-in")
            .arg(config_directory)
            .arg("-ballot")
            .arg(plaintext_ballot_path)
            .arg("-out")
            .arg(output_directory)
            .arg("-device")
            .arg("device42")
            .arg("--noDeviceNameInDir"),
    )
}

#[cfg(test)]
mod tests {
    use types_rs::election::ElectionDefinition;

    use crate::{config::generate_election_config, manifest::ObjectId};

    use super::*;

    fn load_election_definition() -> color_eyre::Result<ElectionDefinition> {
        Ok(ElectionDefinition::try_from(
            &include_bytes!("../tests/fixtures/electionFamousNames2021.json")[..],
        )?)
    }

    #[test]
    fn test_encrypt_ballot() {
        if let Ok(classpath) = std::env::var("EG_CLASSPATH") {
            let election_definition = load_election_definition().unwrap();
            let manifest: Manifest = election_definition.election.clone().into();
            let election_config = generate_election_config(
                &PathBuf::from(&classpath),
                election_definition.election.clone(),
            )
            .unwrap();

            let plaintext_ballot = PlaintextBallot {
                ballot_id: ObjectId("ballot1".to_owned()),
                ballot_style: manifest.ballot_styles[0].object_id.clone(),
                contests: vec![],
                sn: Some(rand::thread_rng().gen_range(SERIAL_NUMBER_RANGE)),
                errors: None,
            };

            let encrypted_ballot = encrypt(
                &PathBuf::from(&classpath),
                &election_config.public_metadata_blob,
                &plaintext_ballot,
            )
            .unwrap();

            let encrypted_ballot: serde_json::Value =
                serde_json::from_slice(&encrypted_ballot).unwrap();

            assert_eq!(encrypted_ballot.get("ballot_id").unwrap(), "ballot1");
        } else {
            eprintln!("EG_CLASSPATH environment variable not set");
        }
    }
}
