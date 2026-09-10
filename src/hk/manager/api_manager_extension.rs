use std::path::Path;

use tempfile::TempDir;

use crate::base::{
    cli::presenter::events::InstallEvent,
    manager::package_manager::{PackageManager, PackageManagerError},
    packages::{InstalledPackageRecord, PackageRecord},
    util::{context::Context, file_handler::unzip_to_dir},
};

pub(crate) const HK_API_PACKAGE_NAME: &str = "HollowKnight.ModdingApi";

pub(crate) trait HkApiPackageManagerExt {
    fn install_hk_api(
        &self,
        ctx: &mut Context,
        package: &PackageRecord,
        progress: &mut dyn FnMut(InstallEvent),
        profile_path: &Path,
    ) -> Result<(), PackageManagerError>;
}

impl HkApiPackageManagerExt for PackageManager {
    fn install_hk_api(
        &self,
        ctx: &mut Context,
        package: &PackageRecord,
        progress: &mut dyn FnMut(InstallEvent),
        profile_path: &Path,
    ) -> Result<(), PackageManagerError> {
        progress(InstallEvent::DownloadingMod {
            name: package.identifier.clone(),
        });
        let zip_dir = self.download_archive(&package.download_url, progress)?;
        progress(InstallEvent::FinishedDownloadingMod {
            name: package.identifier.clone(),
        });
        progress(InstallEvent::InstallingMod {
            name: package.identifier.clone(),
        });

        unpack_hk_api(self, ctx, package, &zip_dir, profile_path)?;
        Ok(())
    }
}

fn unpack_hk_api(
    manager: &PackageManager,
    ctx: &mut Context,
    package: &PackageRecord,
    dir: &TempDir,
    profile_path: &Path,
) -> Result<(), PackageManagerError> {
    let zip_path = dir.path().join("package.zip");
    let unzip_dir = dir.path().join("unzipped");
    let api_path = profile_path.join("api");

    unzip_to_dir(&zip_path, &unzip_dir)?;
    let previous = ctx
        .tracker
        .get(&package.name)
        .map(|record| record.file_path.clone());
    manager.commit_installation(&unzip_dir, &api_path, previous.as_deref())?;
    manager.write_version_marker(&api_path, &package.version)?;

    ctx.tracker.insert(InstalledPackageRecord {
        name: package.name.clone(),
        identifier: package.identifier.clone(),
        version: Some(package.version.clone()),
        file_path: api_path,
    });

    Ok(())
}
