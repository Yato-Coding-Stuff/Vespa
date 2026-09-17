use std::path::Path;

use crate::
    base::{
        cli::presenter::presenter::Presenter,
        profile_manager::profile_manager::{ProfileInitializer, ProfileManagerError},
        util::{config::GameSwitcher, context::Context},
    }
;

pub struct HkProfileInitializer {}

impl ProfileInitializer for HkProfileInitializer {
    fn initialize(
        &self,
        ctx: &mut Context,
        game: &GameSwitcher,
        presenter: &mut Presenter,
        profile_dir: &Path,
        profile: &str,
    ) -> Result<(), ProfileManagerError> {
        todo!();
    }
}
