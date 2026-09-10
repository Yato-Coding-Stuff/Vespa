use std::path::Path;

use crate::{
    base::{
        cli::presenter::{events::InstallEvent, presenter::Presenter},
        manager::{
            package_layout::PackageLayout,
            package_manager::{PackageManager, PackageManagerError},
        },
        util::context::Context,
    },
    sk::{
        manager::bepinex_manager_extension::{BEPINEX_PACKAGE_NAME, BepInExPackageManagerExt},
        packages::PackageRecord,
    },
};

#[derive(Clone)]
pub struct SkPackageLayout;

impl PackageLayout for SkPackageLayout {
    fn package_root(&self, profile_path: &std::path::Path) -> std::path::PathBuf {
        profile_path.join("BepInEx").join("plugins")
    }

    fn install_special_package(
        &self,
        package_manager: &PackageManager,
        ctx: &mut Context,
        package: &PackageRecord,
        presenter: &mut Presenter,
        profile_path: &Path,
    ) -> Result<bool, PackageManagerError> {
        if package.name != BEPINEX_PACKAGE_NAME {
            return Ok(false);
        }

        let progress = &mut |event: InstallEvent| {
            presenter.display(&event);
        };
        package_manager.install_bepinex(ctx, package, progress, profile_path)?;
        Ok(true)
    }
}
