use std::io::Read;

/// A reader that limits the number of bytes that can be read. This is used to
/// prevent reading more bytes than expected from a stream, such as when reading
/// a variable-length value like a [`String`].
pub(crate) struct LimitedReader<R: Read> {
    reader: R,
    remaining_bytes: usize,
}

impl<R: Read> LimitedReader<R> {
    pub(crate) const fn new(reader: R, max_bytes: usize) -> Self {
        Self {
            reader,
            remaining_bytes: max_bytes,
        }
    }
}

impl<R: Read> Read for LimitedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.remaining_bytes == 0 {
            return Ok(0);
        }

        let buf_len = buf.len();
        let buf = &mut buf[..usize::min(buf_len, self.remaining_bytes)];
        let read = self.reader.read(buf)?;
        self.remaining_bytes -= read;
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use super::*;

    #[test]
    fn read_all_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut reader = LimitedReader::new(&data[..], data.len());
        let mut buf = [0; 5];
        let read = reader.read(&mut buf).unwrap();
        assert_eq!(read, 5);
        assert_eq!(&buf, &data);
    }

    #[test]
    fn read_unlimited_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut reader = LimitedReader::new(&data[..], usize::MAX);
        let mut buf = [0; 5];
        let read = reader.read(&mut buf).unwrap();
        assert_eq!(read, 5);
        assert_eq!(&buf, &data);
    }

    #[test]
    fn read_some_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut reader = LimitedReader::new(&data[..], 3);
        let mut buf = [0; 5];
        let read = reader.read(&mut buf).unwrap();
        assert_eq!(read, 3);
        assert_eq!(&buf[..3], &data[..3]);
        assert_eq!(&buf[3..], &[0, 0]);
    }

    #[test]
    fn read_no_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut reader = LimitedReader::new(&data[..], 0);
        let mut buf = [0; 5];
        let read = reader.read(&mut buf).unwrap();
        assert_eq!(read, 0);
        assert_eq!(&buf, &[0, 0, 0, 0, 0]);
    }

    proptest! {
        #[test]
        fn read_bytes(bytes: Vec<u8>, max_bytes: usize) {
            let mut reader = LimitedReader::new(&bytes[..], max_bytes);
            let mut buf = vec![0; bytes.len()];
            let read = reader.read(&mut buf).unwrap();
            let expected = usize::min(max_bytes, bytes.len());
            assert_eq!(read, expected);
            assert_eq!(&buf[..read], &bytes[..read]);
            buf[read..].iter().for_each(|&b| assert_eq!(b, 0));
        }
    }
}
