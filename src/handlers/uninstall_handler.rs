use std::path::PathBuf;

use crate::{
    cli::presenter::events::UninstallEvent,
    manager::{
        sk_dependency_handler::{
            SilkSongDependencyHandler, SilkSongDependencyHandlerError,
            SilkSongReverseDependencyHandler,
        },
        sk_package_manager::SilkSongPackageManagerError,
    },
    packages::{SilkSongFlattenedPackage, SilkSongInstalledPackageRecord},
};
use thiserror::Error;

use crate::{manager::sk_package_manager::SilkSongPackageManager, util::context::Context};

pub enum UninstallResult {
    Uninstalled,
    NotInstalled,
    PackageStillRequired,
}

#[derive(Debug, Error)]
pub enum UninstallError {
    #[error(transparent)]
    ManagerError(#[from] SilkSongPackageManagerError),

    #[error("Dependency errors: {0:?}")]
    DependencyErrors(Vec<SilkSongDependencyHandlerError>),
}

pub fn run<F: FnMut(UninstallEvent)>(
    ctx: &mut Context,
    package: &SilkSongInstalledPackageRecord,
    force: bool,
    progress: &mut F,
    profile_path: &PathBuf,
) -> Result<UninstallResult, UninstallError> {
    let pm = SilkSongPackageManager::new();
    let deps = SilkSongDependencyHandler::new(&pm);

    if SilkSongReverseDependencyHandler::package_is_required(ctx, &package.package_full_name)
        && !force
    {
        return Ok(UninstallResult::PackageStillRequired);
    }

    pm.uninstall_package(ctx, package, progress, profile_path)?;

    let dep: Option<Vec<String>> = ctx
        .index
        .get_package_by_full_name_with_version(&package.package_full_name_with_version)
        .map(|p| p.dependencies);

    match dep {
        Some(dep) => {
            let still_required = deps
                .uninstall_dependencies(ctx, dep, force, progress, profile_path)
                .map_err(UninstallError::DependencyErrors)?;

            if !still_required.is_empty() {
                return Ok(UninstallResult::PackageStillRequired);
            }
        }
        None => {}
    }

    Ok(UninstallResult::Uninstalled)
}

#[cfg(test)]
mod tests {
    use super::{UninstallResult, run};
    use crate::{
        packages::{SilkSongFlattenedPackage, SilkSongIndex, SilkSongInstalledPackageRecord},
        tracker::sk_package_tracker::SilkSongPackageTracker,
        util::{
            config::{Config, GameSwitcher},
            context::Context,
        },
    };

    fn package(name: &str, version: &str, dependencies: Vec<&str>) -> SilkSongFlattenedPackage {
        SilkSongFlattenedPackage {
            package_full_name: name.to_string(),
            owner: "Author".to_string(),
            package_full_name_with_version: format!("{name}-{version}"),
            description: "desc".to_string(),
            download_url: "https://example.test/mod.zip".to_string(),
            version_number: version.to_string(),
            dependencies: dependencies.into_iter().map(str::to_string).collect(),
        }
    }

    fn context_with_dependency_graph() -> (Context, SilkSongInstalledPackageRecord) {
        let target = package("Author-Dependency", "1.0.0", vec![]);
        let dependent = package("Author-Mod", "1.0.0", vec!["Author-Dependency-1.0.0"]);

        let mut tracker = SilkSongPackageTracker::new();
        tracker.add(
            &target,
            std::path::Path::new("/mods/Author-Dependency-1.0.0"),
        );
        tracker.add(&dependent, std::path::Path::new("/mods/Author-Mod-1.0.0"));

        let mut index = SilkSongIndex::new();
        index.packages_by_full_name = vec![target.clone(), dependent.clone()]
            .into_iter()
            .map(|pkg| (pkg.package_full_name_with_version.clone(), pkg))
            .collect();

        let tracked_target = tracker.get("Author-Dependency").unwrap().clone();

        (
            Context {
                config: Config {
                    game_switcher: GameSwitcher::SilkSong,
                    sk_default_profile: None,
                    hk_default_profile: None,
                    hollow_knight_path: "/games/hk".into(),
                    silk_song_path: "/games/sk".into(),
                    index_path: "/config/index.json".into(),
                },
                tracker,
                index,
                black_list: vec![],
            },
            tracked_target,
        )
    }

    #[test]
    fn returns_package_still_required_before_touching_package_manager() {
        let (mut ctx, target) = context_with_dependency_graph();

        let result = run(
            &mut ctx,
            &target,
            false,
            &mut |_| {},
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, UninstallResult::PackageStillRequired));
    }
}
