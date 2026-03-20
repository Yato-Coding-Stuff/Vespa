use std::{
    fs,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    cli::presenter::{
        events::{install_event::InstallEvent, profile_manager_event::ProfileManagerEvent},
        presenter::Presenter,
    },
    manager::sk_package_manager::{SilkSongPackageManager, SilkSongPackageManagerError},
    util::{config::GameSwitcher, context::Context},
};

#[derive(Debug, Error)]
pub enum SilkSongProfileManagerError {
    #[error("Error creating profile directory: {0}")]
    ProfileCreationError(#[from] std::io::Error),
    #[error("Error looking up BepInEx")]
    BepInExLookUpError,
    #[error("Error installing BepInEx: {0}")]
    BepInExInstallError(#[from] SilkSongPackageManagerError),
}

pub struct SilkSongProfileManager {
    base_dir: PathBuf,
}

impl SilkSongProfileManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn ensure_profile(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<PathBuf, SilkSongProfileManagerError> {
        let mut profile_progress = |event: ProfileManagerEvent| {
            presenter.display(&event);
        };

        let profile_dir = self.get_profile_path(game, profile);

        if !profile_dir.exists() {
            profile_progress(ProfileManagerEvent::CreatingProfileDirectory {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            });
            fs::create_dir_all(&profile_dir)
                .map_err(SilkSongProfileManagerError::ProfileCreationError)?;
        }

        let bepinex_dir = profile_dir.join("BepInEx");
        if !bepinex_dir.exists() {
            profile_progress(ProfileManagerEvent::InstallingBepInEx {
                name: profile.to_string(),
                game: game.to_string(),
                path: bepinex_dir.to_string_lossy().to_string(),
            });
            self.install_bepinex(ctx, presenter, &profile_dir)?;
        }

        Ok(profile_dir)
    }

    fn get_profile_path(&self, game: &GameSwitcher, profile: &str) -> PathBuf {
        let game_name = match game {
            GameSwitcher::HollowKnight => "HK",
            GameSwitcher::SilkSong => "SK",
        };

        self.base_dir.join(game_name).join(profile)
    }

    fn install_bepinex(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        profile_dir: &Path,
    ) -> Result<(), SilkSongProfileManagerError> {
        let mut install_progress = |event: InstallEvent| {
            presenter.display(&event);
        };

        let bepinex = ctx
            .index
            .get_latest_package_by_package_name("BepInEx-BepInExPack_Silksong")
            .unwrap();
        let pm = SilkSongPackageManager::new();
        pm.install_bepinex(ctx, &bepinex, &mut install_progress, profile_dir)?;
        install_progress(InstallEvent::Finished);

        Ok(())
    }
}
