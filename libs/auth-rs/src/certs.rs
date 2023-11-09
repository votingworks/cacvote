/// VotingWorks's IANA-assigned enterprise OID
pub const VX_IANA_ENTERPRISE_OID: &str = "1.3.6.1.4.1.59817";

/// One of: admin, central-scan, mark, scan, card (the first four referring to machines)
pub const VX_CUSTOM_CERT_FIELD_COMPONENT: &str = "1.3.6.1.4.1.59817.1";

/// Format: {state-2-letter-abbreviation}.{county-or-town} (e.g. ms.warren or ca.los-angeles)
pub const VX_CUSTOM_CERT_FIELD_JURISDICTION: &str = "1.3.6.1.4.1.59817.2";

/// One of: system-administrator, election-manager, poll-worker, poll-worker-with-pin
pub const VX_CUSTOM_CERT_FIELD_CARD_TYPE: &str = "1.3.6.1.4.1.59817.3";

/// The SHA-256 hash of the election definition
pub const VX_CUSTOM_CERT_FIELD_ELECTION_HASH: &str = "1.3.6.1.4.1.59817.4";
