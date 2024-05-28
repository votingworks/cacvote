use std::{
    fs::{DirBuilder, File},
    io,
    path::PathBuf,
};

use crate::{
    command::run_electionguard_command,
    constants::MANIFEST_FILE,
    manifest::Manifest,
    zip::{zip_files_in_directory_to_buffer, ZipOptions},
};

pub struct ElectionConfig {
    pub public_metadata_blob: Vec<u8>,
    pub private_metadata_blob: Vec<u8>,
}

/// Generate ElectionGuard metadata for an election.
pub fn generate_election_config(
    classpath: &PathBuf,
    election: impl Into<Manifest>,
) -> io::Result<ElectionConfig> {
    let manifest: Manifest = election.into();

    // create a temporary working directory securely
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    // write the manifest to a file
    let manifest_path = temp_dir_path.join(MANIFEST_FILE);
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
    let public_metadata_blob =
        zip_files_in_directory_to_buffer(&output_directory, ZipOptions { recursion_depth: 0 })?;
    let private_metadata_blob =
        zip_files_in_directory_to_buffer(&trustees_directory, ZipOptions { recursion_depth: 0 })?;

    Ok(ElectionConfig {
        public_metadata_blob,
        private_metadata_blob,
    })
}

/// Run the Java ElectionGuard CLI to create an election configuration. Expects
/// to read and write files because the Java ElectionGuard implementation
/// expects to work with files.
pub fn run_create_election_config(
    classpath: &PathBuf,
    manifest_path: &PathBuf,
    output_directory: &PathBuf,
) -> io::Result<()> {
    run_electionguard_command(
        std::process::Command::new("java")
            .arg("-classpath")
            .arg(classpath)
            .arg("org.cryptobiotic.eg.cli.RunCreateElectionConfig")
            .arg("-manifest")
            .arg(manifest_path)
            .arg("-nguardians")
            .arg("1")
            .arg("-quorum")
            .arg("1")
            .arg("-out")
            .arg(output_directory)
            .arg("-group")
            .arg("P-256"),
    )
}

/// Run the Java ElectionGuard CLI to create a trustee (private) election key.
/// Expects to read and write files because the Java ElectionGuard
/// implementation expects to work with files.
pub fn run_trusted_key_ceremony(
    classpath: &PathBuf,
    input_directory: &PathBuf,
    trustees_directory: &PathBuf,
    output_directory: &PathBuf,
) -> io::Result<()> {
    run_electionguard_command(
        std::process::Command::new("java")
            .arg("-classpath")
            .arg(classpath)
            .arg("org.cryptobiotic.eg.cli.RunTrustedKeyCeremony")
            .arg("-in")
            .arg(input_directory)
            .arg("-trustees")
            .arg(trustees_directory)
            .arg("-out")
            .arg(output_directory),
    )
}

/// Extract the manifest from the public metadata blob.
pub fn extract_manifest_from_public_metadata_blob(
    public_metadata_blob: &[u8],
) -> io::Result<Manifest> {
    let mut manifest_zip = zip::ZipArchive::new(std::io::Cursor::new(public_metadata_blob))?;
    let manifest_file = manifest_zip.by_name(MANIFEST_FILE)?;
    let manifest: Manifest = serde_json::from_reader(manifest_file)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use types_rs::election::ElectionDefinition;

    use crate::constants::{CONSTANTS_FILE, ELECTION_CONFIG_FILE, ELECTION_INITIALIZED_FILE};

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
                    CONSTANTS_FILE,
                    ELECTION_CONFIG_FILE,
                    ELECTION_INITIALIZED_FILE,
                    MANIFEST_FILE,
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
