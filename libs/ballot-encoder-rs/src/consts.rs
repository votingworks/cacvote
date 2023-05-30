pub(crate) const ENCODING_VERSION: u8 = 2;
pub(crate) const ELECTION_HASH_HEX_LENGTH: usize = 20;
pub(crate) const ELECTION_HASH_BYTE_LENGTH: usize = ELECTION_HASH_HEX_LENGTH / 2;
pub(crate) const MAXIMUM_WRITE_IN_NAME_LENGTH: usize = 40;
pub(crate) const BITS_PER_WRITE_IN_CHAR: u32 = 5;
pub(crate) const WRITE_IN_CHARS: &str = r#"ABCDEFGHIJKLMNOPQRSTUVWXYZ '"-.,"#;
