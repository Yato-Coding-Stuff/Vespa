use std::path::PathBuf;

use crate::{
    cli::presenter::events::InstallEvent,
    manager::{
        sk_dependency_handler::{
            SilkSongDependencyHandler, SilkSongDependencyHandlerError,
        },
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
