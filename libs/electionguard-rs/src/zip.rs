use std::{
    fs::{read_dir, File},
    io,
    path::PathBuf,
};

use zip::{result::ZipResult, ZipArchive};

/// Zip all files directly within a directory into a zip archive. This function
/// does not recursively zip files in subdirectories, and it does not include
/// the directory itself in the archive.
pub(crate) fn zip_files_in_directory_to_buffer(directory: &PathBuf) -> io::Result<Vec<u8>> {
    let mut zip_buffer = Vec::new();
    let writer = std::io::Cursor::new(&mut zip_buffer);
    let mut zip = zip::ZipWriter::new(writer);

    zip_files_in_directory(&mut zip, directory)?;

    zip.finish()?;

    // `zip` is holding on to `writer` which is holding on to `zip_buffer`,
    // so we need to drop `zip` to release the borrow on `zip_buffer`.
    drop(zip);

    Ok(zip_buffer)
}

/// Zip all files directly within a directory into a zip archive. This function
/// does not recursively zip files in subdirectories, and it does not include
/// the directory itself in the archive.
pub(crate) fn zip_files_in_directory<W>(
    zip: &mut zip::ZipWriter<W>,
    directory: &PathBuf,
) -> io::Result<()>
where
    W: io::Write + io::Seek,
{
    for entry in read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(&directory)
            .expect("entry must be in output directory");

        if path.is_file() {
            zip.start_file(
                name.to_str().expect("entry must have valid UTF-8 name"),
                Default::default(),
            )?;
            let mut file = File::open(&path)?;
            std::io::copy(&mut file, zip)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct UnzipLimits {
    file_count: usize,
    bytes: u64,
}

impl Default for UnzipLimits {
    fn default() -> Self {
        Self {
            file_count: 1000,
            bytes: 100 * 1024 * 1024, // 100 MB
        }
    }
}

pub(crate) fn unzip_into_directory<R>(
    zip: &mut ZipArchive<R>,
    directory: &PathBuf,
    limits: UnzipLimits,
) -> ZipResult<()>
where
    R: io::Read + io::Seek,
{
    if zip.len() > limits.file_count {
        return Err(zip::result::ZipError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Too many files in zip archive, limit is {}",
                limits.file_count
            ),
        )));
    }

    let mut total_extracted_file_size = 0u64;

    for i in 0..zip.len() {
        total_extracted_file_size += zip.by_index(i)?.size();

        if total_extracted_file_size > limits.bytes {
            return Err(zip::result::ZipError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Zip file is too large to extract safely, limit is {} bytes",
                    limits.bytes
                ),
            )));
        }
    }

    zip.extract(directory)?;

    Ok(())
}
