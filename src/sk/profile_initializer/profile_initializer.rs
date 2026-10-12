use std::path::Path;
use std::rc::Rc;

use crate::base::cli::presenter::events::ProfileManagerEvent;
use crate::base::manager::package_manager::PackageManager;
use crate::base::packages::PackageLoader;
use crate::base::util::context::Context;
use crate::sk::manager::bepinex_manager_extension::BepInExPackageManagerExt;

use crate::base::{
    cli::presenter::{events::InstallEvent, presenter::Presenter},
    profile_manager::profile_manager::{ProfileInitializer, ProfileManagerError},
};

pub struct SkProfileInitializer {
    pub package_manager: Rc<PackageManager>,
    pub package_loader: PackageLoader,
    pub blacklist: &'static [&'static str],
}

impl ProfileInitializer for SkProfileInitializer {
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

        let bepinex_dir = profile_dir.join("BepInEx");
        if !bepinex_dir.exists() {
            profile_progress(ProfileManagerEvent::InstallingBepInEx {
                name: profile.to_string(),
                path: bepinex_dir.to_string_lossy().to_string(),
            });
            self.install_bepinex(ctx, presenter, profile_dir)?;
        }
        Ok(())
    }
}

impl SkProfileInitializer {
    fn install_bepinex(
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

        let bepinex = ctx
            .index
            .get_latest_package_by_package_name("BepInEx-BepInExPack_Silksong")
            .unwrap();

        self.package_manager
            .install_bepinex(ctx, &bepinex, &mut install_progress, profile_dir)?;
        install_progress(InstallEvent::Finished);

        Ok(())
    }
}
