use std::io::{Read, Write};

use crate::error::{Error, Result};
use crate::length::Length;

/// Encodes a value into a writer.
pub trait Encode {
    /// Converts the value into a byte representation and writes it to the
    /// writer.
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write;

    /// Returns the length of the encoded value.
    fn encoded_length(&self) -> Result<Length>;
}

/// Decodes a value from a decoder.
pub trait Decode
where
    Self: Sized,
{
    /// Reads the value from the decoder.
    fn decode<R>(reader: &mut R) -> Result<(Self, usize)>
    where
        R: Read;
}

macro_rules! impl_coders {
    ($($t:ty),*) => {
        $(
            impl Encode for $t {
                fn encode<W>(&self, writer: &mut W) -> Result<()> where W: Write {
                    Ok(writer.write_all(&self.to_be_bytes())?)
                }

                fn encoded_length(&self) -> Result<Length> {
                    Ok(Length::new(std::mem::size_of::<$t>() as u16))
                }
            }

            impl Decode for $t {
                fn decode<R>(reader: &mut R) -> Result<(Self, usize)> where R: Read {
                    const SIZE: usize = std::mem::size_of::<$t>();
                    let mut buf = [0; SIZE];
                    reader.read_exact(&mut buf)?;
                    Ok((<$t>::from_be_bytes(buf), SIZE))
                }
            }
        )*
    };
}

impl_coders!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl<T: Encode> Encode for &T {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        (*self).encode(writer)
    }

    fn encoded_length(&self) -> Result<Length> {
        (*self).encoded_length()
    }
}

impl Encode for bool {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        (*self as u8).encode(writer)
    }

    fn encoded_length(&self) -> Result<Length> {
        <u8 as Encode>::encoded_length(&(*self as u8))
    }
}

impl Decode for bool {
    fn decode<R>(reader: &mut R) -> Result<(Self, usize)>
    where
        R: Read,
    {
        u8::decode(reader).map(|(value, size)| (value != 0, size))
    }
}

impl Encode for [u8] {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        match u16::try_from(self.len()) {
            Ok(_) => Ok(writer.write_all(self)?),
            Err(_) => Err(Error::ValueTooLong(self.len())),
        }
    }

    fn encoded_length(&self) -> Result<Length> {
        match u16::try_from(self.len()) {
            Ok(len) => Ok(Length::new(len)),
            Err(_) => Err(Error::ValueTooLong(self.len())),
        }
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        Ok(writer.write_all(self)?)
    }

    fn encoded_length(&self) -> Result<Length> {
        Ok(Length::new(N as u16))
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode<R>(reader: &mut R) -> Result<(Self, usize)>
    where
        R: Read,
    {
        let mut data = [0; N];
        reader.read_exact(&mut data)?;
        Ok((data, N))
    }
}

impl Encode for Vec<u8> {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        self.as_slice().encode(writer)
    }

    fn encoded_length(&self) -> Result<Length> {
        <[u8] as Encode>::encoded_length(self)
    }
}

impl Decode for Vec<u8> {
    fn decode<R>(reader: &mut R) -> Result<(Self, usize)>
    where
        R: Read,
    {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let len = data.len();
        Ok((data, len))
    }
}

impl Encode for String {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        Ok(writer.write_all(self.as_bytes())?)
    }

    fn encoded_length(&self) -> Result<Length> {
        match u16::try_from(self.len()) {
            Ok(len) => Ok(Length::new(len)),
            Err(_) => Err(Error::ValueTooLong(self.len())),
        }
    }
}

impl Decode for String {
    fn decode<R>(reader: &mut R) -> Result<(Self, usize)>
    where
        R: Read,
    {
        let (data, read) = Vec::<u8>::decode(reader)?;
        Ok((std::string::String::from_utf8(data)?, read))
    }
}

impl Decode for () {
    fn decode<R>(_reader: &mut R) -> Result<(Self, usize)>
    where
        R: Read,
    {
        Ok(((), 0))
    }
}

const UUID_BYTES: usize = std::mem::size_of::<uuid::Bytes>();

impl Encode for uuid::Uuid {
    fn encode<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        let bytes = self.as_bytes();

        if bytes.len() != UUID_BYTES {
            return Err(Error::InvalidLength {
                expected: UUID_BYTES,
                actual: bytes.len(),
            });
        }

        Ok(writer.write_all(bytes)?)
    }

    fn encoded_length(&self) -> Result<Length> {
        Ok(Length::new(UUID_BYTES as u16))
    }
}

impl Decode for uuid::Uuid {
    fn decode<R>(reader: &mut R) -> Result<(Self, usize)>
    where
        R: Read,
    {
        let mut data = [0; UUID_BYTES];
        reader.read_exact(&mut data)?;
        Ok((uuid::Uuid::from_bytes(data), UUID_BYTES))
    }
}
