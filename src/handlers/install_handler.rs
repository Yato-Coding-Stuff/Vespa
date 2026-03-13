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

    if ctx.tracker.is_installed(&package.package_name) && !force {
        return Ok(InstallResult::AlreadyInstalled);
    }

    dependency_manager
        .handle_dependencies(ctx, package.dependencies.clone(), progress, profile_path)
        .map_err(InstallError::DependencyErrors)?;
    pm.install_package(ctx, &package, progress, profile_path)?;

    ctx.tracker.save(ctx.config.index_path.to_str().unwrap());

    progress(InstallEvent::Finished);
    Ok(InstallResult::Installed)
}
