use std::{
    fs::{DirBuilder, File},
    io::{self, Read, Write},
    path::PathBuf,
};

use crate::{
    command::run_electionguard_command,
    config::ElectionConfig,
    constants::{
        ENCRYPTED_BALLOTS_DIRECTORY, ENCRYPTED_BALLOT_PREFIX, ENCRYPTED_TALLY_FILE,
        PLAINTEXT_TALLY_FILE,
    },
    zip::{unzip_into_directory, UnzipLimits},
};

/// Accumulate the encrypted ballots into a single encrypted tally. Returns the
/// encrypted tally file as a byte vector, but its format should be UTF-8 JSON.
pub fn accumulate<'a>(
    classpath: &PathBuf,
    electionguard_metadata_config_blob: &[u8],
    encrypted_ballots: impl Iterator<Item = &'a [u8]>,
) -> io::Result<Vec<u8>> {
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    let input_directory = temp_dir_path.join("input");
    DirBuilder::new().create(&input_directory)?;

    let mut zip = zip::ZipArchive::new(io::Cursor::new(electionguard_metadata_config_blob))?;
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

    run_accumulate_tally(classpath, &input_directory, &output_directory)?;

    let mut encrypted_tally_bytes = Vec::new();
    let mut output_file = File::open(output_directory.join(ENCRYPTED_TALLY_FILE))?;
    output_file.read_to_end(&mut encrypted_tally_bytes)?;

    Ok(encrypted_tally_bytes)
}

/// Decrypt the encrypted tally into a plaintext tally. Returns the plaintext
/// tally file as a byte vector, but its format should be UTF-8 JSON.
pub fn decrypt(
    classpath: &PathBuf,
    election_config: &ElectionConfig,
    encrypted_tally: &[u8],
) -> io::Result<Vec<u8>> {
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    let input_directory = temp_dir_path.join("input");
    DirBuilder::new().create(&input_directory)?;

    let mut zip = zip::ZipArchive::new(io::Cursor::new(&election_config.public_metadata_blob))?;
    unzip_into_directory(&mut zip, &input_directory, UnzipLimits::default())?;

    let mut encrypted_tally_file = File::create(input_directory.join(ENCRYPTED_TALLY_FILE))?;
    encrypted_tally_file.write_all(encrypted_tally)?;
    encrypted_tally_file.sync_all()?;

    let trustees_directory = temp_dir_path.join("trustees");
    DirBuilder::new().create(&trustees_directory)?;

    let mut zip = zip::ZipArchive::new(io::Cursor::new(&election_config.private_metadata_blob))?;
    unzip_into_directory(&mut zip, &trustees_directory, UnzipLimits::default())?;

    let output_directory = temp_dir_path.join("output");
    DirBuilder::new().create(&output_directory)?;

    run_trusted_tally_decryption(
        classpath,
        &input_directory,
        &trustees_directory,
        &output_directory,
    )?;

    let mut plaintext_tally_bytes = Vec::new();
    let mut output_file = File::open(output_directory.join(PLAINTEXT_TALLY_FILE))?;
    output_file.read_to_end(&mut plaintext_tally_bytes)?;

    Ok(plaintext_tally_bytes)
}

/// Run the `RunAccumulateTally` command from the ElectionGuard CLI.
pub fn run_accumulate_tally(
    classpath: &PathBuf,
    configuration_directory: &PathBuf,
    output_directory: &PathBuf,
) -> io::Result<()> {
    run_electionguard_command(
        std::process::Command::new("java")
            .arg("-classpath")
            .arg(classpath)
            .arg("org.cryptobiotic.eg.cli.RunAccumulateTally")
            .arg("-in")
            .arg(configuration_directory)
            .arg("-out")
            .arg(output_directory),
    )
}

/// Run the `RunTrustedTallyDecryption` command from the ElectionGuard CLI.
fn run_trusted_tally_decryption(
    classpath: &PathBuf,
    input_directory: &PathBuf,
    trustees_directory: &PathBuf,
    output_directory: &PathBuf,
) -> io::Result<()> {
    run_electionguard_command(
        std::process::Command::new("java")
            .arg("-classpath")
            .arg(classpath)
            .arg("org.cryptobiotic.eg.cli.RunTrustedTallyDecryption")
            .arg("-in")
            .arg(input_directory)
            .arg("-trustees")
            .arg(trustees_directory)
            .arg("-out")
            .arg(output_directory),
    )
}
