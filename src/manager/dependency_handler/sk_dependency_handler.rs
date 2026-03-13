use std::path::PathBuf;

use semver::Version;
use thiserror::Error;

use crate::{
    cli::presenter::events::install_event::InstallEvent,
    manager::sk_package_manager::SilkSongPackageManager, util::context::Context,
};

#[derive(Debug, Error)]
pub enum SilkSongDependencyHandlerError {
    #[error("Dependency error: {0}")]
    DependencyError(String),
    #[error("Dependency not found: {0}")]
    DependencyMissing(String),
    #[error("Failed to parse version: {0}")]
    VersionParseError(String),
    #[error("Failed to install: {0}")]
    InstallError(String),
}

pub struct SilkSongDependencyHandler<'pm> {
    pub package_manager: &'pm SilkSongPackageManager,
}

impl<'pm> SilkSongDependencyHandler<'pm> {
    pub fn new(pm: &'pm SilkSongPackageManager) -> Self {
        SilkSongDependencyHandler {
            package_manager: pm,
        }
    }

    pub fn handle_dependencies<F: FnMut(InstallEvent)>(
        &self,
        ctx: &mut Context,
        dependencies: Vec<String>,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<(), Vec<SilkSongDependencyHandlerError>> {
        progress(InstallEvent::InstallingDependencies {
            dependencies: dependencies.clone(),
        });

        let mut errors: Vec<SilkSongDependencyHandlerError> = Vec::new();
        for dependency in dependencies {
            match self.handle_single_dependency(ctx, dependency, progress, profile_path) {
                Ok(_) => (),
                Err(e) => errors.push(e),
            }
        }
        Ok(())
    }

    fn handle_single_dependency<F: FnMut(InstallEvent)>(
        &self,
        ctx: &mut Context,
        dependency: String,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<(), SilkSongDependencyHandlerError> {
        let package = ctx
            .index
            .get_package_by_full_name(&dependency)
            .ok_or_else(|| SilkSongDependencyHandlerError::DependencyMissing(dependency.clone()))?
            .clone();

        let parsed_required_version = Version::parse(&package.version_number).map_err(|_| {
            SilkSongDependencyHandlerError::VersionParseError(package.version_number.clone())
        })?;

        if let Some(installed) = ctx
            .tracker
            .get_installed_package_record(&package.package_name)
        {
            if installed.version_number >= parsed_required_version {
                progress(InstallEvent::DependencyAlreadyInstalled);
                return Ok(());
            } else {
                progress(InstallEvent::UpdatingDependency {
                    name: package.package_name.clone(),
                    old_version: installed.version_number.to_string(),
                    new_version: package.version_number.clone(),
                });
                self.package_manager
                    .install_package(ctx, &package, progress, profile_path)
                    .map_err(|e| {
                        SilkSongDependencyHandlerError::InstallError(format!(
                            "Failed to update {}: {:?}",
                            package.package_name, e
                        ))
                    })?;
                return Ok(());
            }
        }

        progress(InstallEvent::InstallingDependency {
            name: package.package_name_with_version.clone(),
        });
        self.package_manager
            .install_package(ctx, &package, progress, profile_path)
            .map_err(|e| {
                SilkSongDependencyHandlerError::InstallError(format!(
                    "Failed to install {}: {:?}",
                    package.package_name, e
                ))
            })?;

        Ok(())
    }
}
