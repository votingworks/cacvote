use crate::{limited_reader::LimitedReader, Decode, Encode, Error, Length, Result, Tag};

/// Encodes a full TLV value into a writer.
pub fn encode_tagged<E, W>(tag: Tag, value: E, writer: &mut W) -> Result<()>
where
    E: Encode,
    W: std::io::Write,
{
    tag.encode(writer)?;
    let length = value.encoded_length()?;
    length.encode(writer)?;
    value.encode(writer)
}

/// Computes the full TLV length for a tagged value.
pub fn length_tagged<E>(tag: Tag, value: E) -> Result<Length>
where
    E: Encode,
{
    let length = value.encoded_length()?;
    Ok(tag.encoded_length()? + length.encoded_length()? + length)
}

pub fn decode_tagged<D, R>(tag: Tag, reader: &mut R) -> Result<(D, usize)>
where
    D: Decode,
    R: std::io::Read,
{
    let tag_bytes = tag.to_vec();
    let read_tag_bytes = {
        let mut buf = vec![0; tag_bytes.len()];
        reader.read_exact(&mut buf)?;
        buf
    };

    if tag_bytes != read_tag_bytes {
        return Err(Error::InvalidTag {
            expected: tag_bytes,
            actual: read_tag_bytes,
        });
    }

    let length = Length::from_reader(reader)?;
    let (value, value_size) = D::decode(&mut LimitedReader::new(reader, length.value().into()))?;

    let expected = usize::from(length.value());
    let actual = value_size;

    if actual != expected {
        return Err(Error::InvalidLength { expected, actual });
    }

    Ok((
        value,
        tag_bytes.len() + length.encoded_length()?.value() as usize + value_size,
    ))
}
