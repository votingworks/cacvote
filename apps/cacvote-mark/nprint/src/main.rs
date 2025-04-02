mod nprint;

use std::{future::pending, path::PathBuf, time::Duration};

use anyhow::Context;
use futures_lite::FutureExt;
use libc::{c_int, c_uint};
use nprint::{CallbackMessage, Error, Length, Printer};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::broadcast,
    time::sleep,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomingMessage {
    reply_to: String,
    request: Request,
}

#[derive(Debug, Deserialize)]
#[serde(
    tag = "request",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum Request {
    Connect { printer: String },
    Disconnect,
    Init,
    LineFeed,
    Feed { length: Length },
    PrintImage { image_path: PathBuf },
    PrintLabel { label_path: PathBuf },
    Cut,
    Exit,
}

#[derive(Debug, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum OutgoingMessage {
    Response {
        in_reply_to: String,
        response: Response,
    },
    Event {
        event: Event,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "eventType", rename_all = "camelCase")]
enum Event {
    StatusChanged { change: StatusChangeEvent },
    Error { message: String },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusChangeEvent {
    old_status: Status,
    new_status: Status,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    cover_open: bool,
    no_paper: bool,
}

impl From<c_int> for Status {
    fn from(status: c_int) -> Self {
        Self {
            cover_open: status & 0x02 != 0,
            no_paper: status & 0x04 != 0,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "response", rename_all = "camelCase")]
enum Response {
    Ok,
    Error {
        message: String,
        cause: Option<nprint::Error>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut printer: Option<Printer> = None;
    let mut event_rx: Option<broadcast::Receiver<CallbackMessage>> = None;

    enum RunLoopEvent {
        Line(String),
        PrinterMessage(CallbackMessage),
        Error(String),
    }

    loop {
        let get_next_line = async {
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => break RunLoopEvent::Line(line),
                    Ok(None) => continue,
                    Err(e) => break RunLoopEvent::Error(e.to_string()),
                }
            }
        };
        let get_next_event = async {
            match &mut event_rx {
                Some(event_rx) => match event_rx.recv().await {
                    Ok(message) => RunLoopEvent::PrinterMessage(message),
                    Err(e) => RunLoopEvent::Error(e.to_string()),
                },
                None => pending().await,
            }
        };

        let event = get_next_event.or(get_next_line).await;

        match event {
            RunLoopEvent::Line(line) => {
                let IncomingMessage { request, reply_to } =
                    serde_json::from_str(&line).context("Parsing incoming message")?;

                let response = match request {
                    Request::Connect { printer: name } => {
                        let new_printer = Printer::open(name).await.context("Opening printer")?;
                        event_rx = Some(new_printer.watch_events());
                        printer = Some(new_printer);

                        Response::Ok
                    }
                    Request::Disconnect => {
                        printer = None;
                        Response::Ok
                    }
                    Request::Init => match &mut printer {
                        Some(printer) => printer.send_commands_raw(&[0x1b, 0x40]).map_or_else(
                            |e| Response::Error {
                                message: e.to_string(),
                                cause: Some(e),
                            },
                            |_| Response::Ok,
                        ),
                        None => Response::Error {
                            message: "No printer connected".to_owned(),
                            cause: None,
                        },
                    },
                    Request::PrintLabel { label_path } => match &mut printer {
                        Some(printer) => print_label(printer, label_path).await.map_or_else(
                            |e| Response::Error {
                                message: e.to_string(),
                                cause: Some(e),
                            },
                            |_| Response::Ok,
                        ),
                        None => Response::Error {
                            message: "Must connect before printing".to_owned(),
                            cause: None,
                        },
                    },
                    Request::LineFeed => match &mut printer {
                        Some(printer) => printer.line_feed().map_or_else(
                            |e| Response::Error {
                                message: e.to_string(),
                                cause: Some(e),
                            },
                            |_| Response::Ok,
                        ),
                        None => Response::Error {
                            message: "Must connect before line feeding".to_owned(),
                            cause: None,
                        },
                    },
                    Request::Feed { length } => match &mut printer {
                        Some(printer) => printer.feed(length).map_or_else(
                            |e| Response::Error {
                                message: e.to_string(),
                                cause: Some(e),
                            },
                            |_| Response::Ok,
                        ),
                        None => Response::Error {
                            message: "Must connect before feeding".to_owned(),
                            cause: None,
                        },
                    },
                    Request::PrintImage { image_path } => match &mut printer {
                        Some(printer) => printer.print_image_file(image_path).map_or_else(
                            |e| Response::Error {
                                message: e.to_string(),
                                cause: Some(e),
                            },
                            |_| Response::Ok,
                        ),
                        None => Response::Error {
                            message: "Must connect before printing".to_owned(),
                            cause: None,
                        },
                    },
                    Request::Cut => match &mut printer {
                        Some(printer) => printer.cut().map_or_else(
                            |e| Response::Error {
                                message: e.to_string(),
                                cause: Some(e),
                            },
                            |_| Response::Ok,
                        ),
                        None => Response::Error {
                            message: "Must connect before cutting".to_owned(),
                            cause: None,
                        },
                    },
                    Request::Exit => {
                        break;
                    }
                };

                println!(
                    "{}",
                    serde_json::to_string(&OutgoingMessage::Response {
                        in_reply_to: reply_to,
                        response
                    })?
                );
            }
            RunLoopEvent::PrinterMessage(message) => match message {
                CallbackMessage::StatusChanged(change) => {
                    println!(
                        "{}",
                        serde_json::to_string(&OutgoingMessage::Event {
                            event: Event::StatusChanged {
                                change: StatusChangeEvent {
                                    old_status: Status::from(change.old_status),
                                    new_status: Status::from(change.new_status),
                                }
                            }
                        })?
                    );
                }
                message => {
                    println!(
                        "{}",
                        serde_json::to_string(&OutgoingMessage::Event {
                            event: Event::Error {
                                message: format!("Unexpected printer message: {message:?}")
                            }
                        })?
                    );
                }
            },
            RunLoopEvent::Error(message) => {
                println!(
                    "{}",
                    serde_json::to_string(&OutgoingMessage::Event {
                        event: Event::Error { message }
                    })?
                );
            }
        }
    }

    Ok(())
}

async fn print_label(printer: &nprint::Printer, label_path: PathBuf) -> Result<(), nprint::Error> {
    let job_id: c_uint = 1;

    // The code below was mostly translated from the C++ sample code provided by
    // the manufacturer. I don't fully understand why the bit 7 check is
    // necessary or what it does.

    printer.status()?;

    let mut nms_bit7_check: c_int = 0;
    let mut flg_bit7_chk_loop = true;

    while nms_bit7_check < 2 && flg_bit7_chk_loop {
        nms_bit7_check += 1;
        let mut raw_gs_cmd = [0; 8];
        raw_gs_cmd[0] = 0x1d;
        raw_gs_cmd[1] = 0x47;

        match nms_bit7_check {
            0 => {
                // pattern1
                raw_gs_cmd[2] = 0x11;
            }
            1 => {
                // pattern2
                raw_gs_cmd[2] = 0x31;
            }
            _ => {}
        }

        raw_gs_cmd[3..][..4].copy_from_slice(&job_id.to_le_bytes());
        let _ = printer.send_commands_raw(&raw_gs_cmd);

        // Bit 7 confirmation of status
        let start = std::time::Instant::now();
        loop {
            let result = printer.status();
            match result {
                Err(_) => {
                    break;
                }
                Ok(status) if status.raw & 0x80 == 0x80 => {
                    flg_bit7_chk_loop = false;
                    break;
                }
                _ => {
                    if start.elapsed() > Duration::from_secs(5) {
                        match nms_bit7_check {
                            0 => eprintln!("1D 47 11 command error -> Try 1D 47 31"),
                            1 => eprintln!("Timeout: Bit7 did not turn ON"),
                            _ => {}
                        }
                        break;
                    }
                }
            }
        }
    }

    printer.start_doc()?;
    printer.print_image_file(label_path)?;
    printer.feed(Length::Dot(0xff))?;
    printer.cut()?;

    // Out printing command
    printer.send_commands_raw(&[0x1d, 0x47, if nms_bit7_check == 1 { 0x10 } else { 0x30 }])?;

    if let Err(e) = printer.end_doc() {
        let _ = printer.cancel_doc();
        return Err(e);
    }

    // Wait until printing is completed
    let mut nmu_get_job_id: c_uint = 0;
    let start = std::time::Instant::now();
    while nmu_get_job_id != job_id {
        if let Ok(job_id) = printer.check_for_job_finished_printing() {
            nmu_get_job_id = job_id;
        }

        if start.elapsed() > Duration::from_secs(10) {
            return Err(Error::Timeout);
        }

        sleep(Duration::from_millis(50)).await;
    }

    if nmu_get_job_id != job_id {
        return Err(Error::JobIdMismatch {
            expected: job_id,
            actual: nmu_get_job_id,
        });
    }

    Ok(())
}
