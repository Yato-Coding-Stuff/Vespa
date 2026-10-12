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
    hk::manager::api_manager_extension::{HK_API_PACKAGE_NAME, HkApiPackageManagerExt},
    sk::packages::PackageRecord,
};

#[derive(Clone)]
pub struct HkPackageLayout;

impl PackageLayout for HkPackageLayout {
    fn package_root(&self, profile_path: &std::path::Path) -> std::path::PathBuf {
        profile_path.join("mods")
    }

    fn install_special_package(
        &self,
        package_manager: &PackageManager,
        ctx: &mut Context,
        package: &PackageRecord,
        presenter: &mut Presenter,
        profile_path: &Path,
    ) -> Result<bool, PackageManagerError> {
        if package.name != HK_API_PACKAGE_NAME {
            return Ok(false);
        }

        let progress = &mut |event: InstallEvent| {
            presenter.display(&event);
        };
        package_manager.install_hk_api(ctx, package, progress, profile_path)?;
        Ok(true)
    }
}
