use std::{
    fs::{self, remove_dir_all},
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    cli::presenter::{
        events::{InstallEvent, ProfileManagerEvent},
        presenter::Presenter,
    },
    manager::sk_package_manager::{SilkSongPackageManager, SilkSongPackageManagerError},
    packages::SilkSongIndexError,
    util::{
        config::{ConfigError, GameSwitcher},
        context::Context,
    },
};

#[derive(Debug, Error)]
pub enum SilkSongProfileManagerError {
    #[error("Error creating/removing profile directory: {0}")]
    ProfileCreationOrRemovalError(#[from] std::io::Error),
    #[error("Error looking up BepInEx")]
    BepInExLookUpError,
    #[error("Error installing BepInEx: {0}")]
    BepInExInstallError(#[from] SilkSongPackageManagerError),
    #[error("Failed setting default profile: {0}")]
    SetDefaultProfileError(#[from] ConfigError),
    #[error("Default profile does not exist: {0}")]
    DefaultProfileDoesNotExist(String),
    #[error(transparent)]
    IndexInitializationError(#[from] SilkSongIndexError),
}

pub struct SilkSongProfileManager {
    base_dir: PathBuf,
}

impl SilkSongProfileManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn create_profile(
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
            fs::create_dir_all(&profile_dir)?;
        } else {
            profile_progress(ProfileManagerEvent::ProfileDirectoryAlreadyExists {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            })
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

        self.set_profile_as_default(ctx, presenter, game, profile)?;

        Ok(profile_dir)
    }

    pub fn list_profiles(
        &self,
        game: &GameSwitcher,
    ) -> Result<Vec<String>, SilkSongProfileManagerError> {
        let profiles_dir = self.get_profiles_dir(game);

        if !profiles_dir.exists() {
            return Ok(Vec::new());
        }

        let mut profiles = fs::read_dir(&profiles_dir)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                entry
                    .file_type()
                    .ok()
                    .filter(|file_type| file_type.is_dir())
                    .and_then(|_| entry.file_name().into_string().ok())
            })
            .collect::<Vec<_>>();

        profiles.sort();

        Ok(profiles)
    }

    pub fn delete_profile(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<(), SilkSongProfileManagerError> {
        let mut presenter = |event: ProfileManagerEvent| presenter.display(&event);

        let profile_dir = self.get_profile_path(game, profile);

        if profile_dir.exists() {
            presenter(ProfileManagerEvent::DeletingProfileDirectory {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            });
            remove_dir_all(&profile_dir)
                .map_err(SilkSongProfileManagerError::ProfileCreationOrRemovalError)?;

            if ctx.config.get_default_profile(game).as_deref() == Some(profile) {
                ctx.config.clear_default_profile(game)?;
            }
        } else {
            presenter(ProfileManagerEvent::ProfileDirectoryDoesNotExist {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            })
        }

        Ok(())
    }

    pub fn set_profile_as_default(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<(), SilkSongProfileManagerError> {
        let profile_path = self.get_profile_path(game, profile);

        if !profile_path.exists() {
            return Err(SilkSongProfileManagerError::DefaultProfileDoesNotExist(
                profile.to_string(),
            ));
        }

        presenter.display(&ProfileManagerEvent::SettingProfileAsDefault {
            name: profile.to_string(),
            game: game.to_string(),
        });
        ctx.config.set_default_profile(game, profile.to_string())?;
        Ok(())
    }

    fn get_profile_path(&self, game: &GameSwitcher, profile: &str) -> PathBuf {
        self.get_profiles_dir(game).join(profile)
    }

    fn get_profiles_dir(&self, game: &GameSwitcher) -> PathBuf {
        self.base_dir.join(game.profile_dir_name())
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

        ctx.index.initialize(&ctx.black_list)?;

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

#[cfg(test)]
mod tests {
    use super::{SilkSongProfileManager, SilkSongProfileManagerError};
    use crate::{
        cli::presenter::presenter::Presenter,
        packages::SilkSongIndex,
        tracker::sk_package_tracker::SilkSongPackageTracker,
        util::{
            config::{Config, GameSwitcher},
            context::Context,
        },
    };
    use tempfile::tempdir;

    fn test_context() -> Context {
        Context {
            config: Config {
                game_switcher: GameSwitcher::SilkSong,
                sk_default_profile: None,
                hk_default_profile: None,
                hollow_knight_path: "/games/hk".into(),
                silk_song_path: "/games/sk".into(),
                index_path: "/config/index.json".into(),
            },
            tracker: SilkSongPackageTracker::new(),
            index: SilkSongIndex::new(),
            black_list: vec!["BepInEx-BepInExPack_Silksong"],
        }
    }

    #[test]
    fn list_profiles_returns_sorted_names() {
        let temp_dir = tempdir().unwrap();
        let manager = SilkSongProfileManager::new(temp_dir.path().to_path_buf());
        std::fs::create_dir_all(temp_dir.path().join("SK").join("zeta")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("SK").join("alpha")).unwrap();

        let profiles = manager.list_profiles(&GameSwitcher::SilkSong).unwrap();

        assert_eq!(profiles, vec!["alpha".to_string(), "zeta".to_string()]);
    }

    #[test]
    fn list_profiles_returns_empty_when_game_dir_does_not_exist() {
        let temp_dir = tempdir().unwrap();
        let manager = SilkSongProfileManager::new(temp_dir.path().to_path_buf());

        let profiles = manager.list_profiles(&GameSwitcher::HollowKnight).unwrap();

        assert!(profiles.is_empty());
    }

    #[test]
    fn set_profile_as_default_errors_when_profile_is_missing() {
        let temp_dir = tempdir().unwrap();
        let manager = SilkSongProfileManager::new(temp_dir.path().to_path_buf());
        let mut ctx = test_context();
        let mut presenter = Presenter::new();

        let result = manager.set_profile_as_default(
            &mut ctx,
            &mut presenter,
            &GameSwitcher::SilkSong,
            "missing",
        );

        assert!(matches!(
            result,
            Err(SilkSongProfileManagerError::DefaultProfileDoesNotExist(name)) if name == "missing"
        ));
    }
}
