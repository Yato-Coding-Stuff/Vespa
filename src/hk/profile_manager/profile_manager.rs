use std::{
    fs::{self, remove_dir_all},
    path::PathBuf,
};

use crate::base::{
    cli::presenter::{events::ProfileManagerEvent, presenter::Presenter},
    profile_manager::profile_manager::{ProfileManager, ProfileManagerError},
    util::{config::GameSwitcher, context::Context},
};

pub struct HkProfileManager {
    base_dir: PathBuf,
}

impl HkProfileManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl ProfileManager for HkProfileManager {
    fn create_profile(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<PathBuf, ProfileManagerError> {
        let profile_dir = self.get_profile_path(game, profile);

        if !profile_dir.exists() {
            presenter.display(&ProfileManagerEvent::CreatingProfileDirectory {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            });
            fs::create_dir_all(&profile_dir)?;
        } else {
            presenter.display(&ProfileManagerEvent::ProfileDirectoryAlreadyExists {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            });
        }

        self.set_profile_as_default(ctx, presenter, game, profile)?;

        Ok(profile_dir)
    }

    fn list_profiles(&self, game: &GameSwitcher) -> Result<Vec<String>, ProfileManagerError> {
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

    fn delete_profile(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<(), ProfileManagerError> {
        let profile_dir = self.get_profile_path(game, profile);

        if profile_dir.exists() {
            presenter.display(&ProfileManagerEvent::DeletingProfileDirectory {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            });
            remove_dir_all(&profile_dir)
                .map_err(ProfileManagerError::ProfileCreationOrRemovalError)?;

            if ctx.config.get_default_profile(game).as_deref() == Some(profile) {
                ctx.config.clear_default_profile(game)?;
            }
        } else {
            presenter.display(&ProfileManagerEvent::ProfileDirectoryDoesNotExist {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            });
        }

        Ok(())
    }

    fn set_profile_as_default(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<(), ProfileManagerError> {
        let profile_path = self.get_profile_path(game, profile);

        if !profile_path.exists() {
            return Err(ProfileManagerError::DefaultProfileDoesNotExist(
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
}
