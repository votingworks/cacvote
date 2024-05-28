use std::io::Read;

use crate::{Encode, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Length {
    length: u16,
}

impl Length {
    const MEDIUM_LENGTH_MINIMUM: u16 = 0x81;
    const MEDIUM_LENGTH_MAXIMUM: u16 = 0xff;
    const MEDIUM_LENGTH_TOKEN: u8 = 0x81;
    const LONG_LENGTH_TOKEN: u8 = 0x82;

    pub fn new(length: u16) -> Self {
        Self { length }
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0; 1];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            Self::MEDIUM_LENGTH_TOKEN => {
                let mut buf = [0; 1];
                reader.read_exact(&mut buf)?;
                Ok(Self::new(u16::from(buf[0])))
            }
            Self::LONG_LENGTH_TOKEN => {
                let mut buf = [0; 2];
                reader.read_exact(&mut buf)?;
                Ok(Self::new(u16::from_be_bytes(buf)))
            }
            length => Ok(Self::new(u16::from(length))),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let length = self.value();

        if length < Length::MEDIUM_LENGTH_MINIMUM {
            vec![length as u8]
        } else if length < Length::MEDIUM_LENGTH_MAXIMUM {
            vec![Length::MEDIUM_LENGTH_TOKEN, length as u8]
        } else {
            vec![
                Length::LONG_LENGTH_TOKEN,
                (length >> 8) as u8,
                (length & 0xff) as u8,
            ]
        }
    }

    pub fn value(&self) -> u16 {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl Encode for Length {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: std::io::prelude::Write,
    {
        let bytes: Vec<u8> = self.to_vec();
        Ok(writer.write_all(&bytes)?)
    }

    fn encoded_length(&self) -> Result<Length> {
        let bytes: Vec<u8> = self.to_vec();
        Ok(Length::new(bytes.len() as u16))
    }
}

impl std::ops::Add for Length {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.length + other.length)
    }
}

impl std::ops::AddAssign for Length {
    fn add_assign(&mut self, other: Self) {
        self.length += other.length;
    }
}
