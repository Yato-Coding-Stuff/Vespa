use std::path::PathBuf;

use crate::{
    app::application::App,
    base::{
        cli::presenter::events::UninstallEvent, manager::package_manager::PackageManagerError,
        packages::InstalledPackageRecord,
    },
};

use crate::base::manager::dependency_handler::{
    DependencyHandler, DependencyHandlerError, ReverseDependencyHandler,
};
use thiserror::Error;

pub enum UninstallResult {
    Uninstalled,
    NotInstalled,
    PackageStillRequired { packages: Vec<String> },
}

#[derive(Debug, Error)]
pub enum UninstallError {
    #[error(transparent)]
    ManagerError(#[from] PackageManagerError),

    #[error("Dependency errors: {0:?}")]
    DependencyErrors(Vec<DependencyHandlerError>),
}

pub fn run<F: FnMut(UninstallEvent)>(
    app: &mut App,
    package: &InstalledPackageRecord,
    force: bool,
    progress: &mut F,
    profile_path: &PathBuf,
) -> Result<UninstallResult, UninstallError> {
    let pm = app.game_services.package_manager.as_ref();
    let deps = DependencyHandler::new(pm);
    let ctx = &mut app.context;

    if let packages = ReverseDependencyHandler::packages_requiring(ctx, &package.name)
        && !force
    {
        return Ok(UninstallResult::PackageStillRequired { packages });
    }

    pm.uninstall_package(ctx, package, progress, profile_path)?;

    let dep: Option<Vec<String>> = ctx
        .index
        .get_package_by_identifier(&package.identifier)
        .map(|p| p.dependencies);

    if let Some(dep) = dep {
        let still_required = deps
            .uninstall_dependencies(ctx, dep, force, progress, profile_path)
            .map_err(UninstallError::DependencyErrors)?;

        if !still_required.is_empty() {
            return Ok(UninstallResult::PackageStillRequired {
                packages: still_required,
            });
        }
    }

    Ok(UninstallResult::Uninstalled)
}

#[cfg(test)]
mod tests {
    use super::{UninstallResult, run};
    use crate::{
        app::{application::App, game_services::GameServices},
        base::{
            packages::{Index, InstalledPackageRecord, PackageRecord},
            tracker::package_tracker::PackageTracker,
        },
    };

    use crate::base::util::{
        config::{Config, GameSwitcher},
        context::Context,
    };

    fn package(name: &str, version: &str, dependencies: Vec<&str>) -> PackageRecord {
        PackageRecord {
            name: name.to_string(),
            identifier: format!("{name}-{version}"),
            description: "desc".to_string(),
            download_url: "https://example.test/mod.zip".to_string(),
            version: version.to_string(),
            dependencies: dependencies.into_iter().map(str::to_string).collect(),
        }
    }

    fn app_with_dependency_graph() -> (App, InstalledPackageRecord) {
        let target = package("Author-Dependency", "1.0.0", vec![]);
        let dependent = package("Author-Mod", "1.0.0", vec!["Author-Dependency-1.0.0"]);

        let mut tracker = PackageTracker::new();
        tracker.insert(InstalledPackageRecord {
            name: target.name.clone(),
            identifier: target.identifier.clone(),
            version: Some(target.version.clone()),
            file_path: "/mods/Author-Dependency-1.0.0".into(),
        });
        tracker.insert(InstalledPackageRecord {
            name: dependent.name.clone(),
            identifier: dependent.identifier.clone(),
            version: Some(dependent.version.clone()),
            file_path: "/mods/Author-Mod-1.0.0".into(),
        });

        let mut index = Index::new();
        index.replace([target.clone(), dependent.clone()]);

        let tracked_target = tracker.get("Author-Dependency").unwrap().clone();

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
            tracker,
            index,
        };

        (
            App::new(game.clone(), context, GameServices::for_game(&game)),
            tracked_target,
        )
    }

    #[test]
    fn returns_package_still_required_before_touching_package_manager() {
        let (mut app, target) = app_with_dependency_graph();

        let result = run(
            &mut app,
            &target,
            false,
            &mut |_| {},
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, UninstallResult::PackageStillRequired));
    }
}
