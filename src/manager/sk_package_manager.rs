use std::path::PathBuf;

use thiserror::Error;

use crate::{
    cli::presenter::events::install_event::InstallEvent,
    manager::{
        downloader::sk_package_downloader::{
            SilkSongPackageDownloader, SilkSongPackageDownloaderError,
        },
        installer::sk_package_installer::{
            SilkSongPackageInstaller, SilkSongPackageInstallerError,
        },
    },
    packages::{SilkSongFlattenedPackage, SilkSongInstalledPackageRecord},
};

#[derive(Debug, Error)]
pub enum SilkSongPackageManagerError {
    #[error(transparent)]
    DownloaderError(#[from] SilkSongPackageDownloaderError),
    #[error(transparent)]
    InstallerError(#[from] SilkSongPackageInstallerError),
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
        progress(InstallEvent::DownloadingMod {
            name: package.package_name_with_version.clone(),
        });
        let zip_dir = self.downloader.download(&package.download_url, progress)?;

        progress(InstallEvent::InstallingMod {
            name: package.package_name_with_version.clone(),
        });
        match self
            .installer
            .install_package(ctx, package, &zip_dir, profile_path)
        {
            Ok(_) => {
                let package_record = SilkSongInstalledPackageRecord {
                    version_full_name: package.package_name_with_version.clone(),
                    version_number: package.version_number.parse().unwrap(),
                };
                ctx.tracker
                    .add_installed_package_record(package.package_name.clone(), &package_record);

                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}
