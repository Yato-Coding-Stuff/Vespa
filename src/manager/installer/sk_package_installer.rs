use std::path::{Path, PathBuf};

use tempfile::TempDir;
use thiserror::Error;

use crate::{
    packages::SilkSongFlattenedPackage,
    util::{
        context::Context,
        file_handler::{FileHandlerError, recursively_copy_dir, unzip_to_dir},
    },
};

#[derive(Debug, Error)]
pub enum SilkSongPackageInstallerError {
    #[error("The package is already installed")]
    PackageAlreadyInstalled,
    #[error(transparent)]
    FileHandlingError(#[from] FileHandlerError),
    #[error(transparent)]
    VersionParsingError(#[from] semver::Error),
}

pub struct SilkSongPackageInstaller;

impl SilkSongPackageInstaller {
    pub fn new() -> Self {
        SilkSongPackageInstaller
    }

    pub fn install_package(
        &self,
        ctx: &mut Context,
        package: &SilkSongFlattenedPackage,
        dir: &TempDir,
        profile_path: &PathBuf,
    ) -> Result<(), SilkSongPackageInstallerError> {
        let zip_path = dir.path().join("package.zip");
        let unzip_dir = dir.path().join("unzipped");

        unzip_to_dir(&zip_path, &unzip_dir)?;

        let mod_path = profile_path
            .join("BepInEx")
            .join("plugins")
            .join(&package.package_full_name_with_version);
        recursively_copy_dir(&unzip_dir, &mod_path)?;

        ctx.tracker.add(package, &mod_path);

        Ok(())
    }

    pub fn install_bepinex(
        &self,
        ctx: &mut Context,
        package: &SilkSongFlattenedPackage,
        dir: &TempDir,
        bepinex_path: &Path,
    ) -> Result<(), SilkSongPackageInstallerError> {
        let zip_path = dir.path().join("package.zip");
        let unzip_dir = dir.path().join("unzipped");

        unzip_to_dir(&zip_path, &unzip_dir)?;
        let bepinexpack_path = unzip_dir.join("BepInExPack");

        recursively_copy_dir(&bepinexpack_path, bepinex_path)?;

        ctx.tracker.add(package, &bepinex_path);

        Ok(())
    }
}
