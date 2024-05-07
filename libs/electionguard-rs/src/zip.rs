use std::{
    fs::{read_dir, File},
    io,
    path::PathBuf,
};

use zip::{result::ZipResult, ZipArchive};

/// Zip all files directly within a directory into a zip archive. The root of
/// the archive will be the directory itself, and the files will be stored at
/// the root of the archive.
pub(crate) fn zip_files_in_directory_to_buffer(
    directory: &PathBuf,
    options: impl AsRef<ZipOptions>,
) -> io::Result<Vec<u8>> {
    let mut zip_buffer = Vec::new();
    let writer = std::io::Cursor::new(&mut zip_buffer);
    let mut zip = zip::ZipWriter::new(writer);

    zip_files_in_directory(&mut zip, directory, None, options)?;

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
    prefix: Option<&str>,
    options: impl AsRef<ZipOptions>,
) -> io::Result<()>
where
    W: io::Write + io::Seek,
{
    let options = options.as_ref();

    for entry in read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(directory)
            .expect("entry must be in output directory");
        let name = name.to_str().expect("entry must have valid UTF-8 name");
        let zip_path = match prefix {
            Some(prefix) => format!("{prefix}/{name}"),
            None => name.to_owned(),
        };

        if path.is_file() {
            zip.start_file(zip_path, Default::default())?;
            let mut file = File::open(&path)?;
            std::io::copy(&mut file, zip)?;
        } else if path.is_dir() {
            if options.recursion_depth == 0 {
                tracing::warn!(
                    "Skipping directory '{}' because recursion depth is 0",
                    path.display()
                );
            } else {
                zip_files_in_directory(
                    zip,
                    &path,
                    Some(&zip_path),
                    ZipOptions {
                        recursion_depth: options.recursion_depth - 1,
                    },
                )?;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ZipOptions {
    pub(crate) recursion_depth: usize,
}

impl AsRef<ZipOptions> for ZipOptions {
    fn as_ref(&self) -> &ZipOptions {
        self
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{Read, Write};

    fn create_test_directory_tree() -> io::Result<tempfile::TempDir> {
        let temp_dir = tempfile::tempdir()?;
        let temp_dir_path = temp_dir.path();

        let file1_path = temp_dir_path.join("file1.txt");
        let mut file1 = File::create(file1_path)?;
        writeln!(file1, "file1")?;

        let file2_path = temp_dir_path.join("file2.txt");
        let mut file2 = File::create(file2_path)?;
        writeln!(file2, "file2")?;

        let sub_dir_path = temp_dir_path.join("sub_dir");
        std::fs::create_dir(&sub_dir_path)?;

        let sub_file_path = sub_dir_path.join("sub_file.txt");
        let mut sub_file = File::create(sub_file_path)?;
        writeln!(sub_file, "sub_file")?;

        let sub_sub_dir_path = sub_dir_path.join("sub_sub_dir");
        std::fs::create_dir(&sub_sub_dir_path)?;

        let sub_sub_file_path = sub_sub_dir_path.join("sub_sub_file.txt");
        let mut sub_sub_file = File::create(sub_sub_file_path)?;
        writeln!(sub_sub_file, "sub_sub_file")?;

        Ok(temp_dir)
    }

    #[test]
    fn test_zip_files_in_directory_no_recursion() {
        let temp_dir = create_test_directory_tree().unwrap();
        let temp_dir_path = temp_dir.path();

        let zip_buffer = zip_files_in_directory_to_buffer(
            &temp_dir_path.to_path_buf(),
            ZipOptions { recursion_depth: 0 },
        )
        .unwrap();

        let reader = std::io::Cursor::new(zip_buffer);
        let mut zip = zip::ZipArchive::new(reader).unwrap();

        assert_eq!(zip.len(), 2);

        let mut file1 = zip.by_name("file1.txt").unwrap();
        let mut file1_contents = String::new();
        file1.read_to_string(&mut file1_contents).unwrap();
        assert_eq!(file1_contents, "file1\n");
        drop(file1);

        let mut file2 = zip.by_name("file2.txt").unwrap();
        let mut file2_contents = String::new();
        file2.read_to_string(&mut file2_contents).unwrap();
        assert_eq!(file2_contents, "file2\n");
        drop(file2);
    }

    #[test]
    fn test_zip_files_in_directory_depth_1() {
        let temp_dir = create_test_directory_tree().unwrap();
        let temp_dir_path = temp_dir.path();

        let zip_buffer = zip_files_in_directory_to_buffer(
            &temp_dir_path.to_path_buf(),
            ZipOptions { recursion_depth: 1 },
        )
        .unwrap();

        let reader = std::io::Cursor::new(zip_buffer);
        let mut zip = zip::ZipArchive::new(reader).unwrap();

        assert_eq!(zip.len(), 3);

        let mut file1 = zip.by_name("file1.txt").unwrap();
        let mut file1_contents = String::new();
        file1.read_to_string(&mut file1_contents).unwrap();
        assert_eq!(file1_contents, "file1\n");
        drop(file1);

        let mut file2 = zip.by_name("file2.txt").unwrap();
        let mut file2_contents = String::new();
        file2.read_to_string(&mut file2_contents).unwrap();
        assert_eq!(file2_contents, "file2\n");
        drop(file2);

        let mut sub_file = zip.by_name("sub_dir/sub_file.txt").unwrap();
        let mut sub_file_contents = String::new();
        sub_file.read_to_string(&mut sub_file_contents).unwrap();
        assert_eq!(sub_file_contents, "sub_file\n");
        drop(sub_file);
    }

    #[test]
    fn test_zip_files_in_directory_depth_2() {
        let temp_dir = create_test_directory_tree().unwrap();
        let temp_dir_path = temp_dir.path();

        let zip_buffer = zip_files_in_directory_to_buffer(
            &temp_dir_path.to_path_buf(),
            ZipOptions { recursion_depth: 2 },
        )
        .unwrap();

        let reader = std::io::Cursor::new(zip_buffer);
        let mut zip = zip::ZipArchive::new(reader).unwrap();

        assert_eq!(zip.len(), 4);

        let mut file1 = zip.by_name("file1.txt").unwrap();
        let mut file1_contents = String::new();
        file1.read_to_string(&mut file1_contents).unwrap();
        assert_eq!(file1_contents, "file1\n");
        drop(file1);

        let mut file2 = zip.by_name("file2.txt").unwrap();
        let mut file2_contents = String::new();
        file2.read_to_string(&mut file2_contents).unwrap();
        assert_eq!(file2_contents, "file2\n");
        drop(file2);

        let mut sub_file = zip.by_name("sub_dir/sub_file.txt").unwrap();
        let mut sub_file_contents = String::new();
        sub_file.read_to_string(&mut sub_file_contents).unwrap();
        assert_eq!(sub_file_contents, "sub_file\n");
        drop(sub_file);

        let mut sub_sub_file = zip.by_name("sub_dir/sub_sub_dir/sub_sub_file.txt").unwrap();
        let mut sub_sub_file_contents = String::new();
        sub_sub_file
            .read_to_string(&mut sub_sub_file_contents)
            .unwrap();
        assert_eq!(sub_sub_file_contents, "sub_sub_file\n");
        drop(sub_sub_file);
    }
}
