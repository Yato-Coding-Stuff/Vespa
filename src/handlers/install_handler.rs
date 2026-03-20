use std::path::PathBuf;

use crate::{
    cli::presenter::events::install_event::InstallEvent,
    manager::{
        dependency_handler::sk_dependency_handler::{
            SilkSongDependencyHandler, SilkSongDependencyHandlerError,
        },
        sk_package_manager::SilkSongPackageManagerError,
    },
    packages::SilkSongFlattenedPackage,
};
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
    #[error("Mod not found: {0}")]
    ModNotFound(String),

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

    let requested_version = package.version_number.parse().unwrap();

    if !force {
        match ctx
            .tracker
            .get_installed_package_record(&package.package_name)
        {
            None => {} // Not installed, proceed normally
            Some(installed) => {
                // Compare versions
                let result = if installed.version_number == requested_version {
                    InstallResult::AlreadyInstalled
                } else if installed.version_number > requested_version {
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

    ctx.tracker.save(ctx.config.index_path.to_str().unwrap());

    progress(InstallEvent::Finished);
    Ok(InstallResult::Installed)
}
