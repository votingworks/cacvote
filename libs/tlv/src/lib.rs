pub mod coder;
pub mod error;
pub mod length;
mod limited_reader;
pub mod tag;
pub mod value;

pub use coder::{Decode, Encode};
pub use error::{Error, Result};
pub use length::Length;
pub use tag::Tag;

/// Encodes a value into a vector.
///
/// # Errors
///
/// Returns an error if the value cannot be encoded.
///
/// # Panics
///
/// Panics if the encoded length does not match the expected length of the
/// value.
///
/// # Examples
///
/// ```
/// # use tlv::{Encode, Result, to_vec};
/// #
/// # fn main() -> Result<()> {
/// let value: u16 = 0xbeef;
/// let encoded = to_vec(value)?;
/// assert_eq!(encoded, [0xbe, 0xef]);
/// # Ok(())
/// # }
/// ```
///
/// Most of the time you'll want to use this with a struct from `tlv-derive`.
pub fn to_vec<E>(value: E) -> Result<Vec<u8>>
where
    E: Encode,
{
    let mut buffer = Vec::new();
    value.encode(&mut buffer)?;

    let actual = buffer.len();
    let expected = usize::from(value.encoded_length()?.value());

    if actual != expected {
        Err(Error::InvalidLength { actual, expected })
    } else {
        Ok(buffer)
    }
}

/// Encodes a tagged value into a vector.
///
/// # Errors
///
/// Returns an error if the value cannot be encoded.
///
/// # Panics
///
/// Panics if the encoded length does not match the expected length of the
/// value.
///
/// # Examples
///
/// ```
/// # use tlv::{Encode, Result, Tag, to_vec_tagged};
/// #
/// # fn main() -> Result<()> {
/// let value: u16 = 0xbeef;
/// let encoded = to_vec_tagged(Tag::U8(0xff), value)?;
/// assert_eq!(encoded, [0xff, 0x02, 0xbe, 0xef]);
/// # Ok(())
/// # }
/// ```
///
/// Most of the time you'll want to use this with a struct from `tlv-derive`.
pub fn to_vec_tagged<E>(tag: Tag, value: E) -> Result<Vec<u8>>
where
    E: Encode,
{
    let mut buffer = Vec::new();
    value::encode_tagged(tag, value, &mut buffer)?;
    Ok(buffer)
}

/// Decodes a value from a slice.
///
/// # Errors
///
/// Returns an error if the value cannot be decoded.
///
/// # Examples
///
/// ```
/// # use tlv::{Decode, Result, from_slice};
/// #
/// # fn main() -> Result<()> {
/// let buffer = [0xbe, 0xef];
/// let value: u16 = from_slice(&buffer)?;
/// assert_eq!(value, 0xbeef);
/// # Ok(())
/// # }
/// ```
pub fn from_slice<D>(buffer: &[u8]) -> Result<D>
where
    D: Decode,
{
    let mut cursor = std::io::Cursor::new(buffer);
    let (value, read) = D::decode(&mut cursor)?;

    if read != buffer.len() {
        Err(Error::InvalidLength {
            expected: buffer.len(),
            actual: read,
        })
    } else {
        Ok(value)
    }
}

/// Decodes a tagged value from a slice.
///
/// # Errors
///
/// Returns an error if the value cannot be decoded.
///
/// # Examples
///
/// ```
/// # use tlv::{Decode, from_slice_tagged, Result, Tag};
/// #
/// # fn main() -> Result<()> {
/// let buffer = [0xff, 0x02, 0xbe, 0xef];
/// let value: u16 = from_slice_tagged(Tag::U8(0xff), &buffer)?;
/// assert_eq!(value, 0xbeef);
/// # Ok(())
/// # }
/// ```
pub fn from_slice_tagged<D>(tag: Tag, buffer: &[u8]) -> Result<D>
where
    D: Decode,
{
    let mut cursor = std::io::Cursor::new(buffer);
    let (value, read) = value::decode_tagged(tag, &mut cursor)?;

    if read != buffer.len() {
        Err(Error::InvalidLength {
            expected: buffer.len(),
            actual: read,
        })
    } else {
        Ok(value)
    }
}

/// Decodes a tagged value from a reader, returning the value and the number of
/// bytes read.
///
/// # Errors
///
/// Returns an error if the value cannot be decoded.
///
/// # Examples
///
/// ```
/// # use tlv::{Decode, from_reader_tagged, Result, Tag};
/// #
/// # fn main() -> Result<()> {
/// let buffer = [0xff, 0x02, 0xbe, 0xef];
/// let mut cursor = std::io::Cursor::new(&buffer);
/// let (value, read) = from_reader_tagged::<u16, _>(Tag::U8(0xff), &mut cursor)?;
/// assert_eq!(value, 0xbeef);
/// assert_eq!(read, buffer.len());
/// # Ok(())
/// # }
/// ```
pub fn from_reader_tagged<D, R>(tag: Tag, reader: &mut R) -> Result<(D, usize)>
where
    D: Decode,
    R: std::io::Read,
{
    value::decode_tagged(tag, reader)
}

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use crate::{
        from_reader_tagged, from_slice, from_slice_tagged, tag::Tag, to_vec, to_vec_tagged, Decode,
        Encode, Length, Result,
    };

    #[test]
    fn test_full_tlv_tag_mismatch() {
        let value: u16 = 0xbeef;
        let encoded = to_vec_tagged(Tag::U16(0x0102), value).unwrap();

        from_slice_tagged::<Vec<u8>>(Tag::U16(0x0103), &encoded).unwrap_err();
    }

    #[test]
    fn test_multiple_decode() {
        let data: [u8; 9] = [0x01, 0x02, 0xbe, 0xef, 0x99, 0x00, 0xff, 0x01, 0xfe];
        let mut cursor = std::io::Cursor::new(&data);

        let (beef, read) = from_reader_tagged::<u16, _>(Tag::U8(0x01), &mut cursor).unwrap();
        assert_eq!(beef, 0xbeef);
        assert_eq!(read, 4);

        let (_, read) = from_reader_tagged::<(), _>(Tag::U8(0x99), &mut cursor).unwrap();
        assert_eq!(read, 2);

        let (fe, read) = from_reader_tagged::<u8, _>(Tag::U8(0xff), &mut cursor).unwrap();
        assert_eq!(fe, 0xfe);
        assert_eq!(read, 3);

        assert_eq!(cursor.position(), data.len() as u64);
    }

    #[test]
    fn test_multiple_decode_variable_length() {
        let data = b"\x01\x05hello\x02\x05world";
        let mut cursor = std::io::Cursor::new(&data);

        let (hello, read) = from_reader_tagged::<String, _>(Tag::U8(0x01), &mut cursor).unwrap();
        assert_eq!(hello, "hello");
        assert_eq!(read, 7);

        let (world, read) = from_reader_tagged::<String, _>(Tag::U8(0x02), &mut cursor).unwrap();
        assert_eq!(world, "world");
        assert_eq!(read, 7);

        assert_eq!(cursor.position(), data.len() as u64);
    }

    #[test]
    fn test_encode_codegen() {
        struct Test {
            a: u8,
            b: u16,
        }

        // This is implemented in a way that is conducive to code generation.
        impl Encode for Test {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
                crate::value::encode_tagged(Tag::U8(0x01), self.a, writer)?;
                crate::value::encode_tagged(Tag::U8(0x02), self.b, writer)?;
                Ok(())
            }

            fn encoded_length(&self) -> Result<crate::Length> {
                let mut length = Length::new(0);
                length += crate::value::length_tagged(Tag::U8(0x01), self.a)?;
                length += crate::value::length_tagged(Tag::U8(0x01), self.b)?;
                Ok(length)
            }
        }

        let value = Test { a: 0x99, b: 0xabcd };
        let encoded = [0x01, 0x01, 0x99, 0x02, 0x02, 0xab, 0xcd];

        assert_eq!(to_vec(&value).unwrap(), encoded);
        assert_eq!(value.encoded_length().unwrap(), Length::new(7));
    }

    #[test]
    fn test_decode_codegen() {
        struct Test {
            a: u8,
            b: u16,
        }

        // This is implemented in a way that is conducive to code generation.
        impl Decode for Test {
            #[allow(clippy::match_single_binding)]
            fn decode<R: std::io::Read>(reader: &mut R) -> Result<(Self, usize)> {
                let tlv_decode_read: usize = 0;
                let (a, tlv_decode_read) = match crate::value::decode_tagged(Tag::U8(0x01), reader)?
                {
                    (a, read) => (a, tlv_decode_read + read),
                };
                let (b, tlv_decode_read) = match crate::value::decode_tagged(Tag::U8(0x02), reader)?
                {
                    (b, read) => (b, tlv_decode_read + read),
                };
                Ok((Test { a, b }, tlv_decode_read))
            }
        }

        let data = vec![0x01, 0x01, 0x99, 0x02, 0x02, 0xab, 0xcd];
        let decoded: Test = from_slice(&data).unwrap();
        assert_eq!(decoded.a, 0x99);
        assert_eq!(decoded.b, 0xabcd);
    }

    #[test]
    fn test_medium_length() {
        let tag = Tag::U8(0x01);
        let value = vec![0xbe; 0x81];
        let encoded = to_vec_tagged(tag, &value).unwrap();

        //                        tag    MED   len   data …
        assert_eq!(encoded[0..4], [0x01, 0x81, 0x81, 0xbe]);

        let decoded: Vec<u8> = from_slice_tagged(tag, &encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_long_length() {
        let tag = Tag::U8(0x01);
        let value = vec![0xbe; 0x100];
        let encoded = to_vec_tagged(tag, &value).unwrap();

        //                        tag    LONG  len0  len1  data …
        assert_eq!(encoded[0..5], [0x01, 0x82, 0x01, 0x00, 0xbe]);

        let decoded: Vec<u8> = from_slice_tagged(tag, &encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_boolean() {
        assert_eq!(to_vec(true).unwrap(), [0x01]);
        assert_eq!(to_vec(false).unwrap(), [0x00]);
    }

    #[test]
    fn test_uuid() {
        use uuid::Uuid;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let encoded = to_vec(uuid).unwrap();
        assert_eq!(
            encoded,
            [
                0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
                0x00, 0x00
            ]
        );

        let decoded: Uuid = from_slice(&encoded).unwrap();
        assert_eq!(decoded, uuid);
    }

    proptest! {
        #[test]
        fn test_encode_decode_u8(tag_byte: u8, value: u8) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, value).unwrap();

            let decoded_value: u8 = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_u16(tag_byte: u8, value: u16) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, value).unwrap();

            let decoded_value: u16 = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_u32(tag_byte: u8, value: u32) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, value).unwrap();

            let decoded_value: u32 = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_u64(tag_byte: u8, value: u64) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, value).unwrap();

            let decoded_value: u64 = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_fixed_slice(tag_byte: u8, value: [u8; 100]) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, value).unwrap();

            let decoded_value: [u8; 100] = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_buffer(tag_byte: u8, value: Vec<u8>) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, &value).unwrap();

            let decoded_value: Vec<u8> = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_string(tag_byte: u8, value: String) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, &value).unwrap();

            let decoded_value: String = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }
    }
}
