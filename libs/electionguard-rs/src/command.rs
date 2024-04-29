use std::{io, process::Command};

#[tracing::instrument]
pub fn run_electionguard_command(command: &mut Command) -> io::Result<()> {
    command
        .output()
        .and_then(|output| {
            if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                if !stdout.is_empty() {
                    tracing::debug!("stdout: {stdout}");

                }
            }

            if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
                if !stderr.is_empty() {
                    tracing::debug!("stderr: {stderr}");
                }
            }

            if !output.status.success() {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Java ElectionGuard CLI exited with code {status}\nstdout: {stdout}\nstderr: {stderr}",
                        status = output.status.code().unwrap_or_default(),
                        stdout = std::str::from_utf8(&output.stdout).unwrap_or_default(),
                        stderr = std::str::from_utf8(&output.stderr).unwrap_or_default(),
                    ),
                ))
            } else {
                Ok(())
            }
        })
}
