use std::{
    fs::{self, File},
    io::{self, copy},
    path::Path,
};
use thiserror::Error;
use zip::{ZipArchive, result::ZipError};

#[derive(Debug, Error)]
pub enum FileHandlerError {
    #[error("Failed to open zip archive: {0}")]
    OpenZipError(#[source] io::Error),

    #[error("Failed to read zip archive entry: {0}")]
    ReadZipEntryError(#[source] ZipError),

    #[error("Failed to create output directory for zip extraction: {0}")]
    CreateZipDirError(#[source] io::Error),

    #[error("Failed to create file during zip extraction: {0}")]
    CreateZipFileError(#[source] io::Error),

    #[error("Failed to copy from zip entry to file: {0}")]
    CopyZipFileError(#[source] io::Error),

    #[error("Failed to create directory when copying: {0}")]
    CreateDirError(#[source] io::Error),

    #[error("Failed to delete directory: {0}")]
    DeleteDirError(#[source] io::Error),

    #[error("Failed to copy file: {0}")]
    CopyFileError(#[source] io::Error),
}

/// Unzips a ZIP archive into a folder, giving unique errors for each failure
pub fn unzip_to_dir(zip_path: &Path, dest_dir: &Path) -> Result<(), FileHandlerError> {
    let file = File::open(zip_path).map_err(FileHandlerError::OpenZipError)?;
    let mut archive = ZipArchive::new(file).map_err(FileHandlerError::ReadZipEntryError)?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(FileHandlerError::ReadZipEntryError)?;
        let outpath = dest_dir.join(file.name());

        if file.is_dir() {
            fs::create_dir_all(&outpath).map_err(FileHandlerError::CreateZipDirError)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(FileHandlerError::CreateZipDirError)?;
            }
            let mut outfile =
                File::create(&outpath).map_err(FileHandlerError::CreateZipFileError)?;
            copy(&mut file, &mut outfile).map_err(FileHandlerError::CopyZipFileError)?;
        }
    }
    Ok(())
}

pub fn delete_dir(dir: &Path) -> Result<(), FileHandlerError> {
    fs::remove_dir_all(dir).map_err(FileHandlerError::DeleteDirError)?;
    Ok(())
}

pub fn recursively_copy_dir(src: &Path, dst: &Path) -> Result<(), FileHandlerError> {
    fs::create_dir_all(dst).map_err(FileHandlerError::CreateDirError)?;
    for entry in fs::read_dir(src).map_err(FileHandlerError::CreateDirError)? {
        let entry = entry.map_err(FileHandlerError::CreateDirError)?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            recursively_copy_dir(&path, &dest_path)?;
        } else {
            let mut src_file = File::open(&path).map_err(FileHandlerError::CopyFileError)?;
            let mut dest_file =
                File::create(&dest_path).map_err(FileHandlerError::CopyFileError)?;
            copy(&mut src_file, &mut dest_file).map_err(FileHandlerError::CopyFileError)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{delete_dir, recursively_copy_dir, unzip_to_dir};
    use std::{fs, io::Write};
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    #[test]
    fn recursively_copy_dir_copies_nested_files() {
        let temp_dir = tempdir().unwrap();
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::write(src.join("root.txt"), "root").unwrap();
        fs::write(src.join("nested").join("child.txt"), "child").unwrap();

        recursively_copy_dir(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("root.txt")).unwrap(), "root");
        assert_eq!(
            fs::read_to_string(dst.join("nested").join("child.txt")).unwrap(),
            "child"
        );
    }

    #[test]
    fn delete_dir_removes_directory_tree() {
        let temp_dir = tempdir().unwrap();
        let dir = temp_dir.path().join("to-delete");
        fs::create_dir_all(dir.join("nested")).unwrap();

        delete_dir(&dir).unwrap();

        assert!(!dir.exists());
    }

    #[test]
    fn unzip_to_dir_extracts_files_and_directories() {
        let temp_dir = tempdir().unwrap();
        let zip_path = temp_dir.path().join("archive.zip");
        let dest_dir = temp_dir.path().join("unzipped");

        let zip_file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();
        zip.add_directory("folder/", options).unwrap();
        zip.start_file("folder/file.txt", options).unwrap();
        zip.write_all(b"hello world").unwrap();
        zip.finish().unwrap();

        unzip_to_dir(&zip_path, &dest_dir).unwrap();

        assert_eq!(
            fs::read_to_string(dest_dir.join("folder").join("file.txt")).unwrap(),
            "hello world"
        );
    }
}
