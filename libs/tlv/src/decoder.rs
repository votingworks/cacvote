use std::io::Read;

use crate::{coder::Decode, length::Length, tag::Tag};

pub struct Decoder<R: Read> {
    reader: R,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn decode<T: Decode<R>>(&mut self, tag: &Tag) -> std::io::Result<T> {
        let mut tag_buf = vec![0; tag.len()];
        self.reader.read_exact(&mut tag_buf)?;

        if tag != tag_buf.as_slice() {
            return Err(std::io::ErrorKind::InvalidData.into());
        }

        let length = Length::from_reader(&mut self.reader)?;
        T::decode(self, &length)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.reader.read_exact(buf)
    }

    pub fn remaining(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
