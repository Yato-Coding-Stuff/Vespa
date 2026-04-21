use std::path::PathBuf;

use crate::{
    cli::presenter::events::InstallEvent,
    manager::{
        sk_dependency_handler::{SilkSongDependencyHandler, SilkSongDependencyHandlerError},
        sk_package_manager::SilkSongPackageManagerError,
    },
    packages::SilkSongFlattenedPackage,
};
use semver::Version;
use thiserror::Error;

use crate::{manager::sk_package_manager::SilkSongPackageManager, util::context::Context};

pub enum InstallResult {
    Installed,
    AlreadyInstalled,
    OlderVersionInstalled,
    NewerVersionInstalled,
}

#[derive(Debug, Error)]
pub enum InstallError {
    #[error(transparent)]
    ManagerError(#[from] SilkSongPackageManagerError),

    #[error("Dependency errors: {0:?}")]
    DependencyErrors(Vec<SilkSongDependencyHandlerError>),
}

pub fn run<F: FnMut(InstallEvent)>(
    ctx: &mut Context,
    package: &SilkSongFlattenedPackage,
    force: bool,
    progress: &mut F,
    profile_path: &PathBuf,
) -> Result<InstallResult, InstallError> {
    let pm: SilkSongPackageManager = SilkSongPackageManager::new();
    let dependency_manager: SilkSongDependencyHandler = SilkSongDependencyHandler::new(&pm);

    let requested_version = package.version_number.parse::<Version>().unwrap();

    if !force {
        match ctx.tracker.get(&package.package_full_name) {
            None => {} // Not installed, proceed normally
            Some(installed) => {
                let installed_version = installed
                    .version_number
                    .as_deref() // Option<&str>
                    .unwrap_or("0.0.0") // default if missing
                    .parse::<Version>() // parse to Version
                    .unwrap_or_else(|_| Version::new(0, 0, 0)); // fallback if parse fails

                if installed_version > requested_version {
                    // installed version is newer
                }

                // Compare versions
                let result = if installed_version == requested_version {
                    InstallResult::AlreadyInstalled
                } else if installed_version > requested_version {
                    InstallResult::OlderVersionInstalled
                } else {
                    InstallResult::NewerVersionInstalled
                };
                return Ok(result);
            }
        }
    }

    dependency_manager
        .handle_dependencies(ctx, package.dependencies.clone(), progress, profile_path)
        .map_err(InstallError::DependencyErrors)?;
    pm.install_package(ctx, package, progress, profile_path)?;

    progress(InstallEvent::Finished);
    Ok(InstallResult::Installed)
}

#[cfg(test)]
mod tests {
    use super::{InstallResult, run};
    use crate::{
        packages::{SilkSongFlattenedPackage, SilkSongIndex},
        tracker::sk_package_tracker::SilkSongPackageTracker,
        util::{
            config::{Config, GameSwitcher},
            context::Context,
        },
    };

    fn package(version: &str) -> SilkSongFlattenedPackage {
        SilkSongFlattenedPackage {
            package_full_name: "Author-Mod".to_string(),
            owner: "Author".to_string(),
            package_full_name_with_version: format!("Author-Mod-{version}"),
            description: "desc".to_string(),
            download_url: "https://example.test/mod.zip".to_string(),
            version_number: version.to_string(),
            dependencies: vec![],
        }
    }

    fn empty_context() -> Context {
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

    #[test]
    fn returns_already_installed_when_versions_match() {
        let mut ctx = empty_context();
        let installed = package("1.0.0");
        ctx.tracker
            .add(&installed, std::path::Path::new("/mods/Author-Mod-1.0.0"));
        let mut events = Vec::new();

        let result = run(
            &mut ctx,
            &installed,
            false,
            &mut |event| {
                events.push(matches!(
                    event,
                    crate::cli::presenter::events::InstallEvent::Finished
                ))
            },
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, InstallResult::AlreadyInstalled));
        assert!(events.is_empty());
    }

    #[test]
    fn returns_older_version_installed_when_installed_is_newer() {
        let mut ctx = empty_context();
        let installed = package("2.0.0");
        ctx.tracker
            .add(&installed, std::path::Path::new("/mods/Author-Mod-2.0.0"));

        let result = run(
            &mut ctx,
            &package("1.0.0"),
            false,
            &mut |_| {},
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, InstallResult::OlderVersionInstalled));
    }

    #[test]
    fn returns_newer_version_installed_when_installed_is_older() {
        let mut ctx = empty_context();
        let installed = package("1.0.0");
        ctx.tracker
            .add(&installed, std::path::Path::new("/mods/Author-Mod-1.0.0"));

        let result = run(
            &mut ctx,
            &package("2.0.0"),
            false,
            &mut |_| {},
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, InstallResult::NewerVersionInstalled));
    }
}
