use std::path::{Path, PathBuf};

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
    package_root: fn(&Path) -> PathBuf,
}

impl PackageManager {
    pub fn new(blacklist: &'static [&'static str], package_root: fn(&Path) -> PathBuf) -> Self {
        Self {
            downloader: PackageDownloader::new(),
            installer: PackageInstaller::new(),
            blacklist,
            package_root,
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

        let destination = self.package_destination(profile_path, package);
        match self
            .installer
            .install_package(ctx, package, &zip_dir, &destination)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn install_local_package(
        &self,
        archive_path: &PathBuf,
        profile_path: &PathBuf,
    ) -> Result<(), PackageManagerError> {
        let package_root = (self.package_root)(profile_path);
        self.installer
            .install_local_package(archive_path, &package_root)?;
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

        let destination = self.package_destination(profile_path, package);
        self.installer
            .install_package(ctx, package, &zip_dir, &destination)?;

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

    fn package_destination(&self, profile_path: &Path, package: &PackageRecord) -> PathBuf {
        (self.package_root)(profile_path).join(&package.identifier)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::PackageManager;
    use crate::base::packages::PackageRecord;

    fn nested_package_root(profile_path: &Path) -> PathBuf {
        profile_path.join("packages")
    }

    #[test]
    fn package_destination_combines_game_root_and_identifier() {
        let manager = PackageManager::new(&[], nested_package_root);
        let package = PackageRecord {
            name: "Author-Mod".to_string(),
            identifier: "Author-Mod-1.2.3".to_string(),
            version: "1.2.3".to_string(),
            description: String::new(),
            download_url: String::new(),
            dependencies: Vec::new(),
        };

        assert_eq!(
            manager.package_destination(Path::new("/profile"), &package),
            PathBuf::from("/profile/packages/Author-Mod-1.2.3")
        );
    }
}
