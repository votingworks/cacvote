use std::fmt::{self, Debug, Formatter};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::{path::PathBuf, process::Command};

use color_eyre::eyre::eyre;
use log::{debug, error, info};

/// Start a new scan session.
///
/// # Example
///
/// ```rust
/// use std::path::PathBuf;
/// use central_scanner::scan;
///
/// fn do_scan() -> Result<(), Box<dyn std::error::Error>> {
///     let session = scan(PathBuf::from("/tmp"))?;
///     for (side_a_path, side_b_path) in session {
///         println!("side_a_path: {:?}", side_a_path);
///         println!("side_b_path: {:?}", side_b_path);
///     }
///     Ok(())
/// }
/// ```
pub fn scan(directory: PathBuf) -> io::Result<ScanSession> {
    ScanSession::start(directory)
}

/// Internal actions that control the `scanimage` process.
#[derive(Debug)]
pub enum Action {
    /// Request a new card be scanned.
    Scan,

    /// Stop the scan session.
    Stop,
}

/// Represents a scanned card, i.e. a sheet of paper.
pub type Card = (PathBuf, PathBuf);

/// A scan session manages the `scanimage` process and handles requests to scan
/// cards.
pub struct ScanSession {
    /// The directory where the scanned images are stored. Must be writable by
    /// the current user.
    directory: PathBuf,

    /// The `scanimage` process.
    scanimage: std::process::Child,

    /// The channel used to send scan requests to the `scanimage` process.
    action_tx: SyncSender<Action>,

    /// The channel used to receive scanned cards from the `scanimage` process.
    card_rx: Receiver<(String, String)>,
}

impl Debug for ScanSession {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScanSession")
            .field("directory", &self.directory)
            .finish()
    }
}

/// Start the `scanimage` process in batch mode with all the expected options.
fn start_scanimage(batch_template: &str) -> io::Result<Child> {
    info!("starting scanimage with batch template: {}", batch_template);
    Command::new("scanimage")
        .arg("-d")
        .arg("fujitsu")
        .arg("--resolution")
        .arg("200")
        .arg("--format=jpeg")
        .arg("--source=ADF Duplex")
        .arg("--dropoutcolor")
        .arg("Red")
        .arg(format!("--batch={batch_template}"))
        .arg("--batch-print")
        .arg("--batch-prompt")
        .arg("--mode")
        .arg("gray")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

/// Loops over `lines` until it finds the line containing `<RETURN>`. This line
/// indicates that `scanimage` is ready to scan.
fn block_until_scanimage_ready<I>(lines: &mut I)
where
    I: Iterator<Item = io::Result<String>>,
{
    debug!("[scanimage] blocking until ready to scan");
    for line in lines {
        let line = line.unwrap();
        debug!("[scanimage] stderr: {}", line);

        if line.contains("<RETURN>") {
            debug!("[scanimage] ready");
            break;
        }
    }
}

/// Sends the lines from `lines` to the log as debug messages.
fn pipe_stderr_to_log<I>(lines: I)
where
    I: Iterator<Item = io::Result<String>> + Send + 'static,
{
    std::thread::spawn(move || {
        for line in lines {
            let line = line.unwrap();
            debug!("[scanimage] stderr: {}", line);
        }

        debug!("[scanimage] stderr closed");
    });
}

/// Establish a duplex channel between the `scanimage` process and the calling
/// thread. The calling thread will send `Action` values to the `scanimage`
/// process and receive scanned cards.
fn establish_batch_scanimage_duplex_channel(
    stdin: ChildStdin,
    stdout: ChildStdout,
) -> (SyncSender<Action>, Receiver<(String, String)>) {
    let (card_tx, card_rx) = mpsc::sync_channel::<(String, String)>(1);
    let (action_tx, action_rx) = mpsc::sync_channel::<Action>(1);

    std::thread::spawn(move || -> color_eyre::Result<()> {
        let mut stdin_writer = BufWriter::new(stdin);
        let mut first_line: Option<String> = None;
        let stdout_reader = BufReader::new(stdout);

        let mut wait_for_action = move || -> color_eyre::Result<()> {
            debug!("waiting for action");
            match action_rx.recv() {
                Ok(Action::Scan) => {
                    info!("scanning card");
                    debug!("stdin: <RETURN><RETURN> ('\\n\\n')");
                    stdin_writer
                        .write_all(b"\n\n")
                        .expect("unable to write to stdin");
                    stdin_writer.flush().expect("unable to flush stdin");
                    Ok(())
                }
                Ok(Action::Stop) => {
                    info!("stop received, exiting stdout process loop");
                    Err(eyre!("stop received"))
                }
                Err(err) => {
                    error!("error receiving action: {}", err);
                    Err(eyre!("error receiving action"))
                }
            }
        };

        // don't start reading stdout until we've received the first action.
        // we don't want to start scanning cards until we've received the first
        // request to scan
        wait_for_action()?;

        for line in stdout_reader.lines().flatten() {
            debug!("[scanimage] stdout: {}", line);
            if let Some(first_line) = first_line.take() {
                card_tx.send((first_line, line)).unwrap();
                // wait for the next scan action before reading the next card,
                // just in case the caller wants to stop the scan session early
                wait_for_action()?;
            } else {
                first_line = Some(line);
            }
        }

        if let Some(first_line) = first_line {
            error!(
                "[scanimage] unexpected end of stdout, dropping card: {}",
                first_line
            );
        }

        debug!("[scanimage] stdout closed");
        info!("scanimage session complete");
        Ok(())
    });

    (action_tx, card_rx)
}

impl ScanSession {
    /// Start a new scan session.
    fn start(directory: PathBuf) -> io::Result<ScanSession> {
        let batch_template = directory.join("scan-%04d.jpeg");
        let batch_template = batch_template
            .to_str()
            .expect("unable to convert path to string");
        let mut scanimage = start_scanimage(batch_template)?;

        let stdin = scanimage.stdin.take().unwrap();
        let stdout = scanimage.stdout.take().unwrap();
        let stderr = scanimage.stderr.take().unwrap();

        let stderr_reader = BufReader::new(stderr);
        let mut lines = stderr_reader.lines();

        block_until_scanimage_ready(lines.by_ref());
        pipe_stderr_to_log(lines);
        let (action_tx, card_rx) = establish_batch_scanimage_duplex_channel(stdin, stdout);

        Ok(ScanSession {
            directory,
            scanimage,
            action_tx,
            card_rx,
        })
    }

    /// Stop the scan session early, i.e. maybe before all pending cards have
    /// been scanned. This method does not need to be called if the scan session
    /// is iterated to completion.
    pub fn stop(&mut self) -> io::Result<()> {
        debug!("ScanSession::stop() called, sending Action::Stop");
        let _ = self.action_tx.send(Action::Stop);
        self.scanimage.kill()?;
        self.scanimage.wait()?;
        Ok(())
    }
}

impl Iterator for ScanSession {
    type Item = Card;

    /// Scan the next card. Blocks until the card has been scanned. If `None` is
    /// returned, the scan session has been stopped and `scanimage` has exited.
    /// If `Some` is returned, the card has been scanned and the tuple contains
    /// the paths to the front and back of the card. The paths will be files
    /// inside the directory specified when the scan session was started.
    fn next(&mut self) -> Option<Self::Item> {
        debug!("Iterator::next() called, sending Action::Scan");

        if let Err(err) = self.action_tx.send(Action::Scan) {
            debug!("action_tx.send() returned Err: {}", err);
            return None;
        }

        if let Ok((side_a_path, side_b_path)) = self.card_rx.recv() {
            debug!(
                "card_rx.recv() returned Ok: {:?}",
                (&side_a_path, &side_b_path)
            );
            let side_a_path = PathBuf::from(side_a_path);
            let side_b_path = PathBuf::from(side_b_path);
            assert!(side_a_path.starts_with(&self.directory));
            assert!(side_b_path.starts_with(&self.directory));
            Some((side_a_path, side_b_path))
        } else {
            debug!("card_rx.recv() returned Err, scan session stopped");
            None
        }
    }
}

impl Drop for ScanSession {
    fn drop(&mut self) {
        // make a best effort to stop the scan session
        let _ = self.stop();
    }
}
