use std::{
    fs::{self, File},
    io::{self, copy},
    path::Path,
};

use tempfile::TempDir;
use thiserror::Error;
use zip::ZipArchive;

use crate::{packages::sk_package::SilkSongPackage, util::context::Context};

#[derive(Debug, Error)]
pub enum SilkSongPackageInstallerError {
    #[error("The Package is already installed")]
    PackageAlreadyInstalled,
    #[error("Failed to write to file: {0}")]
    WriteError(#[from] std::io::Error),
}

pub struct SilkSongPackageInstaller;

fn unzip_to_dir(zip_path: &Path, dest_dir: &Path) -> Result<(), std::io::Error> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.name());

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn recursively_copy_dir(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            recursively_copy_dir(&path, &dest_path)?;
        } else {
            let mut src_file = File::open(&path)?;
            let mut dest_file = File::create(&dest_path)?;
            io::copy(&mut src_file, &mut dest_file)?;
        }
    }
    Ok(())
}

impl SilkSongPackageInstaller {
    pub fn new() -> SilkSongPackageInstaller {
        SilkSongPackageInstaller
    }

    pub fn install_package(
        &self,
        context: &Context,
        package: SilkSongPackage,
        dir: TempDir,
    ) -> Result<(), SilkSongPackageInstallerError> {
        if context.tracker.get_package(&package.name).is_some() {
            return Err(SilkSongPackageInstallerError::PackageAlreadyInstalled);
        }

        let zip_path = dir.path().join("package.zip");
        let unzip_dir = dir.path().join("unzipped");

        unzip_to_dir(&zip_path, &unzip_dir)?;

        let mod_path = context.config.silk_song_path.join("data");
        recursively_copy_dir(&unzip_dir, &mod_path)?;

        Ok(())
    }
}
