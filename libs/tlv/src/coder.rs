use std::io::{Read, Write};

use crate::length::Length;

/// Encodes a value into a writer.
pub trait Encode {
    /// Converts the value into a byte representation and writes it to the
    /// writer.
    fn encode<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write;

    /// Returns the length of the encoded value.
    fn encoded_length(&self) -> std::io::Result<Length>;
}

/// Decodes a value from a decoder.
pub trait Decode
where
    Self: Sized,
{
    /// Reads the value from the decoder.
    fn decode<R>(reader: &mut R) -> std::io::Result<(Self, usize)>
    where
        R: Read;
}

macro_rules! impl_coders {
    ($($t:ty),*) => {
        $(
            impl Encode for $t {
                fn encode<W>(&self, writer: &mut W) -> std::io::Result<()> where W: Write {
                    writer.write_all(&self.to_be_bytes())
                }

                fn encoded_length(&self) -> std::io::Result<Length> {
                    Ok(Length::new(std::mem::size_of::<$t>() as u16))
                }
            }

            impl Decode for $t {
                fn decode<R>(reader: &mut R) -> std::io::Result<(Self, usize)> where R: Read {
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
    fn encode<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        (*self).encode(writer)
    }

    fn encoded_length(&self) -> std::io::Result<Length> {
        (*self).encoded_length()
    }
}

impl Encode for bool {
    fn encode<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        (*self as u8).encode(writer)
    }

    fn encoded_length(&self) -> std::io::Result<Length> {
        <u8 as Encode>::encoded_length(&(*self as u8))
    }
}

impl Encode for [u8] {
    fn encode<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        if self.len() > u16::MAX as usize {
            return Err(std::io::ErrorKind::InvalidData.into());
        }

        writer.write_all(self)
    }

    fn encoded_length(&self) -> std::io::Result<Length> {
        if self.len() > u16::MAX as usize {
            Err(std::io::ErrorKind::InvalidData.into())
        } else {
            Ok(Length::new(self.len() as u16))
        }
    }
}

impl Encode for Vec<u8> {
    fn encode<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        self.as_slice().encode(writer)
    }

    fn encoded_length(&self) -> std::io::Result<Length> {
        <[u8] as Encode>::encoded_length(self)
    }
}

impl Encode for String {
    fn encode<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        writer.write_all(self.as_bytes())
    }

    fn encoded_length(&self) -> std::io::Result<Length> {
        if self.len() > u16::MAX as usize {
            Err(std::io::ErrorKind::InvalidData.into())
        } else {
            Ok(Length::new(self.len() as u16))
        }
    }
}

impl Decode for Vec<u8> {
    fn decode<R>(reader: &mut R) -> std::io::Result<(Self, usize)>
    where
        R: Read,
    {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let len = data.len();
        Ok((data, len))
    }
}

impl Decode for String {
    fn decode<R>(reader: &mut R) -> std::io::Result<(Self, usize)>
    where
        R: Read,
    {
        Vec::<u8>::decode(reader).and_then(|(data, read)| {
            match std::string::String::from_utf8(data) {
                Ok(string) => Ok((string, read)),
                Err(_) => Err(std::io::ErrorKind::InvalidData.into()),
            }
        })
    }
}

impl Decode for () {
    fn decode<R>(_reader: &mut R) -> std::io::Result<(Self, usize)>
    where
        R: Read,
    {
        Ok(((), 0))
    }
}
