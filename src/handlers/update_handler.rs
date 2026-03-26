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
