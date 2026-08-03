use std::path::PathBuf;

use thiserror::Error;

use crate::base::{
    cli::presenter::events::{DisableEnableEvent, InstallEvent, UninstallEvent, UpdateEvent},
    manager::{
        package_downloader::{PackageDownloader, PackageDownloaderError},
        package_installer::{PackageInstaller, PackageInstallerError},
    },
    packages::{PackageRecord, package::InstalledPackageRecord},
    util::{context::Context, file_handler::delete_dir},
};

#[derive(Debug, Error)]
pub enum PackageManagerError {
    #[error(transparent)]
    DownloaderError(#[from] PackageDownloaderError),
    #[error(transparent)]
    InstallerError(#[from] PackageInstallerError),
    #[error("Package is blacklisted: {0}")]
    PackageBlacklisted(String),
}

pub struct PackageManager {
    pub downloader: PackageDownloader,
    pub installer: PackageInstaller,
    pub blacklist: &'static [&'static str],
}

impl PackageManager {
    pub fn new(blacklist: &'static [&'static str]) -> Self {
        Self {
            downloader: PackageDownloader::new(),
            installer: PackageInstaller::new(),
            blacklist,
        }
    }

    pub fn install_package(
        &self,
        ctx: &mut Context,
        package: &PackageRecord,
        progress: &mut dyn FnMut(InstallEvent),
        profile_path: &PathBuf,
    ) -> Result<(), PackageManagerError> {
        if self.blacklist.contains(&package.name.as_str()) {
            return Err(PackageManagerError::PackageBlacklisted(
                package.identifier.clone(),
            ));
        }

        if let Some(installed) = ctx.tracker.get(&package.name) {
            let _ = delete_dir(&installed.file_path);
        }

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
        match self
            .installer
            .install_package(ctx, package, &zip_dir, profile_path)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn uninstall_package(
        &self,
        ctx: &mut Context,
        package: &InstalledPackageRecord,
        progress: &mut dyn FnMut(UninstallEvent),
        profile_path: &PathBuf,
    ) -> Result<(), PackageManagerError> {
        if self.blacklist.contains(&package.name.as_str()) {
            return Err(PackageManagerError::PackageBlacklisted(
                package.identifier.clone(),
            ));
        }

        progress(UninstallEvent::UninstallingMod {
            name: package.identifier.clone(),
        });

        match self.installer.uninstall_package(ctx, package) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_package(
        &self,
        ctx: &mut Context,
        package: &PackageRecord,
        progress: &mut dyn FnMut(UpdateEvent),
        profile_path: &PathBuf,
    ) -> Result<(), PackageManagerError> {
        if self.blacklist.contains(&package.name.as_str()) {
            return Err(PackageManagerError::PackageBlacklisted(
                package.identifier.clone(),
            ));
        }

        if let Some(installed) = ctx.tracker.get(&package.name) {
            progress(UpdateEvent::CleaningUpOldMod {
                name: package.name.clone(),
            });
            let _ = delete_dir(&installed.file_path);
        }

        progress(UpdateEvent::DownloadingMod {
            name: package.identifier.clone(),
        });

        let zip_dir = self
            .downloader
            .download(&package.download_url, &mut |_| {})?;

        progress(UpdateEvent::InstallingMod {
            name: package.identifier.clone(),
        });

        self.installer
            .install_package(ctx, package, &zip_dir, profile_path)?;

        Ok(())
    }

    pub fn disable_package(
        &self,
        progress: &mut dyn FnMut(DisableEnableEvent),
        package: &InstalledPackageRecord,
    ) -> Result<(), PackageManagerError> {
        progress(DisableEnableEvent::DisablingMod {
            name: package.identifier.clone(),
        });
        self.installer.disable_package(progress, package)?;
        Ok(())
    }

    pub fn enable_package(
        &self,
        progress: &mut dyn FnMut(DisableEnableEvent),
        package: &InstalledPackageRecord,
    ) -> Result<(), PackageManagerError> {
        progress(DisableEnableEvent::EnablingMod {
            name: package.identifier.clone(),
        });
        self.installer.enable_package(progress, package)?;
        Ok(())
    }
}
