use std::path::PathBuf;

use thiserror::Error;

use crate::base::{
    cli::presenter::presenter::Presenter,
    manager::package_manager::PackageManagerError,
    packages::IndexError,
    util::{
        config::{ConfigError, GameSwitcher},
        context::Context,
    },
};

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

pub trait ProfileManager {
    fn create_profile(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<PathBuf, ProfileManagerError>;

    fn list_profiles(&self, game: &GameSwitcher) -> Result<Vec<String>, ProfileManagerError>;

    fn delete_profile(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<(), ProfileManagerError>;

    fn set_profile_as_default(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<(), ProfileManagerError>;

    fn get_profile_path(&self, game: &GameSwitcher, profile: &str) -> PathBuf;

    fn get_profiles_dir(&self, game: &GameSwitcher) -> PathBuf;
}
