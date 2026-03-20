use std::path::PathBuf;

use crate::{
    cli::presenter::events::uninstall_event::UninstallEvent,
    manager::{
        dependency_handler::sk_dependency_handler::{
            SilkSongDependencyHandler, SilkSongDependencyHandlerError,
        },
        dependency_handler::sk_reverse_dependency_handler::SilkSongReverseDependencyHandler,
        sk_package_manager::SilkSongPackageManagerError,
    },
    packages::SilkSongFlattenedPackage,
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
    package: &SilkSongFlattenedPackage,
    force: bool,
    progress: &mut F,
    profile_path: &PathBuf,
) -> Result<UninstallResult, UninstallError> {
    let pm = SilkSongPackageManager::new();
    let deps = SilkSongDependencyHandler::new(&pm);

    if ctx.tracker.get(&package.package_full_name).is_none() {
        return Ok(UninstallResult::NotInstalled);
    }

    if SilkSongReverseDependencyHandler::package_is_required(ctx, &package.package_full_name)
        && !force
    {
        return Ok(UninstallResult::PackageStillRequired);
    }

    pm.uninstall_package(ctx, package, progress, profile_path)?;

    let still_required = deps
        .uninstall_dependencies(
            ctx,
            package.dependencies.clone(),
            force,
            progress,
            profile_path,
        )
        .map_err(UninstallError::DependencyErrors)?;

    if !still_required.is_empty() {
        return Ok(UninstallResult::PackageStillRequired);
    }

    Ok(UninstallResult::Uninstalled)
}
