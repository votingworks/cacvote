use std::io::{Read, Write};

use crate::{decoder::Decoder, encoder::Encoder, length::Length};

/// Encodes a value into a writer.
pub trait Encode<W: Write> {
    /// Converts the value into a byte representation and writes it to the
    /// encoder.
    fn encode(&self, encoder: &mut Encoder<W>) -> std::io::Result<()>;

    /// Returns the length of the encoded value and, if it was required to
    /// compute the data to get the length, the data itself.
    fn compute_length(&self) -> std::io::Result<(Length, Option<Vec<u8>>)>;
}

/// Decodes a value from a reader.
pub trait Decode<R: Read>
where
    Self: Sized,
{
    /// Reads the value from the decoder, using the number of bytes described by
    /// the provided length.
    fn decode(decoder: &mut Decoder<R>, length: &Length) -> std::io::Result<Self>;
}

macro_rules! impl_coders {
    ($($t:ty),*) => {
        $(
            impl<W: Write> Encode<W> for $t {
                fn encode(&self, encoder: &mut Encoder<W>) -> std::io::Result<()> {
                    encoder.write_all(&self.to_be_bytes())
                }

                fn compute_length(&self) -> std::io::Result<(Length, Option<Vec<u8>>)> {
                    Ok((Length::new(std::mem::size_of::<$t>() as u16), None))
                }
            }

            impl<R: Read> Decode<R> for $t {
                fn decode(decoder: &mut Decoder<R>, length: &Length) -> std::io::Result<Self> {
                    assert_eq!(length.length(), std::mem::size_of::<$t>() as u16);
                    let mut buf = [0; std::mem::size_of::<$t>()];
                    decoder.read_exact(&mut buf)?;
                    Ok(<$t>::from_be_bytes(buf))
                }
            }
        )*
    };
}

impl_coders!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl<W: Write> Encode<W> for [u8] {
    fn encode(&self, encoder: &mut Encoder<W>) -> std::io::Result<()> {
        if self.len() > u16::MAX as usize {
            return Err(std::io::ErrorKind::InvalidData.into());
        }

        encoder.write_all(self)
    }

    fn compute_length(&self) -> std::io::Result<(Length, Option<Vec<u8>>)> {
        if self.len() > u16::MAX as usize {
            return Err(std::io::ErrorKind::InvalidData.into());
        }

        Ok((Length::new(self.len() as u16), None))
    }
}

impl<W: Write> Encode<W> for Vec<u8> {
    fn encode(&self, encoder: &mut Encoder<W>) -> std::io::Result<()> {
        self.as_slice().encode(encoder)
    }

    fn compute_length(&self) -> std::io::Result<(Length, Option<Vec<u8>>)> {
        <[u8] as Encode<W>>::compute_length(self)
    }
}

impl<W: Write> Encode<W> for String {
    fn encode(&self, encoder: &mut Encoder<W>) -> std::io::Result<()> {
        encoder.write_all(self.as_bytes())
    }

    fn compute_length(&self) -> std::io::Result<(Length, Option<Vec<u8>>)> {
        if self.len() > u16::MAX as usize {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        Ok((
            Length::new(self.len() as u16),
            Some(self.as_bytes().to_vec()),
        ))
    }
}

impl<R: Read> Decode<R> for Vec<u8> {
    fn decode(decoder: &mut Decoder<R>, length: &Length) -> std::io::Result<Self> {
        let mut data = vec![0; length.length() as usize];
        decoder.read_exact(&mut data)?;
        Ok(data)
    }
}

impl<R: Read> Decode<R> for String {
    fn decode(decoder: &mut Decoder<R>, length: &Length) -> std::io::Result<Self> {
        Vec::<u8>::decode(decoder, length).and_then(|data| {
            std::string::String::from_utf8(data).map_err(|_| std::io::ErrorKind::InvalidData.into())
        })
    }
}

impl<R: Read> Decode<R> for () {
    fn decode(_decoder: &mut Decoder<R>, length: &Length) -> std::io::Result<Self> {
        if length.length() != 0 {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        Ok(())
    }
}
