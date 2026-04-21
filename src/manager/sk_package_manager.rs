use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    cli::presenter::events::{DisableEnableEvent, InstallEvent, UninstallEvent, UpdateEvent},
    manager::{
        sk_package_downloader::{SilkSongPackageDownloader, SilkSongPackageDownloaderError},
        sk_package_installer::{SilkSongPackageInstaller, SilkSongPackageInstallerError},
    },
    packages::{SilkSongFlattenedPackage, SilkSongInstalledPackageRecord},
    util::{context::Context, file_handler::delete_dir},
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

    pub fn disable_package<F: FnMut(DisableEnableEvent)>(
        &self,
        progress: &mut F,
        package: &SilkSongInstalledPackageRecord,
    ) -> Result<(), SilkSongPackageManagerError> {
        progress(DisableEnableEvent::DisablingMod {
            name: package.package_full_name_with_version.clone(),
        });
        self.installer.disable_package(progress, package)?;
        Ok(())
    }

    pub fn enable_package<F: FnMut(DisableEnableEvent)>(
        &self,
        progress: &mut F,
        package: &SilkSongInstalledPackageRecord,
    ) -> Result<(), SilkSongPackageManagerError> {
        progress(DisableEnableEvent::EnablingMod {
            name: package.package_full_name_with_version.clone(),
        });
        self.installer.enable_package(progress, package)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SilkSongPackageManager, SilkSongPackageManagerError};
    use crate::{
        packages::{SilkSongFlattenedPackage, SilkSongIndex, SilkSongInstalledPackageRecord},
        tracker::sk_package_tracker::SilkSongPackageTracker,
        util::{
            config::{Config, GameSwitcher},
            context::Context,
        },
    };

    fn context() -> Context {
        Context {
            config: Config {
                game_switcher: GameSwitcher::SilkSong,
                sk_default_profile: None,
                hk_default_profile: None,
                hollow_knight_path: "/games/hk".into(),
                silk_song_path: "/games/sk".into(),
                index_path: "/config/index.json".into(),
            },
            tracker: SilkSongPackageTracker::new(),
            index: SilkSongIndex::new(),
            black_list: vec!["Blocked-Mod"],
        }
    }

    fn flattened(name: &str) -> SilkSongFlattenedPackage {
        SilkSongFlattenedPackage {
            package_full_name: name.to_string(),
            owner: "Author".to_string(),
            package_full_name_with_version: format!("{name}-1.0.0"),
            description: "desc".to_string(),
            download_url: "https://example.test/mod.zip".to_string(),
            version_number: "1.0.0".to_string(),
            dependencies: vec![],
        }
    }

    fn installed(name: &str) -> SilkSongInstalledPackageRecord {
        SilkSongInstalledPackageRecord {
            package_full_name_with_version: format!("{name}-1.0.0"),
            package_full_name: name.to_string(),
            version_number: Some("1.0.0".to_string()),
            file_path: format!("/mods/{name}-1.0.0").into(),
        }
    }

    #[test]
    fn install_package_rejects_blacklisted_package_before_downloading() {
        let manager = SilkSongPackageManager::new();
        let mut ctx = context();

        let result = manager.install_package(
            &mut ctx,
            &flattened("Blocked-Mod"),
            &mut |_| {},
            &"/profiles/default".into(),
        );

        assert!(matches!(
            result,
            Err(SilkSongPackageManagerError::PackageBlacklisted(name)) if name == "Blocked-Mod-1.0.0"
        ));
    }

    #[test]
    fn uninstall_package_rejects_blacklisted_package_before_installer_runs() {
        let manager = SilkSongPackageManager::new();
        let mut ctx = context();

        let result = manager.uninstall_package(
            &mut ctx,
            &installed("Blocked-Mod"),
            &mut |_| {},
            &"/profiles/default".into(),
        );

        assert!(matches!(
            result,
            Err(SilkSongPackageManagerError::PackageBlacklisted(name)) if name == "Blocked-Mod-1.0.0"
        ));
    }

    #[test]
    fn update_package_rejects_blacklisted_package_before_downloading() {
        let manager = SilkSongPackageManager::new();
        let mut ctx = context();

        let result = manager.update_package(
            &mut ctx,
            &flattened("Blocked-Mod"),
            &mut |_| {},
            &"/profiles/default".into(),
        );

        assert!(matches!(
            result,
            Err(SilkSongPackageManagerError::PackageBlacklisted(name)) if name == "Blocked-Mod-1.0.0"
        ));
    }
}
