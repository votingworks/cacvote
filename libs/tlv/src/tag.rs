use crate::{Encode, Length, Result};

/// A tag for TLV encoding.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tag {
    /// A tag with a single byte.
    U8(u8),

    /// A tag with two bytes.
    U16(u16),
}

impl Tag {
    /// Converts the tag into a byte representation.
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::U8(value) => vec![*value],
            Self::U16(value) => value.to_be_bytes().to_vec(),
        }
    }
}

impl Encode for Tag {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        Ok(writer.write_all(&self.to_vec())?)
    }

    fn encoded_length(&self) -> Result<Length> {
        Ok(Length::new(match self {
            Self::U8(_) => std::mem::size_of::<u8>(),
            Self::U16(_) => std::mem::size_of::<u16>(),
        } as u16))
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
