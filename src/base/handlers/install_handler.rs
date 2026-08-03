use std::path::PathBuf;

use crate::{
    app::application::App,
    base::manager::dependency_handler::{DependencyHandler, DependencyHandlerError},
    base::{
        cli::presenter::events::InstallEvent, manager::package_manager::PackageManagerError,
        packages::PackageRecord,
    },
};

use semver::Version;
use thiserror::Error;

pub enum InstallResult {
    Installed,
    AlreadyInstalled,
    OlderVersionInstalled,
    NewerVersionInstalled,
}

#[derive(Debug, Error)]
pub enum InstallError {
    #[error(transparent)]
    ManagerError(#[from] PackageManagerError),

    #[error("Dependency errors: {0:?}")]
    DependencyErrors(Vec<DependencyHandlerError>),
}

pub fn run<F: FnMut(InstallEvent)>(
    app: &mut App,
    package: &PackageRecord,
    force: bool,
    progress: &mut F,
    profile_path: &PathBuf,
) -> Result<InstallResult, InstallError> {
    let ctx = &mut app.context;
    let pm = app.game_services.package_manager.as_ref();
    let dependency_manager = DependencyHandler::new(pm);

    let requested_version = package.version.parse::<Version>().unwrap();

    if !force {
        match ctx.tracker.get(&package.name) {
            None => {} // Not installed, proceed normally
            Some(installed) => {
                let installed_version = installed
                    .version
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
        app::{application::App, game_services::GameServices},
        base::{
            cli::presenter::events::InstallEvent,
            packages::{Index, InstalledPackageRecord, PackageRecord},
            tracker::package_tracker::PackageTracker,
            util::{
                config::{Config, GameSwitcher},
                context::Context,
            },
        },
    };

    fn package(version: &str) -> PackageRecord {
        PackageRecord {
            name: "Author-Mod".to_string(),
            identifier: format!("Author-Mod-{version}"),
            description: "desc".to_string(),
            download_url: "https://example.test/mod.zip".to_string(),
            version: version.to_string(),
            dependencies: vec![],
        }
    }

    fn empty_app() -> App {
        let game = GameSwitcher::SilkSong;
        let context = Context {
            config: Config {
                game_switcher: game.clone(),
                default_profile: None,
                hk_default_profile: None,
                hollow_knight_path: "/games/hk".into(),
                silk_song_path: "/games/sk".into(),
                index_path: "/config/index.json".into(),
            },
            tracker: PackageTracker::new(),
            index: Index::new(),
        };

        App::new(game.clone(), context, GameServices::for_game(&game))
    }

    #[test]
    fn returns_already_installed_when_versions_match() {
        let mut app = empty_app();
        let installed = package("1.0.0");
        app.context.tracker.insert(InstalledPackageRecord {
            name: installed.name.clone(),
            identifier: installed.identifier.clone(),
            version: Some(installed.version.clone()),
            file_path: "/mods/Author-Mod-1.0.0".into(),
        });
        let mut events = Vec::new();

        let result = run(
            &mut app,
            &installed,
            false,
            &mut |event| events.push(matches!(event, InstallEvent::Finished)),
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, InstallResult::AlreadyInstalled));
        assert!(events.is_empty());
    }

    #[test]
    fn returns_older_version_installed_when_installed_is_newer() {
        let mut app = empty_app();
        let installed = package("2.0.0");
        app.context.tracker.insert(InstalledPackageRecord {
            name: installed.name.clone(),
            identifier: installed.identifier.clone(),
            version: Some(installed.version.clone()),
            file_path: "/mods/Author-Mod-2.0.0".into(),
        });

        let result = run(
            &mut app,
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
        let mut app = empty_app();
        let installed = package("1.0.0");
        app.context.tracker.insert(InstalledPackageRecord {
            name: installed.name.clone(),
            identifier: installed.identifier.clone(),
            version: Some(installed.version.clone()),
            file_path: "/mods/Author-Mod-1.0.0".into(),
        });

        let result = run(
            &mut app,
            &package("2.0.0"),
            false,
            &mut |_| {},
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, InstallResult::NewerVersionInstalled));
    }
}
