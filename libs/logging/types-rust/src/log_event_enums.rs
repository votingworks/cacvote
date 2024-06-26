//! Do not edit this file directly! It's generated and manual changes will be overwritten.
//! To add a log event, edit `log_event_details.toml` and run `pnpm build:generate-types`.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum EventId {
    // A value for use in a Log's Default trait
    #[serde(rename = "unspecified")]
    Unspecified,
    // Generated enum values
    #[serde(rename = "election-configured")]
    ElectionConfigured,
    #[serde(rename = "election-unconfigured")]
    ElectionUnconfigured,
    #[serde(rename = "machine-boot-init")]
    MachineBootInit,
    #[serde(rename = "machine-boot-complete")]
    MachineBootComplete,
    #[serde(rename = "machine-shutdown-init")]
    MachineShutdownInit,
    #[serde(rename = "machine-shutdown-complete")]
    MachineShutdownComplete,
    #[serde(rename = "usb-device-change-detected")]
    UsbDeviceChangeDetected,
    #[serde(rename = "process-started")]
    ProcessStarted,
    #[serde(rename = "process-terminated")]
    ProcessTerminated,
    #[serde(rename = "auth-pin-entry")]
    AuthPinEntry,
    #[serde(rename = "auth-login")]
    AuthLogin,
    #[serde(rename = "auth-voter-session-updated")]
    AuthVoterSessionUpdated,
    #[serde(rename = "auth-logout")]
    AuthLogout,
    #[serde(rename = "usb-drive-detected")]
    UsbDriveDetected,
    #[serde(rename = "usb-drive-removed")]
    UsbDriveRemoved,
    #[serde(rename = "usb-drive-eject-init")]
    UsbDriveEjectInit,
    #[serde(rename = "usb-drive-eject-complete")]
    UsbDriveEjected,
    #[serde(rename = "usb-drive-mount-init")]
    UsbDriveMountInit,
    #[serde(rename = "usb-drive-mount-complete")]
    UsbDriveMounted,
    #[serde(rename = "usb-drive-format-init")]
    UsbDriveFormatInit,
    #[serde(rename = "usb-drive-format-complete")]
    UsbDriveFormatted,
    #[serde(rename = "application-startup")]
    ApplicationStartup,
    #[serde(rename = "printer-config-added")]
    PrinterConfigurationAdded,
    #[serde(rename = "printer-config-removed")]
    PrinterConfigurationRemoved,
    #[serde(rename = "printer-connection-update")]
    PrinterConnectionUpdate,
    #[serde(rename = "device-attached")]
    DeviceAttached,
    #[serde(rename = "device-unattached")]
    DeviceUnattached,
    #[serde(rename = "file-saved")]
    FileSaved,
    #[serde(rename = "save-election-package-init")]
    SaveElectionPackageInit,
    #[serde(rename = "save-election-package-complete")]
    SaveElectionPackageComplete,
    #[serde(rename = "smart-card-program-init")]
    SmartCardProgramInit,
    #[serde(rename = "smart-card-program-complete")]
    SmartCardProgramComplete,
    #[serde(rename = "smart-card-unprogram-init")]
    SmartCardUnprogramInit,
    #[serde(rename = "smart-card-unprogram-complete")]
    SmartCardUnprogramComplete,
    #[serde(rename = "list-cast-vote-record-exports-on-usb-drive")]
    ListCastVoteRecordExportsOnUsbDrive,
    #[serde(rename = "import-cast-vote-records-init")]
    ImportCastVoteRecordsInit,
    #[serde(rename = "import-cast-vote-records-complete")]
    ImportCastVoteRecordsComplete,
    #[serde(rename = "clear-imported-cast-vote-records-init")]
    ClearImportedCastVoteRecordsInit,
    #[serde(rename = "clear-imported-cast-vote-records-complete")]
    ClearImportedCastVoteRecordsComplete,
    #[serde(rename = "manual-tally-data-edited")]
    ManualTallyDataEdited,
    #[serde(rename = "manual-tally-data-removed")]
    ManualTallyDataRemoved,
    #[serde(rename = "marked-tally-results-official")]
    MarkedTallyResultsOfficial,
    #[serde(rename = "election-report-previewed")]
    ElectionReportPreviewed,
    #[serde(rename = "election-report-printed")]
    ElectionReportPrinted,
    #[serde(rename = "converting-to-sems")]
    ConvertingResultsToSemsFormat,
    #[serde(rename = "initial-election-package-loaded")]
    InitialElectionPackageLoaded,
    #[serde(rename = "system-settings-save-initiated")]
    SystemSettingsSaveInitiated,
    #[serde(rename = "system-settings-saved")]
    SystemSettingsSaved,
    #[serde(rename = "system-settings-retrieved")]
    SystemSettingsRetrieved,
    #[serde(rename = "write-in-adjudicated")]
    WriteInAdjudicated,
    #[serde(rename = "toggle-test-mode-init")]
    TogglingTestMode,
    #[serde(rename = "toggled-test-mode")]
    ToggledTestMode,
    #[serde(rename = "clear-ballot-data-init")]
    ClearingBallotData,
    #[serde(rename = "clear-ballot-data-complete")]
    ClearedBallotData,
    #[serde(rename = "override-mark-threshold-init")]
    OverridingMarkThresholds,
    #[serde(rename = "override-mark-thresholds-complete")]
    OverrodeMarkThresholds,
    #[serde(rename = "saved-scan-image-backup")]
    SavedScanImageBackup,
    #[serde(rename = "configure-from-election-package-init")]
    ConfigureFromElectionPackageInit,
    #[serde(rename = "election-package-files-read-from-usb")]
    ElectionPackageFilesReadFromUsb,
    #[serde(rename = "ballot-configure-machine-complete")]
    BallotConfiguredOnMachine,
    #[serde(rename = "scanner-configure-complete")]
    ScannerConfigured,
    #[serde(rename = "delete-cvr-batch-init")]
    DeleteScanBatchInit,
    #[serde(rename = "delete-cvr-batch-complete")]
    DeleteScanBatchComplete,
    #[serde(rename = "scan-batch-init")]
    ScanBatchInit,
    #[serde(rename = "scan-sheet-complete")]
    ScanSheetComplete,
    #[serde(rename = "scan-batch-complete")]
    ScanBatchComplete,
    #[serde(rename = "scan-batch-continue")]
    ScanBatchContinue,
    #[serde(rename = "scan-adjudication-info")]
    ScanAdjudicationInfo,
    #[serde(rename = "scanner-config-reloaded")]
    ScannerConfigReloaded,
    #[serde(rename = "save-log-file-found")]
    SaveLogFileFound,
    #[serde(rename = "scan-service-config")]
    ScanServiceConfigurationMessage,
    #[serde(rename = "admin-service-config")]
    AdminServiceConfigurationMessage,
    #[serde(rename = "fujitsu-scan-init")]
    FujitsuScanInit,
    #[serde(rename = "fujitsu-scan-sheet-scanned")]
    FujitsuScanImageScanned,
    #[serde(rename = "fujitsu-scan-batch-complete")]
    FujitsuScanBatchComplete,
    #[serde(rename = "fujitsu-scan-message")]
    FujitsuScanMessage,
    #[serde(rename = "convert-log-cdf-complete")]
    LogConversionToCdfComplete,
    #[serde(rename = "convert-log-cdf-log-line-error")]
    LogConversionToCdfLogLineError,
    #[serde(rename = "reboot-machine")]
    RebootMachine,
    #[serde(rename = "power-down-machine")]
    PowerDown,
    #[serde(rename = "election-package-load-from-usb-complete")]
    ElectionPackageLoadedFromUsb,
    #[serde(rename = "export-cast-vote-records-init")]
    ExportCastVoteRecordsInit,
    #[serde(rename = "export-cast-vote-records-complete")]
    ExportCastVoteRecordsComplete,
    #[serde(rename = "polls-opened")]
    PollsOpened,
    #[serde(rename = "voting-paused")]
    VotingPaused,
    #[serde(rename = "voting-resumed")]
    VotingResumed,
    #[serde(rename = "polls-closed")]
    PollsClosed,
    #[serde(rename = "reset-polls-to-paused")]
    ResetPollsToPaused,
    #[serde(rename = "ballot-bag-replaced")]
    BallotBagReplaced,
    #[serde(rename = "ballot-box-emptied")]
    BallotBoxEmptied,
    #[serde(rename = "precinct-configuration-changed")]
    PrecinctConfigurationChanged,
    #[serde(rename = "scanner-batch-started")]
    ScannerBatchStarted,
    #[serde(rename = "scanner-batch-ended")]
    ScannerBatchEnded,
    #[serde(rename = "scanner-state-machine-event")]
    ScannerEvent,
    #[serde(rename = "scanner-state-machine-transition")]
    ScannerStateChanged,
    #[serde(rename = "pat-device-error")]
    PatDeviceError,
    #[serde(rename = "paper-handler-state-machine-transition")]
    PaperHandlerStateChanged,
    #[serde(rename = "vote-cast")]
    VoteCast,
    #[serde(rename = "ballot-invalidated")]
    BallotInvalidated,
    #[serde(rename = "poll-worker-confirmed-ballot-removal")]
    PollWorkerConfirmedBallotRemoval,
    #[serde(rename = "blank-sheet-interpretation")]
    BlankInterpretation,
    #[serde(rename = "paper-handler-connection")]
    PaperHandlerConnection,
    #[serde(rename = "create-virtual-uinput-device-init")]
    CreateVirtualUinputDeviceInit,
    #[serde(rename = "create-virtual-uinput-device-complete")]
    CreateVirtualUinputDeviceComplete,
    #[serde(rename = "connect-to-pat-input-init")]
    ConnectToPatInputInit,
    #[serde(rename = "connect-to-pat-input-complete")]
    ConnectToPatInputComplete,
    #[serde(rename = "controller-connection-init")]
    ControllerConnectionInit,
    #[serde(rename = "controller-connection-complete")]
    ControllerConnectionComplete,
    #[serde(rename = "controller-handshake-init")]
    ControllerHandshakeInit,
    #[serde(rename = "controller-handshake-complete")]
    ControllerHandshakeComplete,
    #[serde(rename = "error-setting-sigint-handler")]
    ErrorSettingSigintHandler,
    #[serde(rename = "unexpected-hardware-device-response")]
    UnexpectedHardwareDeviceResponse,
    #[serde(rename = "diagnostic-init")]
    DiagnosticInit,
    #[serde(rename = "diagnostic-complete")]
    DiagnosticComplete,
    #[serde(rename = "readiness-report-printed")]
    ReadinessReportPrinted,
    #[serde(rename = "unknown-error")]
    UnknownError,
}
