use std::path::Path;

use tempfile::TempDir;

use crate::base::{
    cli::presenter::events::InstallEvent,
    manager::{
        package_installer::PackageInstallerError,
        package_manager::{PackageManager, PackageManagerError},
    },
    packages::{InstalledPackageRecord, PackageRecord},
    util::{
        context::Context,
        file_handler::{recursively_copy_dir, unzip_to_dir},
    },
};

pub(crate) trait BepInExPackageManagerExt {
    fn install_bepinex(
        &self,
        ctx: &mut Context,
        package: &PackageRecord,
        progress: &mut dyn FnMut(InstallEvent),
        bepinex_path: &Path,
    ) -> Result<(), PackageManagerError>;
}

impl BepInExPackageManagerExt for PackageManager {
    fn install_bepinex(
        &self,
        ctx: &mut Context,
        package: &PackageRecord,
        progress: &mut dyn FnMut(InstallEvent),
        bepinex_path: &Path,
    ) -> Result<(), PackageManagerError> {
        let zip_dir = self.downloader.download(&package.download_url, progress)?;
        progress(InstallEvent::FinishedDownloadingMod {
            name: package.identifier.clone(),
        });
        progress(InstallEvent::InstallingMod {
            name: package.identifier.clone(),
        });
        match unpack_bepinex(ctx, package, &zip_dir, bepinex_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

fn unpack_bepinex(
    ctx: &mut Context,
    package: &PackageRecord,
    dir: &TempDir,
    bepinex_path: &Path,
) -> Result<(), PackageInstallerError> {
    let zip_path = dir.path().join("package.zip");
    let unzip_dir = dir.path().join("unzipped");

    unzip_to_dir(&zip_path, &unzip_dir)?;
    let bepinexpack_path = unzip_dir.join("BepInExPack");

    recursively_copy_dir(&bepinexpack_path, bepinex_path)?;

    ctx.tracker.insert(InstalledPackageRecord {
        name: package.name.clone(),
        identifier: package.identifier.clone(),
        version: Some(package.version.clone()),
        file_path: bepinex_path.to_path_buf(),
    });

    Ok(())
}
