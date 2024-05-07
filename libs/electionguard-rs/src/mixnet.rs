use std::{
    fs::{DirBuilder, File},
    io::{self, Write},
    num::NonZeroUsize,
    path::PathBuf,
};

use crate::{
    command::run_electionguard_command,
    constants::{ENCRYPTED_BALLOTS_DIRECTORY, ENCRYPTED_BALLOT_PREFIX},
    zip::{unzip_into_directory, zip_files_in_directory_to_buffer, UnzipLimits, ZipOptions},
};

/// Shuffles the ballots using `phases` phases and returns a buffer containing
/// the ZIP archive of the mixnet output. The ZIP will contain one directory for
/// each phase, each containing three files: `mix_config.json`,
/// `proof_of_shuffle.json`, and `ShuffledBallots.json`. The mix directories are
/// named `mix1`, `mix2`, etc.
pub fn mix<'a>(
    classpath: &PathBuf,
    public_metadata_blob: &[u8],
    encrypted_ballots: impl Iterator<Item = &'a [u8]>,
    phases: NonZeroUsize,
) -> io::Result<Vec<u8>> {
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    let input_directory = temp_dir_path.join("input");
    DirBuilder::new().create(&input_directory)?;

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(public_metadata_blob))?;
    unzip_into_directory(&mut zip, &input_directory, UnzipLimits::default())?;

    let encrypted_ballots_directory = input_directory.join(ENCRYPTED_BALLOTS_DIRECTORY);
    DirBuilder::new().create(&encrypted_ballots_directory)?;

    for (index, encrypted_ballot) in encrypted_ballots.enumerate() {
        let encrypted_ballot_path =
            encrypted_ballots_directory.join(format!("{ENCRYPTED_BALLOT_PREFIX}{index}.json"));
        let mut encrypted_ballot_file = File::create(&encrypted_ballot_path)?;
        encrypted_ballot_file.write_all(encrypted_ballot)?;
        encrypted_ballot_file.sync_all()?;
    }

    let output_directory = temp_dir_path.join("output");
    DirBuilder::new().create(&output_directory)?;

    let mut input_mix_directory = None;

    for phase in 1..=phases.get() {
        let mix_name = format!("mix{phase}");
        let mix_directory = output_directory.join(&mix_name);
        DirBuilder::new().create(&mix_directory)?;
        run_mixnet(
            classpath,
            &input_directory,
            &output_directory,
            &mix_name,
            input_mix_directory.as_ref(),
        )?;

        input_mix_directory = Some(mix_directory);
    }

    zip_files_in_directory_to_buffer(&output_directory, ZipOptions { recursion_depth: 1 })
}

/// Run the mixnet command to shuffle the encrypted ballots.
///
/// # Arguments
///
/// * `classpath` - The classpath to the ElectionGuard CLI JAR file.
/// * `public_directory` - The directory containing the public election data,
///   in particular the encrypted ballots.
/// * `output_directory` - The directory to create the mix directory within. The
///   mix directory will contain three files: `mix_config.json`,
///   `proof_of_shuffle.json`, and `ShuffledBallots.json`.
/// * `mix_name` - The name of the directory inside `output_directory` to write
///   output to.
/// * `input_mix_directory` - The directory containing the input mix data. If
///   `None`, the this is the first mix and the input is the public directory.
///
/// # Errors
///
/// Returns an error if the mixnet command fails.
pub fn run_mixnet(
    classpath: &PathBuf,
    public_directory: &PathBuf,
    output_directory: &PathBuf,
    mix_name: &str,
    input_mix_directory: Option<&PathBuf>,
) -> io::Result<()> {
    let mut command = std::process::Command::new("java");

    command
        .arg("-classpath")
        .arg(classpath)
        .arg("org.cryptobiotic.mixnet.cli.RunMixnet")
        .arg("-publicDir")
        .arg(public_directory)
        .arg("-out")
        .arg(output_directory)
        .arg("--mixName")
        .arg(mix_name);

    if let Some(input_mix_directory) = input_mix_directory {
        command.arg("-in").arg(input_mix_directory);
    }

    run_electionguard_command(&mut command)
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use types_rs::election::ElectionDefinition;

    use super::*;
    use std::{iter::once, num::NonZeroUsize};

    use crate::{
        ballot::{encrypt, PlaintextBallot, SERIAL_NUMBER_RANGE},
        config::generate_election_config,
        manifest::{Manifest, ObjectId},
    };

    fn load_election_definition() -> color_eyre::Result<ElectionDefinition> {
        Ok(ElectionDefinition::try_from(
            &include_bytes!("../tests/fixtures/electionFamousNames2021.json")[..],
        )?)
    }

    #[test]
    fn test_mix1() {
        let Ok(classpath) = std::env::var("EG_CLASSPATH") else {
            eprintln!("EG_CLASSPATH environment variable not set");
            return;
        };
        let classpath = PathBuf::from(classpath);

        let election_definition = load_election_definition().unwrap();
        let manifest: Manifest = election_definition.election.clone().into();
        let election_config =
            generate_election_config(&classpath, election_definition.election.clone()).unwrap();

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
            "device1",
        )
        .unwrap();
        let mix = mix(
            &classpath,
            &election_config.public_metadata_blob,
            once(encrypted_ballot.as_slice()),
            NonZeroUsize::new(1).unwrap(),
        )
        .unwrap();

        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(mix)).unwrap();
        assert_eq!(zip.len(), 3);

        let mix_config_file = zip.by_name("mix1/mix_config.json").unwrap();
        let mix_config: serde_json::Value = serde_json::from_reader(mix_config_file).unwrap();
        assert_eq!(mix_config["mix_name"], "mix1");

        let proof_of_shuffle_file = zip.by_name("mix1/proof_of_shuffle.json").unwrap();
        let proof_of_shuffle: serde_json::Value =
            serde_json::from_reader(proof_of_shuffle_file).unwrap();
        assert_eq!(proof_of_shuffle["mixname"], "mix1");

        let shuffled_ballots_file = zip.by_name("mix1/ShuffledBallots.json").unwrap();
        let shuffled_ballots: serde_json::Value =
            serde_json::from_reader(shuffled_ballots_file).unwrap();
        assert_eq!(shuffled_ballots["rows"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_mix2() {
        let Ok(classpath) = std::env::var("EG_CLASSPATH") else {
            eprintln!("EG_CLASSPATH environment variable not set");
            return;
        };
        let classpath = PathBuf::from(classpath);

        let election_definition = load_election_definition().unwrap();
        let manifest: Manifest = election_definition.election.clone().into();
        let election_config =
            generate_election_config(&classpath, election_definition.election.clone()).unwrap();

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
            "device1",
        )
        .unwrap();
        let mix = mix(
            &classpath,
            &election_config.public_metadata_blob,
            once(encrypted_ballot.as_slice()),
            NonZeroUsize::new(2).unwrap(),
        )
        .unwrap();

        let zip = zip::ZipArchive::new(std::io::Cursor::new(mix)).unwrap();
        assert_eq!(zip.len(), 6);

        let mut file_names = zip.file_names().collect::<Vec<_>>();
        file_names.sort();
        assert_eq!(
            file_names,
            vec![
                "mix1/ShuffledBallots.json",
                "mix1/mix_config.json",
                "mix1/proof_of_shuffle.json",
                "mix2/ShuffledBallots.json",
                "mix2/mix_config.json",
                "mix2/proof_of_shuffle.json",
            ]
        );
    }
}
