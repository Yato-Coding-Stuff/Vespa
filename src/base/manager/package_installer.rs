use std::fs::{read_dir, rename};
use std::path::PathBuf;
use std::{ffi::OsStr, fs, io};

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
        Self
    }

    pub(crate) fn commit_installation(
        &self,
        source: &std::path::Path,
        destination: &std::path::Path,
        previous: Option<&std::path::Path>,
    ) -> Result<(), PackageInstallerError> {
        let parent = destination
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        fs::create_dir_all(parent).map_err(FileHandlerError::CreateDirError)?;

        let name = destination
            .file_name()
            .ok_or_else(|| {
                FileHandlerError::ReplaceDirError(io::Error::other("invalid destination"))
            })?
            .to_string_lossy();
        let suffix = std::process::id();
        let staging = parent.join(format!(".{name}.vespa-staging-{suffix}"));
        let backup = parent.join(format!(".{name}.vespa-backup-{suffix}"));

        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(FileHandlerError::ReplaceDirError)?;
        }
        if backup.exists() {
            fs::remove_dir_all(&backup).map_err(FileHandlerError::ReplaceDirError)?;
        }

        recursively_copy_dir(source, &staging)?;

        let had_destination = destination.exists();
        if had_destination {
            fs::rename(destination, &backup).map_err(FileHandlerError::ReplaceDirError)?;
        }

        if let Err(error) = fs::rename(&staging, destination) {
            if had_destination {
                let _ = fs::rename(&backup, destination);
            }
            let _ = fs::remove_dir_all(&staging);
            return Err(FileHandlerError::ReplaceDirError(error).into());
        }

        if had_destination {
            fs::remove_dir_all(&backup).map_err(FileHandlerError::ReplaceDirError)?;
        }

        if let Some(previous) = previous {
            if previous != destination && previous.exists() {
                delete_dir(previous)?;
            }
        }

        Ok(())
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

        let previous = ctx
            .tracker
            .get(&package.name)
            .map(|record| record.file_path.clone());
        self.commit_installation(&unzip_dir, destination, previous.as_deref())?;

        ctx.tracker.insert(InstalledPackageRecord {
            name: package.name.clone(),
            identifier: package.identifier.clone(),
            version: Some(package.version.clone()),
            file_path: destination.clone(),
        });

        Ok(())
    }

    pub fn install_local_package(
        &self,
        ctx: &mut Context,
        dir: &PathBuf,
        package_root: &PathBuf,
    ) -> Result<(), PackageInstallerError> {
        let zip_path = dir;

        let temp_dir = TempDir::new().map_err(FileHandlerError::CreateZipDirError)?;

        let unzip_dir = temp_dir.path().join("unzipped");

        std::fs::create_dir_all(&unzip_dir).map_err(FileHandlerError::CreateZipDirError)?;

        unzip_to_dir(zip_path, &unzip_dir)?;

        let package_name = zip_path.file_stem().ok_or_else(|| {
            PackageInstallerError::IOError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "local package archive has no filename",
            ))
        })?;
        if package_name == std::ffi::OsStr::new(".") || package_name == std::ffi::OsStr::new("..") {
            return Err(PackageInstallerError::IOError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid local package filename",
            )));
        }

        let mod_path = package_root.join(package_name);

        let name = package_name.to_string_lossy().into_owned();
        let previous = ctx
            .tracker
            .get(&name)
            .map(|record| record.file_path.clone());
        self.commit_installation(&unzip_dir, &mod_path, previous.as_deref())?;

        ctx.tracker.insert(InstalledPackageRecord {
            name,
            identifier: package_name.to_string_lossy().into_owned(),
            version: None,
            file_path: mod_path,
        });

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

    #[test]
    fn transaction_replaces_destination_and_cleans_previous_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source = temp_dir.path().join("source");
        let destination = temp_dir.path().join("destination");
        let previous = temp_dir.path().join("previous");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&destination).unwrap();
        fs::create_dir_all(&previous).unwrap();
        fs::write(source.join("new.txt"), "new").unwrap();
        fs::write(destination.join("old.txt"), "old").unwrap();

        PackageInstaller::new()
            .commit_installation(&source, &destination, Some(&previous))
            .unwrap();

        assert!(!destination.join("old.txt").exists());
        assert!(destination.join("new.txt").exists());
        assert!(!previous.exists());
    }
}
