pub mod coder;
pub mod decoder;
pub mod encoder;
pub mod length;
pub mod tag;

use std::io::Write;

pub use coder::{Decode, Encode};
pub use decoder::Decoder;
pub use encoder::Encoder;
pub use length::Length;
pub use tag::Tag;

pub fn encode_to_vec<W: Write>(value: impl Encode<W>) -> std::io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut encoder = Encoder::new(&mut buffer);
    value.encode(&mut encoder)?;
    Ok(())
}

/// Decodes a value from a reader.
pub fn decode<'a, T: Decode<&'a [u8]>>(buffer: &'a [u8]) -> std::io::Result<T> {
    let mut decoder = Decoder::new(buffer);
    let length =
        Length::new(u16::try_from(buffer.len()).map_err(|_| std::io::ErrorKind::InvalidData)?);
    let value = T::decode(&mut decoder, &length)?;
    if decoder.remaining()?.is_empty() {
        Ok(value)
    } else {
        Err(std::io::ErrorKind::InvalidData.into())
    }
}

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use crate::{decode, decoder::Decoder, encoder::Encoder, tag::Tag};

    #[test]
    fn test_full_tlv_tag_mismatch() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        let tag = Tag::U16(0x0102);
        let value: u16 = 0xbeef;
        encoder.encode(&tag, &value).unwrap();

        let mut decoder = Decoder::new(&buf[..]);
        let tag = Tag::U16(0x0103);
        decoder.decode::<u16>(&tag).unwrap_err();

        assert_eq!(decoder.remaining().unwrap(), vec![0x02, 0xbe, 0xef]);
    }

    #[test]
    fn test_multiple_decode() {
        let data: Vec<u8> = vec![0x01, 0x02, 0xbe, 0xef, 0x99, 0x00, 0xff, 0x01, 0xfe];
        let mut decoder = Decoder::new(&data[..]);

        let beef: u16 = decoder.decode(&Tag::U8(0x01)).unwrap();
        assert_eq!(beef, 0xbeef);

        decoder.decode::<()>(&Tag::U8(0x99)).unwrap();

        let fe: u8 = decoder.decode(&Tag::U8(0xff)).unwrap();
        assert_eq!(fe, 0xfe);

        assert_eq!(decoder.remaining().unwrap(), vec![]);
    }

    #[test]
    fn test_medium_length() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        let tag = Tag::U8(0x01);
        let value = vec![0xbe; 0x81];
        encoder.encode(&tag, &value).unwrap();

        //                    tag    MED   len   data …
        assert_eq!(buf[0..4], [0x01, 0x81, 0x81, 0xbe]);

        let mut decoder = Decoder::new(&buf[..]);
        let decoded_value: Vec<u8> = decoder.decode(&tag).unwrap();
        assert_eq!(decoded_value, value);
        assert_eq!(decoder.remaining().unwrap(), vec![]);
    }

    #[test]
    fn test_long_length() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        let tag = Tag::U8(0x01);
        let value = vec![0xbe; 0x100];
        encoder.encode(&tag, &value).unwrap();

        //                     tag   LONG  len0  len1  data …
        assert_eq!(buf[0..5], [0x01, 0x82, 0x01, 0x00, 0xbe]);

        let mut decoder = Decoder::new(&buf[..]);
        let decoded_value: Vec<u8> = decoder.decode(&tag).unwrap();
        assert_eq!(decoded_value, value);
        assert_eq!(decoder.remaining().unwrap(), vec![]);
    }

    proptest! {
        #[test]
        fn test_encode_decode_u8(tag_byte: u8, value: u8) {
            let tag = Tag::U8(tag_byte);
            let mut buf = Vec::new();
            let mut encoder = Encoder::new(&mut buf);
            encoder.encode(&tag, &value).unwrap();

            let decoded_value: u8 = decode(&buf).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_u16(tag_byte: u8, value: u16) {
            let tag = Tag::U8(tag_byte);
            let mut buf = Vec::new();
            let mut encoder = Encoder::new(&mut buf);
            encoder.encode(&tag, &value).unwrap();

            let mut decoder = Decoder::new(&buf[..]);
            let decoded_value: u16 = decoder.decode(&tag).unwrap();
            assert_eq!(decoded_value, value);
            assert_eq!(decoder.remaining().unwrap(), vec![]);
        }

        #[test]
        fn test_encode_decode_u32(tag_byte: u8, value: u32) {
            let tag = Tag::U8(tag_byte);
            let mut buf = Vec::new();
            let mut encoder = Encoder::new(&mut buf);
            encoder.encode(&tag, &value).unwrap();

            let mut decoder = Decoder::new(&buf[..]);
            let decoded_value: u32 = decoder.decode(&tag).unwrap();
            assert_eq!(decoded_value, value);
            assert_eq!(decoder.remaining().unwrap(), vec![]);
        }

        #[test]
        fn test_encode_decode_u64(tag_byte: u8, value: u64) {
            let tag = Tag::U8(tag_byte);
            let mut buf = Vec::new();
            let mut encoder = Encoder::new(&mut buf);
            encoder.encode(&tag, &value).unwrap();

            let mut decoder = Decoder::new(&buf[..]);
            let decoded_value: u64 = decoder.decode(&tag).unwrap();
            assert_eq!(decoded_value, value);
            assert_eq!(decoder.remaining().unwrap(), vec![]);
        }

        #[test]
        fn test_encode_decode_buffer(tag_byte: u8, value: Vec<u8>) {
            let tag = Tag::U8(tag_byte);
            let mut buf = Vec::new();
            let mut encoder = Encoder::new(&mut buf);
            encoder.encode(&tag, &value).unwrap();

            let mut decoder = Decoder::new(&buf[..]);
            let decoded_value: Vec<u8> = decoder.decode(&tag).unwrap();
            assert_eq!(decoded_value, value);
            assert_eq!(decoder.remaining().unwrap(), vec![]);
        }

        #[test]
        fn test_encode_decode_string(tag_byte: u8, value: String) {
            let tag = Tag::U8(tag_byte);
            let mut buf = Vec::new();
            let mut encoder = Encoder::new(&mut buf);
            encoder.encode(&tag, &value).unwrap();

            let mut decoder = Decoder::new(&buf[..]);
            let decoded_value: String = decoder.decode(&tag).unwrap();
            assert_eq!(decoded_value, value);
            assert_eq!(decoder.remaining().unwrap(), vec![]);
        }
    }
}
