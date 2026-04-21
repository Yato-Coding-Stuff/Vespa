use std::{
    fs::{self, read_dir, rename},
    path::{Path, PathBuf},
};

use tempfile::{TempDir, tempdir};
use thiserror::Error;

use crate::{
    cli::presenter::events::DisableEnableEvent,
    packages::{SilkSongFlattenedPackage, SilkSongInstalledPackageRecord},
    util::{
        context::Context,
        file_handler::{FileHandlerError, delete_dir, recursively_copy_dir, unzip_to_dir},
    },
};

#[derive(Debug, Error)]
pub enum SilkSongPackageInstallerError {
    #[error("The package is already installed")]
    PackageAlreadyInstalled,
    #[error("The package is not installed")]
    PackageNotInstalled,
    #[error(transparent)]
    FileHandlingError(#[from] FileHandlerError),
    #[error(transparent)]
    VersionParsingError(#[from] semver::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
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

    pub fn install_local_package(
        &self,
        dir: &PathBuf,
        profile_path: &PathBuf,
    ) -> Result<(), SilkSongPackageInstallerError> {
        let zip_path = dir;

        let temp_dir = TempDir::new().map_err(FileHandlerError::CreateZipDirError)?;

        let unzip_dir = temp_dir.path().join("unzipped");

        std::fs::create_dir_all(&unzip_dir).map_err(FileHandlerError::CreateZipDirError)?;

        unzip_to_dir(zip_path, &unzip_dir)?;

        let package_name = zip_path.file_stem().unwrap().to_string_lossy().to_string();

        let mod_path = profile_path
            .join("BepInEx")
            .join("plugins")
            .join(package_name);

        recursively_copy_dir(&unzip_dir, &mod_path)?;

        Ok(())
    }

    pub fn uninstall_package(
        &self,
        ctx: &mut Context,
        package: &SilkSongInstalledPackageRecord,
    ) -> Result<(), SilkSongPackageInstallerError> {
        let installed_package = match ctx.tracker.get(&package.package_full_name) {
            Some(p) => p,
            None => return Err(SilkSongPackageInstallerError::PackageNotInstalled),
        };

        delete_dir(&installed_package.file_path)
            .map_err(SilkSongPackageInstallerError::FileHandlingError)?;

        ctx.tracker.remove(&package.package_full_name);

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

    pub fn disable_package<F: FnMut(DisableEnableEvent)>(
        &self,
        progress: &mut F,
        package: &SilkSongInstalledPackageRecord,
    ) -> Result<(), SilkSongPackageInstallerError> {
        let entry = read_dir(&package.file_path)
            .map_err(SilkSongPackageInstallerError::IOError)?
            .flatten()
            .find_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?;

                let stem = name.strip_suffix(".dll")?;
                Some((path.clone(), stem.to_string()))
            });

        let Some((path, stem)) = entry else {
            progress(DisableEnableEvent::ModAlreadyDisabled {
                name: package.package_full_name.clone(),
            });
            return Ok(());
        };

        let new_path = path.with_file_name(format!("{stem}.dll.disabled"));

        rename(&path, &new_path).map_err(SilkSongPackageInstallerError::IOError)?;

        Ok(())
    }

    pub fn enable_package<F: FnMut(DisableEnableEvent)>(
        &self,
        progress: &mut F,
        package: &SilkSongInstalledPackageRecord,
    ) -> Result<(), SilkSongPackageInstallerError> {
        let entry = read_dir(&package.file_path)
            .map_err(SilkSongPackageInstallerError::IOError)?
            .flatten()
            .find_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?;

                let stem = name.strip_suffix(".dll.disabled")?;
                Some((path.clone(), stem.to_string()))
            });

        let Some((path, stem)) = entry else {
            progress(DisableEnableEvent::ModAlreadyEnabled {
                name: package.package_full_name.clone(),
            });
            return Ok(());
        };

        let new_path = path.with_file_name(format!("{stem}.dll"));

        rename(&path, &new_path).map_err(SilkSongPackageInstallerError::IOError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SilkSongPackageInstaller, SilkSongPackageInstallerError};
    use crate::{
        cli::presenter::events::DisableEnableEvent,
        packages::{SilkSongFlattenedPackage, SilkSongIndex},
        tracker::sk_package_tracker::SilkSongPackageTracker,
        util::{
            config::{Config, GameSwitcher},
            context::Context,
        },
    };
    use std::{fs, path::Path};

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
            black_list: vec![],
        }
    }

    fn package_record() -> SilkSongFlattenedPackage {
        SilkSongFlattenedPackage {
            package_full_name: "Author-Mod".to_string(),
            owner: "Author".to_string(),
            package_full_name_with_version: "Author-Mod-1.0.0".to_string(),
            description: "desc".to_string(),
            download_url: "https://example.test/mod.zip".to_string(),
            version_number: "1.0.0".to_string(),
            dependencies: vec![],
        }
    }

    #[test]
    fn uninstall_package_errors_when_tracker_does_not_contain_package() {
        let installer = SilkSongPackageInstaller::new();
        let mut ctx = context();
        let package = crate::packages::SilkSongInstalledPackageRecord {
            package_full_name_with_version: "Author-Mod-1.0.0".to_string(),
            package_full_name: "Author-Mod".to_string(),
            version_number: Some("1.0.0".to_string()),
            file_path: "/mods/Author-Mod-1.0.0".into(),
        };

        let result = installer.uninstall_package(&mut ctx, &package);

        assert!(matches!(
            result,
            Err(SilkSongPackageInstallerError::PackageNotInstalled)
        ));
    }

    #[test]
    fn disable_and_enable_package_toggle_dll_suffixes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package_dir = temp_dir.path().join("Author-Mod-1.0.0");
        fs::create_dir_all(&package_dir).unwrap();
        let dll_path = package_dir.join("mod.dll");
        fs::write(&dll_path, "binary").unwrap();

        let installer = SilkSongPackageInstaller::new();
        let installed = crate::packages::SilkSongInstalledPackageRecord {
            package_full_name_with_version: "Author-Mod-1.0.0".to_string(),
            package_full_name: "Author-Mod".to_string(),
            version_number: Some("1.0.0".to_string()),
            file_path: package_dir.clone(),
        };

        let mut events = Vec::new();
        installer
            .disable_package(&mut |event| events.push(event), &installed)
            .unwrap();
        assert!(!dll_path.exists());
        assert!(package_dir.join("mod.dll.disabled").exists());

        installer
            .enable_package(&mut |event| events.push(event), &installed)
            .unwrap();
        assert!(dll_path.exists());
        assert!(!package_dir.join("mod.dll.disabled").exists());
        assert!(events.is_empty());
    }

    #[test]
    fn disable_package_reports_already_disabled_when_no_dll_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package_dir = temp_dir.path().join("Author-Mod-1.0.0");
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(package_dir.join("mod.dll.disabled"), "binary").unwrap();

        let installer = SilkSongPackageInstaller::new();
        let installed = crate::packages::SilkSongInstalledPackageRecord {
            package_full_name_with_version: "Author-Mod-1.0.0".to_string(),
            package_full_name: "Author-Mod".to_string(),
            version_number: Some("1.0.0".to_string()),
            file_path: package_dir,
        };

        let mut events = Vec::new();
        installer
            .disable_package(&mut |event| events.push(event), &installed)
            .unwrap();

        assert!(matches!(
            events.as_slice(),
            [DisableEnableEvent::ModAlreadyDisabled { name }] if name == "Author-Mod"
        ));
    }

    #[test]
    fn uninstall_package_removes_directory_and_tracker_record() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package_dir = temp_dir.path().join("Author-Mod-1.0.0");
        fs::create_dir_all(&package_dir).unwrap();

        let installer = SilkSongPackageInstaller::new();
        let package = package_record();
        let mut ctx = context();
        ctx.tracker.add(&package, Path::new(&package_dir));
        let installed = ctx.tracker.get("Author-Mod").unwrap().clone();

        installer.uninstall_package(&mut ctx, &installed).unwrap();

        assert!(!package_dir.exists());
        assert!(ctx.tracker.get("Author-Mod").is_none());
    }
}
