/// A tag for TLV encoding.
#[derive(Debug, PartialEq)]
pub enum Tag {
    /// A tag with a single byte.
    U8(u8),

    /// A tag with two bytes.
    U16(u16),
}

impl Tag {
    /// Returns the length of the tag in bytes.
    pub fn len(&self) -> usize {
        match self {
            Self::U8(_) => std::mem::size_of::<u8>(),
            Self::U16(_) => std::mem::size_of::<u16>(),
        }
    }
}

impl PartialEq<[u8]> for Tag {
    fn eq(&self, other: &[u8]) -> bool {
        match (self, other) {
            (Tag::U8(byte), [other_byte]) => byte == other_byte,
            (Tag::U16(value), [other_byte0, other_byte1]) => {
                let other_value = u16::from_be_bytes([*other_byte0, *other_byte1]);
                value == &other_value
            }
            _ => false,
        }
    }
}
