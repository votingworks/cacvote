use std::fmt;

/// A Tag-Length-Value (TLV) structure.
///
/// See https://en.wikipedia.org/wiki/Type-length-value for more information.
#[derive(Clone, PartialEq, Eq)]
pub struct Tlv {
    tag: u8,
    value: Vec<u8>,
}

impl fmt::Debug for Tlv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tlv")
            .field("tag", &format_args!("{:02x}", self.tag))
            .field("value", &format_args!("{:02x?}", self.value))
            .finish()
    }
}

impl Tlv {
    /// Creates a new TLV with the given tag and value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use auth_rs::tlv::Tlv;
    /// let bytes = Tlv::new(0x01, vec![0x02, 0x03]).to_bytes().unwrap();
    /// assert_eq!(bytes, vec![0x01, 0x02, 0x02, 0x03]);
    /// ```
    pub fn new(tag: u8, value: Vec<u8>) -> Self {
        Self { tag, value }
    }

    /// Returns the tag of the TLV.
    pub fn tag(&self) -> u8 {
        self.tag
    }

    /// Returns the value of the TLV.
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// Parses a TLV from the given value, returning the remainder of the value and the parsed TLV.
    ///
    /// # Examples
    ///
    /// ```
    /// # use auth_rs::tlv::{Tlv, ParseError};
    /// let (remainder, tlv) = Tlv::parse_partial(0x01, &[0x01, 0x01, 0x03]).unwrap();
    /// assert!(remainder.is_empty());
    /// assert_eq!(tlv, Tlv::new(0x01, vec![0x03]));
    ///
    /// let (remainder, tlv) = Tlv::parse_partial(0x01, &[0x01, 0x01, 0x03, 0x04, 0x01, 0xfa]).unwrap();
    /// assert_eq!(remainder, vec![0x04, 0x01, 0xfa]);
    /// assert_eq!(tlv, Tlv::new(0x01, vec![0x03]));
    ///
    /// let (remainder, tlv) = Tlv::parse_partial(0x04, &remainder).unwrap();
    /// assert!(remainder.is_empty());
    /// assert_eq!(tlv, Tlv::new(0x04, vec![0xfa]));

    /// assert_eq!(
    ///     Tlv::parse_partial(0x01, &[0x02, 0x01, 0x04]),
    ///     Err(ParseError::InvalidTag {
    ///         expected: 0x01,
    ///         actual: 0x02
    ///     })
    /// );
    /// ```
    pub fn parse_partial(tag: u8, value: &[u8]) -> Result<(Vec<u8>, Self), ParseError> {
        if value.len() < 2 {
            return Err(ParseError::TooShort {
                length: value.len(),
            });
        }

        if tag != value[TAG_OFFSET] {
            return Err(ParseError::InvalidTag {
                expected: tag,
                actual: value[TAG_OFFSET],
            });
        }

        let (length, length_len) = match value[LENGTH_OFFSET] {
            length @ 0x00..=0x80 => (length as usize, 1),
            0x81 => match value.get(LENGTH_OFFSET + 1) {
                Some(length) => (*length as usize, 2),
                None => {
                    return Err(ParseError::TooShort {
                        length: value.len(),
                    })
                }
            },
            0x82 => match value.get(LENGTH_OFFSET + 1..=LENGTH_OFFSET + 2) {
                Some([length1, length2]) => {
                    let length = u16::from_be_bytes([*length1, *length2]);
                    (length as usize, 3)
                }
                _ => {
                    return Err(ParseError::TooShort {
                        length: value.len(),
                    })
                }
            },
            _ => {
                return Err(ParseError::InvalidLength {
                    length: value[1] as usize,
                })
            }
        };

        if value.len() < 1 + length_len + length {
            return Err(ParseError::InvalidLength {
                length: value.len(),
            });
        }

        let (value, remainder) = value[LENGTH_OFFSET + length_len..].split_at(length);

        Ok((remainder.to_vec(), Tlv::new(tag, value.to_vec())))
    }

    /// Parses a TLV from the given value, returning the parsed TLV if the value
    /// is completely consumed. Otherwise, returns an error with the parsed TLV
    /// and the remainder of the value.
    ///
    /// ```
    /// # use auth_rs::tlv::{Tlv, ParseError};
    /// assert_eq!(
    ///     Tlv::parse(0x01, &[0x01, 0x01, 0x03]),
    ///     Ok(Tlv::new(0x01, vec![0x03]))
    /// );
    /// assert_eq!(
    ///     Tlv::parse(0x01, &[0x01, 0x01, 0x03, 0x04, 0x01, 0xfa]),
    ///     Err(ParseError::UnexpectedRemainder {
    ///         tlv: Tlv::new(0x01, vec![0x03]),
    ///         remainder: vec![0x04, 0x01, 0xfa]
    ///     })
    /// );
    /// ```
    pub fn parse(tag: u8, value: &[u8]) -> Result<Self, ParseError> {
        let (remainder, tlv) = Self::parse_partial(tag, value)?;
        if !remainder.is_empty() {
            return Err(ParseError::UnexpectedRemainder { tlv, remainder });
        }
        Ok(tlv)
    }

    /// Converts the TLV to a byte vector.
    pub fn to_bytes(self) -> Result<Vec<u8>, ConstructError> {
        let mut bytes = vec![self.tag];
        let value = self.value;
        if value.len() <= 0x80 {
            bytes.push(value.len() as u8);
        } else if value.len() <= 0xff {
            bytes.push(0x81);
            bytes.push(value.len() as u8);
        } else if value.len() <= 0xffff {
            bytes.push(0x82);
            bytes.extend(&(value.len() as u16).to_be_bytes());
        } else {
            return Err(ConstructError::ValueTooLong);
        }
        bytes.extend(value);
        Ok(bytes)
    }
}

const TAG_OFFSET: usize = 0;
const LENGTH_OFFSET: usize = 1;

#[derive(Debug, thiserror::Error)]
pub enum ConstructError {
    #[error("value is too long")]
    ValueTooLong,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("too short: {length} < 2")]
    TooShort { length: usize },
    #[error("invalid tag: expected = {expected}, actual = {actual}")]
    InvalidTag { expected: u8, actual: u8 },
    #[error("invalid length: {length}")]
    InvalidLength { length: usize },
    #[error("unexpected remainder for TLV: tlv = {tlv:?}, remainder = {remainder:02x?}")]
    UnexpectedRemainder { tlv: Tlv, remainder: Vec<u8> },
}

#[macro_export]
macro_rules! tlv {
    ($tag: expr, $value: expr) => {{
        $crate::tlv::Tlv::new($tag, $value.into())
            .to_bytes()
            .unwrap()
    }};
}

#[macro_export]
macro_rules! concat_bytes {
    ($($bytes: expr),* $(,)?) => {
        {
            let mut bytes = Vec::new();
            $(bytes.extend_from_slice(&$bytes);)*
            bytes
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_tlv {
        (value_length = $len:expr, length_bytes = $($bytes:expr),*) => {
            let buffer = vec![0; $len];
            let tlv = Tlv::new(0x01, buffer.clone());
            let serialized = tlv.to_bytes().unwrap();
            let expected = vec![0x01, $($bytes),*].into_iter().chain(buffer.into_iter()).collect::<Vec<u8>>();
            assert_eq!(serialized, expected);
        };
    }

    #[test]
    fn test_tlv_length() {
        assert_tlv!(value_length = 0x0, length_bytes = 0x0);
        assert_tlv!(value_length = 0x05, length_bytes = 0x05);
        assert_tlv!(value_length = 0x33, length_bytes = 0x33);
        assert_tlv!(value_length = 0x7f, length_bytes = 0x7f);
        assert_tlv!(value_length = 0x80, length_bytes = 0x80);
        assert_tlv!(value_length = 0x9f, length_bytes = 0x81, 0x9f);
        assert_tlv!(value_length = 0xff, length_bytes = 0x81, 0xff);
        assert_tlv!(value_length = 0x0bc9, length_bytes = 0x82, 0x0b, 0xc9);
        assert_tlv!(value_length = 0xffff, length_bytes = 0x82, 0xff, 0xff);
    }

    #[test]
    fn test_construct() {
        let bytes = Tlv::new(0x01, vec![0x02, 0x03]).to_bytes().unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x02, 0x03]);

        let bytes = tlv!(0x01, vec![0x02, 0x03]);
        assert_eq!(bytes, vec![0x01, 0x02, 0x02, 0x03]);
    }
}
