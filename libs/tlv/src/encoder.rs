use std::io::Write;

use crate::{coder::Encode, tag::Tag};

pub struct Encoder<W: Write> {
    writer: W,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn encode(&mut self, tag: &Tag, value: &impl Encode<W>) -> std::io::Result<()> {
        match tag {
            Tag::U8(value) => self.write_all(&value.to_be_bytes())?,
            Tag::U16(value) => self.write_all(&value.to_be_bytes())?,
        }

        match value.compute_length()? {
            (length, Some(data)) => {
                let length_bytes: Vec<u8> = length.into();
                self.write_all(&length_bytes)?;
                self.write_all(&data)
            }
            (length, None) => {
                let length_bytes: Vec<u8> = length.into();
                self.write_all(&length_bytes)?;
                value.encode(self)
            }
        }
    }

    pub fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(buf)
    }
}
