#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(static_mut_refs)]

use std::{
    ffi::{CStr, CString},
    path::PathBuf,
    ptr::null_mut,
    sync::{Arc, Mutex, Once},
};

use libc::{c_char, c_int, c_uint, c_ulong, c_void};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

static mut GLOBAL_CALLBACK_HANDLER: Option<Arc<Mutex<CallbackHandler>>> = None;
static INIT: Once = Once::new();

#[derive(Debug, Clone)]
pub struct StatusChange {
    pub printer_name: String,
    pub old_status: c_int,
    pub new_status: c_int,
}

#[derive(Debug, Clone)]
pub enum CallbackMessage {
    StatusChanged(StatusChange),
    ExtendedInfo {
        printer_name: String,
        extended_info_id: c_int,
    },
    TransmissionQueueEmpty {
        printer_name: String,
    },
    OpenResult {
        printer_name: String,
        result: c_int,
    },
}

struct CallbackHandler {
    tx: broadcast::Sender<CallbackMessage>,
}

impl CallbackHandler {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        Self { tx }
    }

    fn subscribe(&mut self) -> broadcast::Receiver<CallbackMessage> {
        self.tx.subscribe()
    }

    fn handle(
        &mut self,
        printer_name: *mut c_char,
        callback_type: c_int,
        value1: c_int,
        value2: c_int,
    ) {
        let printer_name = unsafe { CStr::from_ptr(printer_name) }
            .to_string_lossy()
            .into_owned();
        let message = match (callback_type, value1, value2) {
            (1, _, _) => CallbackMessage::StatusChanged(StatusChange {
                printer_name,
                old_status: value1,
                new_status: value2,
            }),
            (2, _, 0x00) => CallbackMessage::ExtendedInfo {
                printer_name,
                extended_info_id: value1,
            },
            (3, 0x00, 0x00) => CallbackMessage::TransmissionQueueEmpty { printer_name },
            (5, _, 0x00) => CallbackMessage::OpenResult {
                printer_name,
                result: value1,
            },
            _ => {
                eprintln!(
                    "Unknown callback: printer_name={printer_name}, callback_type={callback_type}, value1={value1}, value2={value2}"
                );
                return;
            }
        };
        let _ = self.tx.send(message.clone());
    }
}

type NPrintCallback = extern "C" fn(*mut c_char, c_int, c_int, c_int);
extern "C" fn nprint_callback_handler(
    printer_name: *mut c_char,
    callback_type: c_int,
    value1: c_int,
    value2: c_int,
) {
    if let Some(handler) = unsafe { &GLOBAL_CALLBACK_HANDLER } {
        let mut handler = handler.lock().unwrap();
        handler.handle(printer_name, callback_type, value1, value2);
    }
}

#[link(name = "NPrint")]
extern "C" {
    fn NEnumPrinters(o_printers: *mut c_char, o_size: *mut c_uint) -> c_int;
    fn NOpenPrinter(i_prt: *mut c_char, i_statusFlg: u8, i_callback: NPrintCallback) -> c_int;
    fn NClosePrinter(i_prt: *mut c_char) -> c_int;
    fn NPrint(
        i_prt: *mut c_char,
        i_dat: *mut c_char,
        i_size: c_uint,
        o_jobid: *mut c_uint,
    ) -> c_int;
    fn NDPrint(i_prt: *mut c_char, i_dat: *mut u8, i_size: c_uint, o_jobid: *mut c_uint) -> c_int;
    fn NImagePrint(
        i_prt: *mut c_char,
        i_bmp: *mut u8,
        i_width: c_uint,
        i_height: c_uint,
        i_channels: c_uint,
        i_step: c_uint,
        i_putType: u8,
        o_jobid: *mut c_uint,
    ) -> c_int;
    fn NImagePrintF(
        i_prt: *mut c_char,
        i_bmp: *mut c_char,
        i_putType: u8,
        o_jobid: *mut c_uint,
    ) -> c_int;
    fn NGetStatus(i_prt: *mut c_char, o_status: *mut c_ulong) -> c_int;
    fn NGetInformation(
        i_prt: *mut c_char,
        i_id: u8,
        o_dat: *mut c_void,
        o_time: *mut c_ulong,
    ) -> c_int;
    fn NStartDoc(i_prt: *mut c_char, o_jobid: *mut c_uint) -> c_int;
    fn NEndDoc(i_prt: *mut c_char) -> c_int;
    fn NCancelDoc(i_prt: *mut c_char) -> c_int;
}

#[derive(thiserror::Error, Debug, Serialize)]
pub enum Error {
    #[error("Handle error (-1)")]
    Handle,

    #[error("Printer open error (-2)")]
    PrinterOpen,

    #[error("Printer is offline (-5)")]
    Offline,

    #[error("Printer is unexplained offline (-6)")]
    UnexplainedOffline,

    #[error("File open error (-10)")]
    FileOpen,

    #[error("Printer output error (-13)")]
    PrinterOutput,

    #[error("File write error (-15)")]
    FileWrite,

    #[error("File read error (-16)")]
    FileRead,

    #[error("Make directory error (-18)")]
    MakeDirectory,

    #[error("Printer is already opened (-21)")]
    AlreadyOpened,

    #[error("Printer information does not exist (-22)")]
    NoHandle,

    #[error("Lack of resources (-31)")]
    LackOfResources,

    #[error("Failed to load image file (-50)")]
    LoadImageFile,

    #[error("Resetting failure (-60)")]
    ResettingFailure,

    #[error("StartDoc failure (-70)")]
    StartDocFailure,

    #[error("Not in the document start state (-71)")]
    DocNotStarted,

    #[error("Already in the document start state (-72)")]
    DocAlreadyStarted,

    #[error("EndDoc failure (-73)")]
    EndDocFailure,

    #[error("FWT file error (-80)")]
    FwfFile,

    #[error("The checksum entered in the argument does not match the checksum obtained by the printer (-81)")]
    FwfChecksum,

    #[error("Timeout checking the firmware checksum (-83)")]
    FwfChecksumTimeout,

    #[error("Error checking status during firmware download (-84)")]
    FwfCheckFailure,

    #[error("Argument error (-90)")]
    Argument,

    #[error("1st argument error (-91)")]
    Argument1,

    #[error("2nd argument error (-92)")]
    Argument2,

    #[error("3rd argument error (-93)")]
    Argument3,

    #[error("4th argument error (-94)")]
    Argument4,

    #[error("5th argument error (-95)")]
    Argument5,

    #[error("6th argument error (-96)")]
    Argument6,

    #[error("7th argument error (-97)")]
    Argument7,

    #[error("8th argument error (-98)")]
    Argument8,

    #[error("9th argument error (-99)")]
    Argument9,

    #[error("Printer information cannot be obtained (-136)")]
    PrinterInfoGet,

    #[error("Printer information limit exceeded (-138)")]
    PrinterInfoLimit,

    #[error("Connection type not supported (-150)")]
    DeviceNotSupported,

    #[error("Command line error (-170)")]
    CommandLine,

    #[error("Search error (-180)")]
    Search,

    #[error("Invalid UTF-8")]
    FromUtf8Failure(
        #[serde(skip)]
        #[from]
        std::string::FromUtf8Error,
    ),

    #[error("Null error")]
    NullFailure(
        #[serde(skip)]
        #[from]
        std::ffi::NulError,
    ),

    #[error("Length out of range: {0}")]
    LengthOutOfRange(
        #[serde(skip)]
        #[from]
        RangeError<f32, u8>,
    ),

    #[error("IO error: {0}")]
    Io(
        #[serde(skip)]
        #[from]
        std::io::Error,
    ),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Timeout error")]
    Timeout,

    #[error("Job ID mismatch")]
    JobIdMismatch { expected: u32, actual: u32 },

    #[error("Unknown error code: {0}")]
    Unknown(i32),
}

impl From<i32> for Error {
    fn from(value: i32) -> Self {
        match value {
            -1 => Error::Handle,
            -2 => Error::PrinterOpen,
            -5 => Error::Offline,
            -6 => Error::UnexplainedOffline,
            -10 => Error::FileOpen,
            -13 => Error::PrinterOutput,
            -15 => Error::FileWrite,
            -16 => Error::FileRead,
            -18 => Error::MakeDirectory,
            -21 => Error::AlreadyOpened,
            -22 => Error::NoHandle,
            -31 => Error::LackOfResources,
            -50 => Error::LoadImageFile,
            -60 => Error::ResettingFailure,
            -70 => Error::StartDocFailure,
            -71 => Error::DocNotStarted,
            -72 => Error::DocAlreadyStarted,
            -73 => Error::EndDocFailure,
            -80 => Error::FwfFile,
            -81 => Error::FwfChecksum,
            -83 => Error::FwfChecksumTimeout,
            -84 => Error::FwfCheckFailure,
            -90 => Error::Argument,
            -91 => Error::Argument1,
            -92 => Error::Argument2,
            -93 => Error::Argument3,
            -94 => Error::Argument4,
            -95 => Error::Argument5,
            -96 => Error::Argument6,
            -97 => Error::Argument7,
            -98 => Error::Argument8,
            -99 => Error::Argument9,
            -136 => Error::PrinterInfoGet,
            -138 => Error::PrinterInfoLimit,
            -150 => Error::DeviceNotSupported,
            -170 => Error::CommandLine,
            -180 => Error::Search,
            _ => Error::Unknown(value),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RangeError<F: std::fmt::Display, T: std::fmt::Display> {
    #[error("Value out of range: {0} (expected {1} ≤ x ≤ {2})")]
    Value(F, T, T),
}

/// Convenience macro for running NPrint API functions and returning an error if the return value is less than 0.
macro_rules! np {
    ($func:ident($($arg:expr),*)) => {
        {
            let ret = unsafe { $func($($arg),*) };
            if ret < 0 {
                return Err(Error::from(ret));
            }
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", content = "value")]
pub enum Length {
    /// A length in dots, or 1/203 of an inch.
    Dot(u8),

    /// A length in inches.
    Inch(f32),

    /// A length in millimeters.
    Millimeter(f32),
}

impl Length {
    pub fn to_dot(&self) -> Result<u8, RangeError<f32, u8>> {
        match self {
            Length::Dot(d) => Ok(*d),
            Length::Inch(i) => {
                if i < &0.0 {
                    return Err(RangeError::Value(*i, 0, u8::MAX));
                }

                if i * 203.0 > u8::MAX as f32 {
                    return Err(RangeError::Value(i * 203.0, 0, u8::MAX));
                }

                Ok((i * 203.0) as u8)
            }
            Length::Millimeter(m) => {
                if m < &0.0 {
                    return Err(RangeError::Value(*m, 0, u8::MAX));
                }

                if m / 25.4 * 203.0 > u8::MAX as f32 {
                    return Err(RangeError::Value((m / 25.4) * 203.0, 0, u8::MAX));
                }

                Ok((m / 25.4 * 203.0) as u8)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InformationType {
    ExtendedStatus = 1,
    ModelName = 2,
    FirmwareVersion = 3,
    BootVersion = 4,
    // Reserved = 5,
    NumberOfDotLinesEnergizingHead = 6,
    NumberOfFedDotLines = 7,
    NumberOfCuts = 8,
    UserMainenanceCounter = 9,
    // Reserved = 10,
    // Reserved = 11,
    // Reserved = 12,
    NvRegistrationStatus = 13,
    // Reserved = 14,
    // Reserved = 15,
    // Reserved = 16,
    // Reserved = 17,
    // Reserved = 18,
    EndOfPrintNotification = 19,
    // Reserved = 20,
    // Reserved = 21,
    // Reserved = 22,
    // Reserved = 23,
    // Reserved = 24,
    TransferCompletionNotification = 25,
    // Reserved = 26,
    // Reserved = 27,
    FirmwareChecksum = 28,
    // Reserved = 29,
    // Reserved = 30,
    CommunicationStatus = 31,
}

const IMG_RASTER_LINE: u8 = 0x00;
const IMG_RASTER_BLOCK: u8 = 0x01;
const IMG_RASTER_GRADATION: u8 = 0x02;
const IMG_BITIMG: u8 = 0x10;

#[derive(Debug, Clone)]
pub struct Printer {
    name: CString,
}

impl Printer {
    pub async fn open(name: String) -> Result<Self, Error> {
        INIT.call_once(|| {
            let handler = CallbackHandler::new();
            unsafe {
                GLOBAL_CALLBACK_HANDLER = Some(Arc::new(Mutex::new(handler)));
            }
        });

        let handler = unsafe {
            GLOBAL_CALLBACK_HANDLER
                .as_ref()
                .expect("GLOBAL_CALLBACK_HANDLER was initialized in the Once::call_once block")
                .clone()
        };
        let mut rx = handler.lock().unwrap().subscribe();

        let name = CString::new(name)?;
        np!(NOpenPrinter(
            name.as_ptr() as *mut c_char,
            1,
            nprint_callback_handler
        ));

        while let Ok(message) = rx.recv().await {
            if let CallbackMessage::OpenResult {
                printer_name,
                result,
            } = message
            {
                if printer_name == name.to_string_lossy() {
                    if result < 0 {
                        return Err(Error::from(result));
                    }
                    break;
                }
            }
        }

        Ok(Self { name })
    }

    pub fn watch_events(&self) -> broadcast::Receiver<CallbackMessage> {
        let handler = unsafe {
            GLOBAL_CALLBACK_HANDLER
                .as_ref()
                .expect("GLOBAL_CALLBACK_HANDLER was initialized in the Once::call_once block")
                .clone()
        };
        let mut guard = handler.lock().unwrap();
        guard.subscribe()
    }

    pub fn close(self) -> Result<(), Error> {
        self.close_without_consuming()
    }

    fn close_without_consuming(&self) -> Result<(), Error> {
        np!(NClosePrinter(self.name.as_ptr() as *mut c_char));
        Ok(())
    }

    /// Sends command strings to the printer. These strings are interpreted by
    /// libNPrint according to the API reference for the `NPrint` function.
    ///
    /// # Example
    ///
    /// ```
    /// let printer = Printer::named("PRT001".to_string());
    /// printer.open().unwrap();
    /// // "Hello, world!": print "Hello, world!"
    /// // 0a: send a line feed
    /// // 1b4aff: print and feed by 0xff/203 of an inch (255/203 inch, 31.9 mm)
    /// // 1b69: cut the paper
    /// printer.send_commands("\"Hello, world!\"0a1b4aff1b69").unwrap();
    /// printer.close().unwrap();
    /// ```
    ///
    /// # Command Processing
    ///
    /// By default, the string is composed of hexadecimal characters that are
    /// converted to bytes and sent to the printer. The string is processed as
    /// follows:
    ///
    /// - A string enclosed in double quotes will be sent to the printer as is.
    /// - A string enclosed in angle brackets will be treated as a file
    ///   path that should be read and sent to the printer without further
    ///   processing.
    /// - A string enclosed in square brackets will be treated as a
    ///   file path to an image file that should be printed using raster block
    ///   mode.
    ///
    /// If you don't want this processing to happen, you can use the raw byte
    /// sending function [`send_commands_raw`] instead.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be converted to a C string or if
    /// the `NPrint` function returns an error code.
    pub fn send_commands(&self, data: &str) -> Result<(), Error> {
        let cdata = CString::new(data)?;
        let cdata_size = cdata.as_bytes().len();
        np!(NPrint(
            self.name.as_ptr() as *mut c_char,
            cdata.as_ptr() as *mut c_char,
            cdata_size as c_uint,
            null_mut()
        ));
        Ok(())
    }

    /// Sends raw bytes to the printer.
    pub fn send_commands_raw(&self, data: &[u8]) -> Result<(), Error> {
        let cdata_size = data.len();
        np!(NDPrint(
            self.name.as_ptr() as *mut c_char,
            data.as_ptr() as *mut u8,
            cdata_size as c_uint,
            null_mut()
        ));
        Ok(())
    }

    pub fn get_model_name(&self) -> Result<String, Error> {
        let mut buffer = [0; 128];
        self.get_information(InformationType::ModelName, &mut buffer)?;
        let c_model_name = unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) };
        Ok(c_model_name.to_string_lossy().into_owned())
    }

    pub fn get_firmware_version(&self) -> Result<String, Error> {
        let mut buffer = [0; 128];
        self.get_information(InformationType::FirmwareVersion, &mut buffer)?;
        let c_firmware_version = unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) };
        Ok(c_firmware_version.to_string_lossy().into_owned())
    }

    pub fn get_boot_version(&self) -> Result<String, Error> {
        let mut buffer = [0; 128];
        self.get_information(InformationType::BootVersion, &mut buffer)?;
        let c_boot_version = unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) };
        Ok(c_boot_version.to_string_lossy().into_owned())
    }

    pub fn check_for_job_finished_printing(&self) -> Result<c_uint, Error> {
        let mut buffer = [0; 128];
        self.get_information(InformationType::EndOfPrintNotification, &mut buffer)?;
        let job_id = buffer[..std::mem::size_of::<c_uint>()]
            .try_into()
            .expect("Slice is exactly 4 bytes");
        Ok(u32::from_le_bytes(job_id))
    }

    fn get_information(&self, info_type: InformationType, buffer: &mut [u8]) -> Result<(), Error> {
        np!(NGetInformation(
            self.name.as_ptr() as *mut c_char,
            info_type as u8,
            buffer.as_mut_ptr() as *mut c_void,
            null_mut()
        ));
        Ok(())
    }

    pub fn start_doc(&self) -> Result<c_uint, Error> {
        let mut job_id: c_uint = 0;
        np!(NStartDoc(self.name.as_ptr() as *mut c_char, &mut job_id));
        Ok(job_id)
    }

    pub fn end_doc(&self) -> Result<(), Error> {
        np!(NEndDoc(self.name.as_ptr() as *mut c_char));
        Ok(())
    }

    pub fn cancel_doc(&self) -> Result<(), Error> {
        np!(NCancelDoc(self.name.as_ptr() as *mut c_char));
        Ok(())
    }

    /// Prints an image file to the printer.
    pub fn print_image_file(&self, image_path: impl Into<PathBuf>) -> Result<(), Error> {
        let image_path = image_path.into().into_os_string();
        let image_path = std::ffi::CString::new(image_path.as_encoded_bytes())?;
        np!(NImagePrintF(
            self.name.as_ptr() as *mut c_char,
            image_path.as_ptr() as *mut c_char,
            IMG_RASTER_BLOCK,
            null_mut()
        ));
        Ok(())
    }

    /// Causes the printer to feed the paper by the specified length.
    pub fn feed(&self, length: Length) -> Result<(), Error> {
        self.send_commands_raw(&[0x1b, 0x4a, length.to_dot()?])
    }

    /// Causes the printer to cut the paper.
    pub fn cut(&self) -> Result<(), Error> {
        self.send_commands_raw(&[0x1b, 0x69])
    }

    /// Causes the printer to print out the data in the buffer and feed the
    /// paper by one line.
    pub fn line_feed(&self) -> Result<(), Error> {
        self.send_commands_raw(&[0x0a])
    }

    /// TODO: Interpret the results of the `NGetStatus` function and return a
    /// `PrinterStatus` struct with the relevant information.
    pub fn status(&self) -> Result<PrinterStatus, Error> {
        let mut status: c_ulong = 0;
        np!(NGetStatus(self.name.as_ptr() as *mut c_char, &mut status));
        Ok(PrinterStatus { raw: status })
    }
}

impl Drop for Printer {
    fn drop(&mut self) {
        let _ = self.close_without_consuming();
    }
}

pub fn enum_printers() -> Result<Vec<String>, Error> {
    let mut size: c_uint = 0;
    np!(NEnumPrinters(null_mut(), &mut size));

    if size == 0 {
        return Ok(Vec::new());
    }

    let mut buf = vec![0u8; (size + 1) as usize];
    np!(NEnumPrinters(buf.as_mut_ptr() as *mut c_char, &mut size));

    Ok(String::from_utf8(buf)?
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect())
}

#[derive(Debug)]
pub struct PrinterStatus {
    pub raw: c_ulong,
}
