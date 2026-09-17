use std::{
    fs::{self, create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::
    base::{
        cli::presenter::{events::ProfileManagerEvent, presenter::Presenter},
        manager::package_manager::PackageManagerError,
        packages::IndexError,
        util::{
            config::{ConfigError, GameSwitcher},
            context::Context,
        },
    }
;

pub trait ProfileInitializer {
    fn initialize(
        &self,
        ctx: &mut Context,
        game: &GameSwitcher,
        presenter: &mut Presenter,
        profile_dir: &Path,
        profile: &str,
    ) -> Result<(), ProfileManagerError>;
}

#[derive(Debug, Error)]
pub enum ProfileManagerError {
    #[error("Error creating/removing profile directory: {0}")]
    ProfileCreationOrRemovalError(#[from] std::io::Error),
    #[error("failed to initialize profile: {0}")]
    InitializationError(#[from] PackageManagerError),
    #[error("Failed setting default profile: {0}")]
    SetDefaultProfileError(#[from] ConfigError),
    #[error("Default profile does not exist: {0}")]
    DefaultProfileDoesNotExist(String),
    #[error(transparent)]
    IndexInitializationError(#[from] IndexError),
}

pub struct ProfileManager {
    base_dir: PathBuf,
    initializer: Box<dyn ProfileInitializer>,
}

impl ProfileManager {
    pub fn new(base_dir: PathBuf, initializer: Box<dyn ProfileInitializer>) -> Self {
        Self {
            base_dir,
            initializer,
        }
    }

    pub fn create_profile(
        &self,
        ctx: &mut Context,
        game: &GameSwitcher,
        presenter: &mut Presenter,
        profile: &str,
    ) -> Result<PathBuf, ProfileManagerError> {
        let profile_dir = self.get_profile_path(game, profile);

        if !profile_dir.exists() {
            presenter.display(&ProfileManagerEvent::CreatingProfileDirectory {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            });
            create_dir_all(&profile_dir)?;
        } else {
            presenter.display(&ProfileManagerEvent::ProfileDirectoryAlreadyExists {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            });
        }

        self.initializer
            .initialize(ctx, game, presenter, &profile_dir, profile)?;

        self.set_profile_as_default(ctx, game, presenter, profile)?;

        Ok(profile_dir)
    }

    pub fn list_profiles(&self, game: &GameSwitcher) -> Result<Vec<String>, ProfileManagerError> {
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
        game: &GameSwitcher,
        presenter: &mut Presenter,
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

    pub fn set_profile_as_default(
        &self,
        ctx: &mut Context,
        game: &GameSwitcher,
        presenter: &mut Presenter,
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

    pub fn get_profile_path(&self, game: &GameSwitcher, profile: &str) -> PathBuf {
        self.get_profiles_dir(game).join(profile)
    }

    pub fn get_profiles_dir(&self, game: &GameSwitcher) -> PathBuf {
        self.base_dir.join(game.profile_dir_name())
    }
}
