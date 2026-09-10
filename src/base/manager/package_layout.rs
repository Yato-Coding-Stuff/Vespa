use std::path::{Path, PathBuf};

use crate::{
    base::{
        cli::presenter::presenter::Presenter,
        manager::package_manager::{PackageManager, PackageManagerError},
        packages::package::InstalledPackageRecord,
        util::context::Context,
    },
    sk::packages::PackageRecord,
};

pub trait PackageLayout {
    fn package_root(&self, profile_path: &Path) -> PathBuf;

    fn scan_special_packages(&self, _profile_path: &Path) -> Vec<InstalledPackageRecord> {
        Vec::new()
    }

    fn install_special_package(
        &self,
        package_manager: &PackageManager,
        ctx: &mut Context,
        package: &PackageRecord,
        presenter: &mut Presenter,
        profile_path: &Path,
    ) -> Result<bool, PackageManagerError>;
}
