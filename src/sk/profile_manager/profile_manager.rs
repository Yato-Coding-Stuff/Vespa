use std::{
    fs::{self, remove_dir_all},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    base::{
        cli::presenter::{
            events::{InstallEvent, ProfileManagerEvent},
            presenter::Presenter,
        },
        manager::package_manager::PackageManager,
        packages::PackageLoader,
        profile_manager::profile_manager::{ProfileManager, ProfileManagerError},
        util::{config::GameSwitcher, context::Context},
    },
    sk::manager::bepinex_manager_extension::BepInExPackageManagerExt,
};

pub struct SkProfileManager {
    base_dir: PathBuf,
    package_manager: Rc<PackageManager>,
    package_loader: PackageLoader,
    blacklist: &'static [&'static str],
}

impl ProfileManager for SkProfileManager {
    fn create_profile(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        game: &GameSwitcher,
        profile: &str,
    ) -> Result<PathBuf, ProfileManagerError> {
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
        let mut presenter = |event: ProfileManagerEvent| presenter.display(&event);

        let profile_dir = self.get_profile_path(game, profile);

        if profile_dir.exists() {
            presenter(ProfileManagerEvent::DeletingProfileDirectory {
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
            presenter(ProfileManagerEvent::ProfileDirectoryDoesNotExist {
                name: profile.to_string(),
                game: game.to_string(),
                path: profile_dir.to_string_lossy().to_string(),
            })
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

impl SkProfileManager {
    pub fn new(
        base_dir: PathBuf,
        package_manager: Rc<PackageManager>,
        package_loader: PackageLoader,
        blacklist: &'static [&'static str],
    ) -> Self {
        Self {
            base_dir,
            package_manager,
            package_loader,
            blacklist,
        }
    }

    fn install_bepinex(
        &self,
        ctx: &mut Context,
        presenter: &mut Presenter,
        profile_dir: &Path,
    ) -> Result<(), ProfileManagerError> {
        let mut install_progress = |event: InstallEvent| {
            presenter.display(&event);
        };

        let packages = (self.package_loader)(self.blacklist)?;
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

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        rc::Rc,
    };

    use tempfile::tempdir;

    use super::SkProfileManager;
    use crate::base::{
        cli::presenter::presenter::Presenter,
        manager::package_manager::PackageManager,
        packages::{Index, IndexError, PackageRecord},
        profile_manager::profile_manager::{ProfileManager, ProfileManagerError},
        tracker::package_tracker::PackageTracker,
        util::{
            config::{Config, GameSwitcher},
            context::Context,
        },
    };

    const TEST_BLACKLIST: &[&str] = &[];

    fn empty_package_loader(_: &[&str]) -> Result<Vec<PackageRecord>, IndexError> {
        Ok(Vec::new())
    }

    fn manager(base_dir: PathBuf) -> SkProfileManager {
        SkProfileManager::new(
            base_dir,
            Rc::new(PackageManager::new(TEST_BLACKLIST, Path::to_path_buf)),
            empty_package_loader,
            TEST_BLACKLIST,
        )
    }

    fn test_context() -> Context {
        Context {
            config: Config {
                game_switcher: GameSwitcher::SilkSong,
                default_profile: None,
                hk_default_profile: None,
                hollow_knight_path: "/games/hk".into(),
                silk_song_path: "/games/sk".into(),
                index_path: "/config/index.json".into(),
            },
            tracker: PackageTracker::new(),
            index: Index::new(),
        }
    }

    #[test]
    fn list_profiles_returns_sorted_names() {
        let temp_dir = tempdir().unwrap();
        let manager = manager(temp_dir.path().to_path_buf());
        std::fs::create_dir_all(temp_dir.path().join("SK").join("zeta")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("SK").join("alpha")).unwrap();

        let profiles = manager.list_profiles(&GameSwitcher::SilkSong).unwrap();

        assert_eq!(profiles, vec!["alpha".to_string(), "zeta".to_string()]);
    }

    #[test]
    fn list_profiles_returns_empty_when_game_dir_does_not_exist() {
        let temp_dir = tempdir().unwrap();
        let manager = manager(temp_dir.path().to_path_buf());

        let profiles = manager.list_profiles(&GameSwitcher::HollowKnight).unwrap();

        assert!(profiles.is_empty());
    }

    #[test]
    fn set_profile_as_default_errors_when_profile_is_missing() {
        let temp_dir = tempdir().unwrap();
        let manager = manager(temp_dir.path().to_path_buf());
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
            Err(ProfileManagerError::DefaultProfileDoesNotExist(name)) if name == "missing"
        ));
    }
}
