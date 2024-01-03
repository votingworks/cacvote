#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tlv {
    tag: u8,
    value: Vec<u8>,
}

impl Tlv {
    pub fn new(tag: u8, value: Vec<u8>) -> Self {
        Self { tag, value }
    }

    pub fn tag(&self) -> u8 {
        self.tag
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

impl TryFrom<&Tlv> for Vec<u8> {
    type Error = ConstructError;

    fn try_from(tlv: &Tlv) -> Result<Self, Self::Error> {
        let mut bytes = vec![tlv.tag];
        let value = &tlv.value;
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

impl TryFrom<Tlv> for Vec<u8> {
    type Error = ConstructError;

    fn try_from(value: Tlv) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

const TAG_OFFSET: usize = 0;
const LENGTH_OFFSET: usize = 1;

impl TryFrom<&[u8]> for Tlv {
    type Error = ParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 2 {
            return Err(ParseError::TooShort {
                length: bytes.len(),
            });
        }

        let tag = bytes[TAG_OFFSET];
        let (length, length_len) = match bytes[LENGTH_OFFSET] {
            length @ 0x00..=0x80 => (length as usize, 1),
            0x81 => match bytes.get(LENGTH_OFFSET + 1) {
                Some(length) => (*length as usize, 2),
                None => {
                    return Err(ParseError::TooShort {
                        length: bytes.len(),
                    })
                }
            },
            0x82 => match bytes.get(LENGTH_OFFSET + 1..=LENGTH_OFFSET + 2) {
                Some([length1, length2]) => {
                    let length = u16::from_be_bytes([*length1, *length2]);
                    (length as usize, 3)
                }
                _ => {
                    return Err(ParseError::TooShort {
                        length: bytes.len(),
                    })
                }
            },
            _ => {
                return Err(ParseError::InvalidLength {
                    length: bytes[1] as usize,
                })
            }
        };

        if bytes.len() < 1 + length_len + length {
            return Err(ParseError::InvalidLength {
                length: bytes.len(),
            });
        }

        Ok(Tlv::new(
            tag,
            bytes[1 + length_len..1 + length_len + length].into(),
        ))
    }
}

impl TryFrom<Vec<u8>> for Tlv {
    type Error = ParseError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConstructError {
    #[error("value is too long")]
    ValueTooLong,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("too short: {length} < 2")]
    TooShort { length: usize },
    #[error("invalid tag: {tag}")]
    InvalidTag { tag: u8 },
    #[error("invalid length: {length}")]
    InvalidLength { length: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_tlv {
        (value_length = $len:expr, length_bytes = $($bytes:expr),*) => {
            let buffer = vec![0; $len];
            let tlv = Tlv::new(0x01, buffer.clone());
            let serialized: Vec<u8> = tlv.try_into().unwrap();
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
        let bytes: Vec<u8> = Tlv::new(0x01, vec![0x02, 0x03]).try_into().unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x02, 0x03]);
    }

    #[test]
    fn test_parse() {
        let tlv = Tlv::try_from(vec![0x01, 0x01, 0x03]).unwrap();
        assert_eq!(tlv, Tlv::new(0x01, vec![0x03]));
    }
}
