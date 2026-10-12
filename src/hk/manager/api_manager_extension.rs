use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::base::{
    cli::presenter::{events::InstallEvent, presenter::Presenter},
    manager::{
        package_installer::PackageInstallerError,
        package_manager::{PackageManager, PackageManagerError},
    },
    packages::{InstalledPackageRecord, PackageRecord},
    util::{
        context::Context,
        file_handler::{delete_dir, recursively_copy_dir, unzip_to_dir},
    },
};

pub(crate) const HK_API_PACKAGE_NAME: &str = "HollowKnight.ModdingApi";

pub(crate) fn update_hk_api_package_layout(
    package_manager: &PackageManager,
    ctx: &mut Context,
    package: &PackageRecord,
    presenter: &mut Presenter,
    profile_path: &PathBuf,
) -> Result<bool, PackageManagerError> {
    if package.name != HK_API_PACKAGE_NAME {
        return Ok(false);
    }

    let progress = &mut |event: InstallEvent| {
        presenter.display(&event);
    };
    package_manager.install_hk_api(ctx, package, progress, profile_path)?;
    Ok(true)
}

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
        let zip_dir = self.downloader.download(&package.download_url, progress)?;
        progress(InstallEvent::FinishedDownloadingMod {
            name: package.identifier.clone(),
        });
        progress(InstallEvent::InstallingMod {
            name: package.identifier.clone(),
        });

        unpack_hk_api(ctx, package, &zip_dir, profile_path)?;
        Ok(())
    }
}

fn unpack_hk_api(
    ctx: &mut Context,
    package: &PackageRecord,
    dir: &TempDir,
    profile_path: &Path,
) -> Result<(), PackageInstallerError> {
    let zip_path = dir.path().join("package.zip");
    let unzip_dir = dir.path().join("unzipped");
    let api_path = profile_path.join("api");

    if let Some(installed) = ctx.tracker.get(&package.name) {
        let _ = delete_dir(&installed.file_path);
    }

    unzip_to_dir(&zip_path, &unzip_dir)?;
    recursively_copy_dir(&unzip_dir, &api_path)?;

    ctx.tracker.insert(InstalledPackageRecord {
        name: package.name.clone(),
        identifier: package.identifier.clone(),
        version: Some(package.version.clone()),
        file_path: api_path,
    });

    Ok(())
}
