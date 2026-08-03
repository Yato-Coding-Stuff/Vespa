use std::ffi::OsStr;
use std::fs::{read_dir, rename};
use std::path::PathBuf;

use tempfile::TempDir;
use thiserror::Error;

use crate::base::{
    cli::presenter::events::DisableEnableEvent,
    packages::{InstalledPackageRecord, PackageRecord},
    util::{
        context::Context,
        file_handler::{FileHandlerError, delete_dir, recursively_copy_dir, unzip_to_dir},
    },
};

#[derive(Debug, Error)]
pub enum PackageInstallerError {
    #[error("The package is not installed")]
    PackageNotInstalled,
    #[error(transparent)]
    FileHandlingError(#[from] FileHandlerError),
    #[error(transparent)]
    VersionParsingError(#[from] semver::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub struct PackageInstaller;

impl PackageInstaller {
    pub fn new() -> Self {
        PackageInstaller
    }

    pub fn install_package(
        &self,
        ctx: &mut Context,
        package: &PackageRecord,
        dir: &TempDir,
        destination: &PathBuf,
    ) -> Result<(), PackageInstallerError> {
        let zip_path = dir.path().join("package.zip");
        let unzip_dir = dir.path().join("unzipped");

        unzip_to_dir(&zip_path, &unzip_dir)?;

        recursively_copy_dir(&unzip_dir, &destination)?;

        ctx.tracker.insert(InstalledPackageRecord {
            name: package.name.clone(),
            identifier: package.identifier.clone(),
            version: Some(package.version.clone()),
            file_path: destination.clone(),
        });

        Ok(())
    }

    // TODO
    // this should also save into the index, otherwise we have no way of removing them afterwards
    pub fn install_local_package(
        &self,
        dir: &PathBuf,
        package_root: &PathBuf,
    ) -> Result<(), PackageInstallerError> {
        let zip_path = dir;

        let temp_dir = TempDir::new().map_err(FileHandlerError::CreateZipDirError)?;

        let unzip_dir = temp_dir.path().join("unzipped");

        std::fs::create_dir_all(&unzip_dir).map_err(FileHandlerError::CreateZipDirError)?;

        unzip_to_dir(zip_path, &unzip_dir)?;

        let package_name = zip_path.file_stem().unwrap();

        let mod_path = package_root.join(package_name);

        recursively_copy_dir(&unzip_dir, &mod_path)?;

        Ok(())
    }

    pub fn uninstall_package(
        &self,
        ctx: &mut Context,
        package: &InstalledPackageRecord,
    ) -> Result<(), PackageInstallerError> {
        let installed_package = match ctx.tracker.get(&package.name) {
            Some(p) => p,
            None => return Err(PackageInstallerError::PackageNotInstalled),
        };

        delete_dir(&installed_package.file_path)
            .map_err(PackageInstallerError::FileHandlingError)?;

        ctx.tracker.remove(&package.name);

        Ok(())
    }

    pub fn disable_package(
        &self,
        progress: &mut dyn FnMut(DisableEnableEvent),
        package: &InstalledPackageRecord,
    ) -> Result<(), PackageInstallerError> {
        let entry = read_dir(&package.file_path)
            .map_err(PackageInstallerError::IOError)?
            .flatten()
            .find_map(|e| {
                let path = e.path();
                if path.extension()? != OsStr::new("dll") {
                    return None;
                }

                let mut disabled_name = path.file_name()?.to_os_string();
                disabled_name.push(".disabled");
                let new_path = path.with_file_name(disabled_name);

                Some((path, new_path))
            });

        let Some((path, new_path)) = entry else {
            progress(DisableEnableEvent::ModAlreadyDisabled {
                name: package.name.clone(),
            });
            return Ok(());
        };

        rename(&path, &new_path).map_err(PackageInstallerError::IOError)?;

        Ok(())
    }

    pub fn enable_package(
        &self,
        progress: &mut dyn FnMut(DisableEnableEvent),
        package: &InstalledPackageRecord,
    ) -> Result<(), PackageInstallerError> {
        let entry = read_dir(&package.file_path)
            .map_err(PackageInstallerError::IOError)?
            .flatten()
            .find_map(|e| {
                let path = e.path();
                if path.extension()? != OsStr::new("disabled") {
                    return None;
                }

                let new_path = path.with_extension("");
                if new_path.extension()? != OsStr::new("dll") {
                    return None;
                }

                Some((path, new_path))
            });

        let Some((path, new_path)) = entry else {
            progress(DisableEnableEvent::ModAlreadyEnabled {
                name: package.name.clone(),
            });
            return Ok(());
        };

        rename(&path, &new_path).map_err(PackageInstallerError::IOError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{PackageInstaller, PackageInstallerError};
    use crate::base::{
        cli::presenter::events::DisableEnableEvent,
        packages::{Index, InstalledPackageRecord, PackageRecord},
        tracker::package_tracker::PackageTracker,
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
                default_profile: None,
                hk_default_profile: None,
                hollow_knight_path: "/games/hk".into(),
                silk_song_path: "/games/sk".into(),
                index_path: "/config/index.json".into(),
            },
            tracker: PackageTracker::new(),
            index: Index::new(),
        }
    }

    fn package_record() -> PackageRecord {
        PackageRecord {
            name: "Author-Mod".to_string(),
            identifier: "Author-Mod-1.0.0".to_string(),
            description: "desc".to_string(),
            download_url: "https://example.test/mod.zip".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
        }
    }

    #[test]
    fn uninstall_package_errors_when_tracker_does_not_contain_package() {
        let installer = PackageInstaller::new();
        let mut ctx = context();
        let package = InstalledPackageRecord {
            identifier: "Author-Mod-1.0.0".to_string(),
            name: "Author-Mod".to_string(),
            version: Some("1.0.0".to_string()),
            file_path: "/mods/Author-Mod-1.0.0".into(),
        };

        let result = installer.uninstall_package(&mut ctx, &package);

        assert!(matches!(
            result,
            Err(PackageInstallerError::PackageNotInstalled)
        ));
    }

    #[test]
    fn disable_and_enable_package_toggle_dll_suffixes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package_dir = temp_dir.path().join("Author-Mod-1.0.0");
        fs::create_dir_all(&package_dir).unwrap();
        let dll_path = package_dir.join("mod.dll");
        fs::write(&dll_path, "binary").unwrap();

        let installer = PackageInstaller::new();
        let installed = InstalledPackageRecord {
            identifier: "Author-Mod-1.0.0".to_string(),
            name: "Author-Mod".to_string(),
            version: Some("1.0.0".to_string()),
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

        let installer = PackageInstaller::new();
        let installed = InstalledPackageRecord {
            identifier: "Author-Mod-1.0.0".to_string(),
            name: "Author-Mod".to_string(),
            version: Some("1.0.0".to_string()),
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

        let installer = PackageInstaller::new();
        let package = package_record();
        let mut ctx = context();
        ctx.tracker.insert(InstalledPackageRecord {
            name: package.name.clone(),
            identifier: package.identifier.clone(),
            version: Some(package.version.clone()),
            file_path: Path::new(&package_dir).to_path_buf(),
        });
        let installed = ctx.tracker.get("Author-Mod").unwrap().clone();

        installer.uninstall_package(&mut ctx, &installed).unwrap();

        assert!(!package_dir.exists());
        assert!(ctx.tracker.get("Author-Mod").is_none());
    }
}
