pub mod coder;
pub mod length;
pub mod tag;
pub mod value;

pub use coder::{Decode, Encode};
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
/// # use tlv::{Encode, to_vec};
/// #
/// # fn main() -> std::io::Result<()> {
/// let value: u16 = 0xbeef;
/// let encoded = to_vec(value)?;
/// assert_eq!(encoded, [0xbe, 0xef]);
/// # Ok(())
/// # }
/// ```
///
/// Most of the time you'll want to use this with a struct from `tlv-derive`:
///
/// ```ignore
/// # use tlv::{Encode, to_vec};
/// # use tlv_derive::Encode;
/// #
/// #[derive(Encode)]
/// struct Test {
///     #[tlv(tag = 0x01)]
///     a: u8,
///     #[tlv(tag = 0x02)]
///     b: u16,
/// }
///
/// # fn main() -> std::io::Result<()> {
/// let value = Test { a: 0x99, b: 0xabcd };
/// let encoded = to_vec(&value)?;
/// assert_eq!(encoded, [
///     0x01, // `a` tag
///     0x01, // `a` length
///     0x99, // `a` value
///     0x02, // `b` tag
///     0x02, // `b` length
///     0xab, // `b` value
///     0xcd  // `b` value
/// ]);
/// # Ok(())
/// # }
/// ```
pub fn to_vec<E>(value: E) -> std::io::Result<Vec<u8>>
where
    E: Encode,
{
    let mut buffer = Vec::new();
    value.encode(&mut buffer)?;
    assert_eq!(buffer.len(), value.length()?.value() as usize);
    Ok(buffer)
}

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
/// # use tlv::{Encode, to_vec};
/// #
/// # fn main() -> std::io::Result<()> {
/// let value: u16 = 0xbeef;
/// let encoded = to_vec(value)?;
/// assert_eq!(encoded, [0xbe, 0xef]);
/// # Ok(())
/// # }
/// ```
///
/// Most of the time you'll want to use this with a struct from `tlv-derive`:
///
/// ```ignore
/// # use tlv::{Encode, to_vec};
/// # use tlv_derive::Encode;
/// #
/// #[derive(Encode)]
/// struct Test {
///     #[tlv(tag = 0x01)]
///     a: u8,
///     #[tlv(tag = 0x02)]
///     b: u16,
/// }
///
/// # fn main() -> std::io::Result<()> {
/// let value = Test { a: 0x99, b: 0xabcd };
/// let encoded = to_vec(&value)?;
/// assert_eq!(encoded, [
///     0x01, // `a` tag
///     0x01, // `a` length
///     0x99, // `a` value
///     0x02, // `b` tag
///     0x02, // `b` length
///     0xab, // `b` value
///     0xcd  // `b` value
/// ]);
/// # Ok(())
/// # }
/// ```
pub fn to_vec_tagged<E>(tag: Tag, value: &E) -> std::io::Result<Vec<u8>>
where
    E: Encode,
{
    let mut buffer = Vec::new();
    value::encode_tagged(tag, value, &mut buffer)?;
    Ok(buffer)
}

/// Decodes a value from a slice.
pub fn from_slice<D>(buffer: &[u8]) -> std::io::Result<D>
where
    D: Decode,
{
    let mut cursor = std::io::Cursor::new(buffer);
    let (value, read) = D::decode(&mut cursor)?;

    if read != buffer.len() {
        Err(std::io::ErrorKind::InvalidData.into())
    } else {
        Ok(value)
    }
}

/// Decodes a tagged value from a slice.
pub fn from_slice_tagged<D>(tag: Tag, buffer: &[u8]) -> std::io::Result<D>
where
    D: Decode,
{
    let mut cursor = std::io::Cursor::new(buffer);
    let (value, read) = value::decode_tagged(tag, &mut cursor)?;

    if read != buffer.len() {
        Err(std::io::ErrorKind::InvalidData.into())
    } else {
        Ok(value)
    }
}

/// Decodes a tagged value from a reader, returning the value and the number of
/// bytes read.
pub fn from_reader_tagged<D, R>(tag: Tag, reader: &mut R) -> std::io::Result<(D, usize)>
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
        Encode, Length,
    };

    #[test]
    fn test_full_tlv_tag_mismatch() {
        let value: u16 = 0xbeef;
        let encoded = to_vec_tagged(Tag::U16(0x0102), &value).unwrap();

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
    fn test_encode_codegen() {
        struct Test {
            a: u8,
            b: u16,
        }

        // This is implemented in a way that is conducive to code generation.
        impl Encode for Test {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                crate::value::encode_tagged(Tag::U8(0x01), &self.a, writer)?;
                crate::value::encode_tagged(Tag::U8(0x02), &self.b, writer)?;
                Ok(())
            }

            fn length(&self) -> std::io::Result<crate::Length> {
                let mut length = Length::new(0);

                length += Tag::U8(0x01).length()?;
                length += self.a.length()?.length()?;
                length += self.a.length()?;

                length += Tag::U8(0x02).length()?;
                length += self.b.length()?.length()?;
                length += self.b.length()?;

                Ok(length)
            }
        }

        let value = Test { a: 0x99, b: 0xabcd };
        let encoded = [0x01, 0x01, 0x99, 0x02, 0x02, 0xab, 0xcd];

        assert_eq!(to_vec(&value).unwrap(), encoded);
        assert_eq!(value.length().unwrap(), Length::new(7));
    }

    #[test]
    fn test_decode_codegen() {
        struct Test {
            a: u8,
            b: u16,
        }

        // This is implemented in a way that is conducive to code generation.
        impl Decode for Test {
            fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<(Self, usize)> {
                let tlv_decode_read: usize = 0;

                let tlv_decode_read: usize = {
                    let tag_bytes = Tag::U8(0x01).to_vec();
                    let read_tag_bytes = {
                        let mut buf = vec![0; tag_bytes.len()];
                        reader.read_exact(&mut buf)?;
                        buf
                    };

                    if tag_bytes != read_tag_bytes {
                        return Err(std::io::ErrorKind::InvalidData.into());
                    }

                    tlv_decode_read + tag_bytes.len()
                };
                let tlv_decode_length = Length::from_reader(reader)?;
                let tlv_decode_read =
                    tlv_decode_read + tlv_decode_length.length()?.value() as usize;
                let (a, tlv_decode_read) = match <_ as Decode>::decode(reader)? {
                    (a, read) => {
                        if read != tlv_decode_length.value() as usize {
                            return Err(std::io::ErrorKind::InvalidData.into());
                        }
                        (a, tlv_decode_read + read)
                    }
                };

                let tlv_decode_read: usize = {
                    let tag_bytes = Tag::U8(0x02).to_vec();
                    let read_tag_bytes = {
                        let mut buf = vec![0; tag_bytes.len()];
                        reader.read_exact(&mut buf)?;
                        buf
                    };

                    if tag_bytes != read_tag_bytes {
                        return Err(std::io::ErrorKind::InvalidData.into());
                    }

                    tlv_decode_read + tag_bytes.len()
                };
                let tlv_decode_length = Length::from_reader(reader)?;
                let tlv_decode_read =
                    tlv_decode_read + tlv_decode_length.length()?.value() as usize;
                let (b, tlv_decode_read) = match <_ as Decode>::decode(reader)? {
                    (b, read) => {
                        if read != tlv_decode_length.value() as usize {
                            return Err(std::io::ErrorKind::InvalidData.into());
                        }
                        (b, tlv_decode_read + read)
                    }
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

    proptest! {
        #[test]
        fn test_encode_decode_u8(tag_byte: u8, value: u8) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, &value).unwrap();

            let decoded_value: u8 = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_u16(tag_byte: u8, value: u16) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, &value).unwrap();

            let decoded_value: u16 = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_u32(tag_byte: u8, value: u32) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, &value).unwrap();

            let decoded_value: u32 = from_slice_tagged(tag, &encoded).unwrap();
            assert_eq!(decoded_value, value);
        }

        #[test]
        fn test_encode_decode_u64(tag_byte: u8, value: u64) {
            let tag = Tag::U8(tag_byte);
            let encoded = to_vec_tagged(tag, &value).unwrap();

            let decoded_value: u64 = from_slice_tagged(tag, &encoded).unwrap();
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
