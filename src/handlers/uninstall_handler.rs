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
