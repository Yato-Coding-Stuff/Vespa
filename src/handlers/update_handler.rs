use std::path::PathBuf;

use crate::{
    cli::presenter::{
        events::{InstallEvent, UninstallEvent, UpdateEvent},
        presenter::Presenter,
    },
    manager::{
        sk_dependency_handler::{SilkSongDependencyHandler, SilkSongDependencyHandlerError},
        sk_package_manager::SilkSongPackageManagerError,
    },
    packages::SilkSongFlattenedPackage,
};
use semver::Version;
use thiserror::Error;

use crate::{manager::sk_package_manager::SilkSongPackageManager, util::context::Context};

pub enum UpdateResult {
    NotInstalled,
    Updated,
    AlreadyNewestVersion,
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error(transparent)]
    ManagerError(#[from] SilkSongPackageManagerError),

    #[error("Dependency errors: {0:?}")]
    DependencyErrors(Vec<SilkSongDependencyHandlerError>),
}

pub fn run(
    ctx: &mut Context,
    package: &SilkSongFlattenedPackage,
    progress: &mut Presenter,
    bulk_update: bool,
    profile_path: &PathBuf,
) -> Result<UpdateResult, UpdateError> {
    let update_progress = &mut |event: UpdateEvent| {
        progress.display(&event);
    };

    let pm: SilkSongPackageManager = SilkSongPackageManager::new();
    let dependency_manager: SilkSongDependencyHandler = SilkSongDependencyHandler::new(&pm);

    let requested_version = package.version_number.parse::<Version>().unwrap();

    match ctx.tracker.get(&package.package_full_name) {
        None => {
            return Ok(UpdateResult::NotInstalled);
        }
        Some(installed) => {
            let installed_version = installed
                .version_number
                .as_deref() // Option<&str>
                .unwrap_or("0.0.0") // default if missing
                .parse::<Version>() // parse to Version
                .unwrap_or_else(|_| Version::new(0, 0, 0)); // fallback if parse fails

            // Compare versions
            if installed_version == requested_version {
                return Ok(UpdateResult::AlreadyNewestVersion);
            } else if installed_version < requested_version {
                update_progress(UpdateEvent::UpdateMod {
                    name: package.package_full_name.clone(),
                    old_version: installed_version.to_string(),
                    new_version: package.version_number.clone(),
                });
            };
        }
    }

    pm.update_package(ctx, package, update_progress, profile_path)?;
    if !bulk_update {
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

#[cfg(test)]
mod tests {
    use super::{UpdateResult, run};
    use crate::{
        cli::presenter::presenter::Presenter,
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
    fn returns_not_installed_when_tracker_has_no_package() {
        let mut ctx = empty_context();
        let mut presenter = Presenter::new();

        let result = run(
            &mut ctx,
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
        let mut ctx = empty_context();
        let installed = package("1.0.0");
        ctx.tracker
            .add(&installed, std::path::Path::new("/mods/Author-Mod-1.0.0"));
        let mut presenter = Presenter::new();

        let result = run(
            &mut ctx,
            &installed,
            &mut presenter,
            false,
            &"/profiles/default".into(),
        )
        .unwrap();

        assert!(matches!(result, UpdateResult::AlreadyNewestVersion));
    }
}
