use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    cli::presenter::events::{InstallEvent, UninstallEvent, UpdateEvent},
    manager::{
        sk_package_downloader::{SilkSongPackageDownloader, SilkSongPackageDownloaderError},
        sk_package_installer::{SilkSongPackageInstaller, SilkSongPackageInstallerError},
    },
    packages::{SilkSongFlattenedPackage, SilkSongInstalledPackageRecord},
    util::file_handler::delete_dir,
};

#[derive(Debug, Error)]
pub enum SilkSongPackageManagerError {
    #[error(transparent)]
    DownloaderError(#[from] SilkSongPackageDownloaderError),
    #[error(transparent)]
    InstallerError(#[from] SilkSongPackageInstallerError),
    #[error("Package is blacklisted: {0}")]
    PackageBlacklisted(String),
}

pub struct SilkSongPackageManager {
    pub downloader: SilkSongPackageDownloader,
    pub installer: SilkSongPackageInstaller,
}

impl SilkSongPackageManager {
    pub fn new() -> Self {
        SilkSongPackageManager {
            downloader: SilkSongPackageDownloader::new(),
            installer: SilkSongPackageInstaller::new(),
        }
    }

    pub fn install_package<F: FnMut(InstallEvent)>(
        &self,
        ctx: &mut crate::util::context::Context,
        package: &SilkSongFlattenedPackage,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<(), SilkSongPackageManagerError> {
        if ctx.black_list.contains(&package.package_full_name.as_str()) {
            return Err(SilkSongPackageManagerError::PackageBlacklisted(
                package.package_full_name_with_version.clone(),
            ));
        }

        if let Some(installed) = ctx.tracker.get(&package.package_full_name) {
            let _ = delete_dir(&installed.file_path);
        }

        progress(InstallEvent::DownloadingMod {
            name: package.package_full_name_with_version.clone(),
        });
        let zip_dir = self.downloader.download(&package.download_url, progress)?;

        progress(InstallEvent::FinishedDownloadingMod {
            name: package.package_full_name_with_version.clone(),
        });
        progress(InstallEvent::InstallingMod {
            name: package.package_full_name_with_version.clone(),
        });
        match self
            .installer
            .install_package(ctx, package, &zip_dir, profile_path)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn install_bepinex<F: FnMut(InstallEvent)>(
        &self,
        ctx: &mut crate::util::context::Context,
        package: &SilkSongFlattenedPackage,
        progress: &mut F,
        bepinex_path: &Path,
    ) -> Result<(), SilkSongPackageManagerError> {
        let zip_dir = self.downloader.download(&package.download_url, progress)?;
        progress(InstallEvent::FinishedDownloadingMod {
            name: package.package_full_name_with_version.clone(),
        });
        progress(InstallEvent::InstallingMod {
            name: package.package_full_name_with_version.clone(),
        });
        match self
            .installer
            .install_bepinex(ctx, package, &zip_dir, bepinex_path)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn uninstall_package<F: FnMut(UninstallEvent)>(
        &self,
        ctx: &mut crate::util::context::Context,
        package: &SilkSongInstalledPackageRecord,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<(), SilkSongPackageManagerError> {
        if ctx.black_list.contains(&package.package_full_name.as_str()) {
            return Err(SilkSongPackageManagerError::PackageBlacklisted(
                package.package_full_name_with_version.clone(),
            ));
        }

        progress(UninstallEvent::UninstallingMod {
            name: package.package_full_name_with_version.clone(),
        });

        match self.installer.uninstall_package(ctx, package) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_package<F: FnMut(UpdateEvent)>(
        &self,
        ctx: &mut crate::util::context::Context,
        package: &SilkSongFlattenedPackage,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<(), SilkSongPackageManagerError> {
        if ctx.black_list.contains(&package.package_full_name.as_str()) {
            return Err(SilkSongPackageManagerError::PackageBlacklisted(
                package.package_full_name_with_version.clone(),
            ));
        }

        if let Some(installed) = ctx.tracker.get(&package.package_full_name) {
            progress(UpdateEvent::CleaningUpOldMod {
                name: package.package_full_name.clone(),
            });
            let _ = delete_dir(&installed.file_path);
        }

        progress(UpdateEvent::DownloadingMod {
            name: package.package_full_name_with_version.clone(),
        });

        let zip_dir = self
            .downloader
            .download(&package.download_url, &mut |_| {})?;

        progress(UpdateEvent::InstallingMod {
            name: package.package_full_name_with_version.clone(),
        });

        self.installer
            .install_package(ctx, package, &zip_dir, profile_path)?;

        Ok(())
    }
}
