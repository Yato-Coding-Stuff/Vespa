use std::path::PathBuf;

use semver::Version;
use thiserror::Error;

use crate::{
    cli::presenter::events::{InstallEvent, UninstallEvent, UpdateEvent},
    manager::sk_package_manager::SilkSongPackageManager,
    util::context::Context,
};

#[derive(Debug, Error)]
pub enum SilkSongDependencyHandlerError {
    #[error("Dependency error: {0}")]
    DependencyError(String),
    #[error("Dependency not found: {0}")]
    DependencyMissing(String),
    #[error("Dependency is still required: {0}")]
    DependencyStillRequired(String),
    #[error("Failed to parse version: {0}")]
    VersionParseError(String),
    #[error("Failed to install: {0}")]
    InstallError(String),
    #[error("Failed to uninstall: {0}")]
    UninstallError(String),
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

        if !errors.is_empty() {
            return Err(errors);
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
            .get_package_by_full_name_with_version(&dependency)
            .ok_or_else(|| SilkSongDependencyHandlerError::DependencyMissing(dependency.clone()))?
            .clone();

        let parsed_required_version = Version::parse(&package.version_number).map_err(|_| {
            SilkSongDependencyHandlerError::VersionParseError(package.version_number.clone())
        })?;

        if let Some(installed) = ctx.tracker.get(&package.package_full_name) {
            let installed_version = installed
                .version_number
                .as_deref() // Option<&str>
                .unwrap_or("0.0.0") // default if missing
                .parse::<Version>() // parse to Version
                .unwrap_or_else(|_| Version::new(0, 0, 0)); // fallback if parse fails

            if installed_version >= parsed_required_version {
                progress(InstallEvent::DependencyAlreadyInstalled);
                return Ok(());
            } else {
                progress(InstallEvent::UpdatingDependency {
                    name: package.package_full_name.clone(),
                    old_version: installed_version.to_string(),
                    new_version: package.version_number.clone(),
                });
                self.package_manager
                    .install_package(ctx, &package, progress, profile_path)
                    .map_err(|e| {
                        SilkSongDependencyHandlerError::InstallError(format!(
                            "Failed to update {}: {:?}",
                            package.package_full_name_with_version, e
                        ))
                    })?;
                return Ok(());
            }
        }

        progress(InstallEvent::InstallingDependency {
            name: package.package_full_name_with_version.clone(),
        });
        self.package_manager
            .install_package(ctx, &package, progress, profile_path)
            .map_err(|e| {
                SilkSongDependencyHandlerError::InstallError(format!(
                    "Failed to install {}: {:?}",
                    package.package_full_name_with_version, e
                ))
            })?;

        Ok(())
    }

    pub fn uninstall_dependencies<F: FnMut(UninstallEvent)>(
        &self,
        ctx: &mut Context,
        dependencies: Vec<String>,
        force: bool,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<Vec<String>, Vec<SilkSongDependencyHandlerError>> {
        progress(UninstallEvent::UninstallingDependencies {
            dependencies: dependencies.clone(),
        });

        let mut errors: Vec<SilkSongDependencyHandlerError> = Vec::new();
        let mut still_required = Vec::new();
        for dependency in dependencies {
            if SilkSongReverseDependencyHandler::dependency_is_required(ctx, &dependency) && !force
            {
                still_required.push(dependency.clone());
                continue;
            }

            match self.uninstall_single_dependency(ctx, dependency, progress, profile_path) {
                Ok(_) => (),
                Err(e) => errors.push(e),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(still_required)
    }

    fn uninstall_single_dependency<F: FnMut(UninstallEvent)>(
        &self,
        ctx: &mut Context,
        dependency: String,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<(), SilkSongDependencyHandlerError> {
        let package = ctx
            .index
            .get_package_by_full_name_with_version(&dependency)
            .ok_or_else(|| SilkSongDependencyHandlerError::DependencyMissing(dependency.clone()))?
            .clone();

        let installed_pkg = match ctx.tracker.get(&package.package_full_name) {
            Some(installed_pkg) => installed_pkg.clone(),
            None => {
                progress(UninstallEvent::DependencyAlreadyUninstalled {
                    name: dependency.clone(),
                });
                return Ok(());
            }
        };

        let is_required = ctx.tracker.get_all().values().any(|installed_pkg| {
            match ctx.index.get_package_by_full_name_with_version(
                &installed_pkg.package_full_name_with_version,
            ) {
                Some(installed_package_info) => installed_package_info
                    .dependencies
                    .iter()
                    .any(|dep| dep == &dependency),
                None => false,
            }
        });

        if is_required {
            return Err(SilkSongDependencyHandlerError::DependencyStillRequired(
                dependency.clone(),
            ));
        }

        self.package_manager
            .uninstall_package(ctx, &installed_pkg, progress, profile_path)
            .map_err(|e| {
                SilkSongDependencyHandlerError::UninstallError(format!(
                    "Failed to uninstall {}: {:?}",
                    package.package_full_name_with_version, e
                ))
            })?;

        Ok(())
    }

    pub fn update_dependencies<F: FnMut(UpdateEvent)>(
        &self,
        ctx: &mut Context,
        dependencies: Vec<String>,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<(), Vec<SilkSongDependencyHandlerError>> {
        progress(UpdateEvent::UpdatingDependencies {
            dependencies: dependencies.clone(),
        });

        let mut errors: Vec<SilkSongDependencyHandlerError> = Vec::new();

        for dependency in dependencies {
            if let Err(e) =
                self.update_single_dependency(ctx, dependency.clone(), progress, profile_path)
            {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn update_single_dependency<F: FnMut(UpdateEvent)>(
        &self,
        ctx: &mut Context,
        dependency: String,
        progress: &mut F,
        profile_path: &PathBuf,
    ) -> Result<(), SilkSongDependencyHandlerError> {
        // Look up package in index
        let package = ctx
            .index
            .get_package_by_full_name_with_version(&dependency)
            .ok_or_else(|| SilkSongDependencyHandlerError::DependencyMissing(dependency.clone()))?
            .clone();

        let package_newest_version = ctx
            .index
            .get_latest_package_by_package_name(&package.package_full_name)
            .ok_or_else(|| SilkSongDependencyHandlerError::DependencyMissing(dependency.clone()))?
            .clone();

        // Parse required version
        let required_version =
            Version::parse(&package_newest_version.version_number).map_err(|_| {
                SilkSongDependencyHandlerError::VersionParseError(package.version_number.clone())
            })?;

        // Check installed version
        if let Some(installed) = ctx.tracker.get(&package.package_full_name) {
            let installed_version = installed
                .version_number
                .as_deref()
                .unwrap_or("0.0.0")
                .parse::<semver::Version>()
                .unwrap_or_else(|_| semver::Version::new(0, 0, 0));

            if installed_version >= required_version {
                progress(UpdateEvent::DependencyAlreadyNewestVersion {
                    name: package.package_full_name.clone(),
                });
                return Ok(());
            } else {
                progress(UpdateEvent::UpdatingDependency {
                    name: package.package_full_name.clone(),
                    old_version: installed_version.to_string(),
                    new_version: package.version_number.clone(),
                });
            }
        } else {
            progress(UpdateEvent::InstallingDependency {
                name: package.package_full_name.clone(),
            });
        }

        // Install or upgrade dependency
        self.package_manager
            .update_package(ctx, &package_newest_version, progress, profile_path)
            .map_err(|e| {
                SilkSongDependencyHandlerError::InstallError(format!(
                    "Failed to install/update {}: {:?}",
                    package.package_full_name_with_version, e
                ))
            })?;

        Ok(())
    }
}

pub struct SilkSongReverseDependencyHandler;

impl SilkSongReverseDependencyHandler {
    pub fn package_is_required(ctx: &Context, target: &str) -> bool {
        ctx.tracker.get_all().values().any(|installed_pkg| {
            match ctx.index.get_package_by_full_name_with_version(
                &installed_pkg.package_full_name_with_version,
            ) {
                Some(installed_info) => installed_info.dependencies.iter().any(|dep| {
                    let dep_name = dep.rsplitn(2, '-').nth(1).unwrap_or(dep);
                    dep_name == target
                }),
                None => false,
            }
        })
    }

    pub fn dependency_is_required(ctx: &Context, target: &str) -> bool {
        ctx.tracker.get_all().values().any(|installed_pkg| {
            match ctx.index.get_package_by_full_name_with_version(
                &installed_pkg.package_full_name_with_version,
            ) {
                Some(installed_package_info) => installed_package_info
                    .dependencies
                    .iter()
                    .any(|dep| dep == target),
                None => false,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SilkSongReverseDependencyHandler;
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

    fn context_with_packages(
        tracked: Vec<SilkSongInstalledPackageRecord>,
        indexed: Vec<SilkSongFlattenedPackage>,
    ) -> Context {
        let mut tracker = SilkSongPackageTracker::new();
        for package in tracked {
            tracker.add(
                &SilkSongFlattenedPackage {
                    package_full_name: package.package_full_name.clone(),
                    owner: "Author".to_string(),
                    package_full_name_with_version: package.package_full_name_with_version.clone(),
                    description: "desc".to_string(),
                    download_url: "https://example.test/mod.zip".to_string(),
                    version_number: package
                        .version_number
                        .clone()
                        .unwrap_or("0.0.0".to_string()),
                    dependencies: vec![],
                },
                &package.file_path,
            );
        }

        let mut index = SilkSongIndex::new();
        index.packages_by_full_name = indexed
            .iter()
            .cloned()
            .map(|pkg| (pkg.package_full_name_with_version.clone(), pkg))
            .collect();

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
        }
    }

    #[test]
    fn package_is_required_checks_dependency_by_package_name() {
        let tracked = vec![SilkSongInstalledPackageRecord {
            package_full_name_with_version: "Author-ModA-1.0.0".to_string(),
            package_full_name: "Author-ModA".to_string(),
            version_number: Some("1.0.0".to_string()),
            file_path: "/mods/Author-ModA-1.0.0".into(),
        }];
        let indexed = vec![package(
            "Author-ModA",
            "1.0.0",
            vec!["Author-Dependency-2.1.0"],
        )];
        let ctx = context_with_packages(tracked, indexed);

        assert!(SilkSongReverseDependencyHandler::package_is_required(
            &ctx,
            "Author-Dependency"
        ));
        assert!(!SilkSongReverseDependencyHandler::package_is_required(
            &ctx,
            "SomethingElse"
        ));
    }

    #[test]
    fn dependency_is_required_checks_full_dependency_identifier() {
        let tracked = vec![SilkSongInstalledPackageRecord {
            package_full_name_with_version: "Author-ModA-1.0.0".to_string(),
            package_full_name: "Author-ModA".to_string(),
            version_number: Some("1.0.0".to_string()),
            file_path: "/mods/Author-ModA-1.0.0".into(),
        }];
        let indexed = vec![package(
            "Author-ModA",
            "1.0.0",
            vec!["Author-Dependency-2.1.0"],
        )];
        let ctx = context_with_packages(tracked, indexed);

        assert!(SilkSongReverseDependencyHandler::dependency_is_required(
            &ctx,
            "Author-Dependency-2.1.0"
        ));
        assert!(!SilkSongReverseDependencyHandler::dependency_is_required(
            &ctx,
            "Author-Dependency-1.0.0"
        ));
    }
}
