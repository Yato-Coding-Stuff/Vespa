use std::path::PathBuf;

use crate::{
    app::application::App,
    base::manager::dependency_handler::{DependencyHandler, DependencyHandlerError},
    base::{
        cli::presenter::{events::UpdateEvent, presenter::Presenter},
        manager::package_manager::PackageManagerError,
        packages::PackageRecord,
        util::context::Context,
    },
};
use semver::Version;
use thiserror::Error;

pub enum UpdateResult {
    NotInstalled,
    Updated,
    AlreadyNewestVersion,
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error(transparent)]
    ManagerError(#[from] PackageManagerError),

    #[error("Dependency errors: {0:?}")]
    DependencyErrors(Vec<DependencyHandlerError>),

    #[error("Invalid package version: {0}")]
    InvalidVersion(String),
}

/// Returns the newest indexed version for every installed package that has an
/// available update.
pub fn packages_to_update(ctx: &Context) -> Vec<PackageRecord> {
    ctx.tracker
        .get_all()
        .values()
        .filter_map(|installed| {
            let latest = ctx
                .index
                .get_latest_package_by_package_name(&installed.name)?;
            let installed_version = installed
                .version
                .as_deref()
                .unwrap_or("0.0.0")
                .parse::<Version>()
                .unwrap_or_else(|_| Version::new(0, 0, 0));
            let latest_version = latest.version.parse::<Version>().ok()?;

            (installed_version < latest_version).then_some(latest)
        })
        .collect()
}

pub fn run(
    app: &mut App,
    package: &PackageRecord,
    progress: &mut Presenter,
    bulk_update: bool,
    profile_path: &PathBuf,
) -> Result<UpdateResult, UpdateError> {
    let requested_version = package
        .version
        .parse::<Version>()
        .map_err(|_| UpdateError::InvalidVersion(package.version.clone()))?;

    match app.context.tracker.get(&package.name) {
        None => {
            return Ok(UpdateResult::NotInstalled);
        }
        Some(installed) => {
            let installed_version = installed
                .version
                .as_deref()
                .unwrap_or("0.0.0")
                .parse::<Version>()
                .unwrap_or_else(|_| Version::new(0, 0, 0));

            if installed_version == requested_version {
                return Ok(UpdateResult::AlreadyNewestVersion);
            } else if installed_version < requested_version {
                progress.display(&UpdateEvent::UpdateMod {
                    name: package.name.clone(),
                    old_version: installed_version.to_string(),
                    new_version: package.version.clone(),
                });
            } else {
                return Ok(UpdateResult::AlreadyNewestVersion);
            }
        }
    }

    update_package_with_game_layout(app, package, progress, profile_path)?;

    if !bulk_update {
        let pm = app.game_services.package_manager.as_ref();
        let dependency_manager = DependencyHandler::new(pm);
        let ctx = &mut app.context;
        let update_progress = &mut |event: UpdateEvent| {
            progress.display(&event);
        };

        dependency_manager
            .update_dependencies(
                ctx,
                package.dependencies.clone(),
                update_progress,
                profile_path,
            )
            .map_err(UpdateError::DependencyErrors)?;
    }

    Ok(UpdateResult::Updated)
}

fn update_package_with_game_layout(
    app: &mut App,
    package: &PackageRecord,
    progress: &mut Presenter,
    profile_path: &PathBuf,
) -> Result<(), PackageManagerError> {
    let pm = app.game_services.package_manager.as_ref();

    if pm.install_special_package(&mut app.context, package, progress, profile_path)? {
        return Ok(());
    }

    let update_progress = &mut |event: UpdateEvent| {
        progress.display(&event);
    };
    pm.update_package(&mut app.context, package, update_progress, profile_path)
}

#[cfg(test)]
mod tests {
    use super::{UpdateResult, run};
    use crate::{
        app::{application::App, game_services::GameServices},
        base::{
            cli::presenter::presenter::Presenter,
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
    fn returns_not_installed_when_tracker_has_no_package() {
        let mut app = empty_app();
        let mut presenter = Presenter::new();

        let result = run(
            &mut app,
            &package("1.0.0"),
            &mut presenter,
            false,
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, UpdateResult::NotInstalled));
    }

    #[test]
    fn returns_already_newest_version_when_versions_match() {
        let mut app = empty_app();
        let installed = package("1.0.0");
        app.context.tracker.insert(InstalledPackageRecord {
            name: installed.name.clone(),
            identifier: installed.identifier.clone(),
            version: Some(installed.version.clone()),
            file_path: "/mods/Author-Mod-1.0.0".into(),
        });
        let mut presenter = Presenter::new();

        let result = run(
            &mut app,
            &installed,
            &mut presenter,
            false,
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, UpdateResult::AlreadyNewestVersion));
    }
}
