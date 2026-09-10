use std::{path::Path, rc::Rc};

use crate::base::{
    cli::presenter::{
        events::{InstallEvent, ProfileManagerEvent},
        presenter::Presenter,
    },
    manager::package_manager::PackageManager,
    packages::PackageLoader,
    profile_manager::profile_manager::{ProfileInitializer, ProfileManagerError},
    util::context::Context,
};
use crate::hk::manager::api_manager_extension::{HK_API_PACKAGE_NAME, HkApiPackageManagerExt};

pub struct HkProfileInitializer {
    pub package_manager: Rc<PackageManager>,
    pub package_loader: PackageLoader,
    pub blacklist: &'static [&'static str],
}

impl ProfileInitializer for HkProfileInitializer {
    fn initialize(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        profile_dir: &Path,
        profile: &str,
    ) -> Result<(), ProfileManagerError> {
        let mut profile_progress = |event: ProfileManagerEvent| {
            presenter.display(&event);
        };

        let api_dir = profile_dir.join("api");
        if !api_dir.exists() {
            profile_progress(ProfileManagerEvent::InstallingModdingApi {
                name: profile.to_string(),
                path: api_dir.to_string_lossy().to_string(),
            });
            self.install_api(ctx, presenter, profile_dir)?;
        }
        Ok(())
    }
}

impl HkProfileInitializer {
    fn install_api(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        profile_dir: &Path,
    ) -> Result<(), ProfileManagerError> {
        let mut install_progress = |event: InstallEvent| {
            presenter.display(&event);
        };

        let packages = ((self.package_loader)(self.blacklist))?;
        ctx.index.replace(packages);

        let api = ctx
            .index
            .get_latest_package_by_package_name(HK_API_PACKAGE_NAME)
            .ok_or_else(|| {
                ProfileManagerError::RequiredPackageMissing(HK_API_PACKAGE_NAME.to_string())
            })?;

        self.package_manager
            .install_hk_api(ctx, &api, &mut install_progress, profile_dir)?;
        install_progress(InstallEvent::Finished);

        Ok(())
    }
}
