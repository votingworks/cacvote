use crate::{Decode, Encode, Length, Tag};

/// Encodes a full TLV value into a writer.
pub fn encode_tagged<E, W>(tag: Tag, value: &E, writer: &mut W) -> std::io::Result<()>
where
    E: Encode,
    W: std::io::Write,
{
    tag.encode(writer)?;
    let length = value.length()?;
    length.encode(writer)?;
    value.encode(writer)
}

/// Computes the full TLV length for a tagged value.
pub fn length_tagged<E>(tag: Tag, value: &E) -> std::io::Result<Length>
where
    E: Encode,
{
    let length = value.length()?;
    Ok(tag.length()? + length.length()? + length)
}

pub fn decode_tagged<D, R>(tag: Tag, reader: &mut R) -> std::io::Result<(D, usize)>
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
        return Err(std::io::ErrorKind::InvalidData.into());
    }

    let length = Length::from_reader(reader)?;
    let (value, value_size) = D::decode(reader)?;

    if value_size != length.value() as usize {
        return Err(std::io::ErrorKind::InvalidData.into());
    }

    Ok((
        value,
        tag_bytes.len() + length.length()?.value() as usize + value_size,
    ))
}
