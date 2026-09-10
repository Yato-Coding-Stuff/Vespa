use std::path::{Component, Path, PathBuf};

use tempfile::TempDir;
use thiserror::Error;

use crate::base::{
    cli::presenter::events::{DisableEnableEvent, InstallEvent, UninstallEvent, UpdateEvent},
    cli::presenter::presenter::Presenter,
    manager::{
        package_downloader::{PackageDownloader, PackageDownloaderError},
        package_installer::{PackageInstaller, PackageInstallerError},
        package_layout::PackageLayout,
    },
    packages::{PackageRecord, package::InstalledPackageRecord},
    util::{context::Context, file_handler::FileHandlerError},
};

#[derive(Debug, Error)]
pub enum PackageManagerError {
    #[error(transparent)]
    DownloaderError(#[from] PackageDownloaderError),
    #[error(transparent)]
    InstallerError(#[from] PackageInstallerError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    FileHandlingError(#[from] FileHandlerError),
    #[error("Package is blacklisted: {0}")]
    PackageBlacklisted(String),
    #[error("Invalid package path component: {0}")]
    InvalidPackagePath(String),
}

pub struct PackageManager {
    downloader: PackageDownloader,
    installer: PackageInstaller,
    blacklist: &'static [&'static str],
    package_layout: Box<dyn PackageLayout>,
}

impl PackageManager {
    pub fn new(blacklist: &'static [&'static str], package_layout: Box<dyn PackageLayout>) -> Self {
        Self {
            downloader: PackageDownloader::new(),
            installer: PackageInstaller::new(),
            blacklist,
            package_layout,
        }
    }

    pub(crate) fn download_archive(
        &self,
        url: &str,
        progress: &mut dyn FnMut(InstallEvent),
    ) -> Result<TempDir, PackageDownloaderError> {
        self.downloader.download(url, progress)
    }

    pub(crate) fn commit_installation(
        &self,
        source: &Path,
        destination: &Path,
        previous: Option<&Path>,
    ) -> Result<(), PackageInstallerError> {
        self.installer
            .commit_installation(source, destination, previous)
    }

    pub(crate) fn write_version_marker(
        &self,
        destination: &Path,
        version: &str,
    ) -> Result<(), PackageManagerError> {
        std::fs::write(destination.join(".vespa-version"), version)?;
        Ok(())
    }

    pub(crate) fn install_special_package(
        &self,
        ctx: &mut Context,
        package: &PackageRecord,
        presenter: &mut Presenter,
        profile_path: &Path,
    ) -> Result<bool, PackageManagerError> {
        self.package_layout
            .install_special_package(self, ctx, package, presenter, profile_path)
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

        let destination = self.package_destination(profile_path, package)?;
        self.installer
            .install_package(ctx, package, &zip_dir, &destination)?;
        self.write_version_marker(&destination, &package.version)?;
        Ok(())
    }

    pub fn install_local_package(
        &self,
        ctx: &mut Context,
        archive_path: &PathBuf,
        profile_path: &PathBuf,
    ) -> Result<(), PackageManagerError> {
        let package_root = self.package_layout.package_root(profile_path);
        self.installer
            .install_local_package(ctx, archive_path, &package_root)?;
        Ok(())
    }

    pub fn uninstall_package(
        &self,
        ctx: &mut Context,
        package: &InstalledPackageRecord,
        progress: &mut dyn FnMut(UninstallEvent),
        _profile_path: &PathBuf,
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

        if ctx.tracker.get(&package.name).is_some() {
            progress(UpdateEvent::CleaningUpOldMod {
                name: package.name.clone(),
            });
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

        let destination = self.package_destination(profile_path, package)?;
        self.installer
            .install_package(ctx, package, &zip_dir, &destination)?;
        self.write_version_marker(&destination, &package.version)?;

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

    fn package_destination(
        &self,
        profile_path: &Path,
        package: &PackageRecord,
    ) -> Result<PathBuf, PackageManagerError> {
        let identifier_path = Path::new(&package.identifier);
        if package.identifier.is_empty()
            || identifier_path.components().count() != 1
            || !matches!(
                identifier_path.components().next(),
                Some(Component::Normal(_))
            )
        {
            return Err(PackageManagerError::InvalidPackagePath(
                package.identifier.clone(),
            ));
        }

        Ok(self
            .package_layout
            .package_root(profile_path)
            .join(identifier_path))
    }
}
