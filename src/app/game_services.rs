use std::rc::Rc;

use crate::{
    base::{
        manager::package_manager::PackageManager,
        packages::{PackageLoader, package_scanner::PackageScanner},
        profile_manager::profile_manager::ProfileManager,
        runner::game_runner::GameRunner,
        util::config::{Config, GameSwitcher},
    },
    hk::profile_manager::profile_manager::HkProfileManager,
    sk::{
        packages::package_scanner::SkPackageScanner,
        profile_manager::profile_manager::SkProfileManager, runner::game_runner::SkGameRunner,
    },
};

const SK_BLACKLIST: &[&str] = &["BepInEx-BepInExPack_Silksong"];

const HK_BLACKLIST: &[&str] = &[];

pub struct GameServices {
    pub package_loader: PackageLoader,
    pub package_scanner: Box<dyn PackageScanner>,
    pub package_manager: Rc<PackageManager>,
    pub profile_manager: Box<dyn ProfileManager>,
    pub game_runner: Box<dyn GameRunner>,
    pub blacklist: &'static [&'static str],
}

impl GameServices {
    pub fn for_game(game: &GameSwitcher) -> Self {
        match game {
            GameSwitcher::SilkSong => {
                let package_manager = Rc::new(PackageManager::new(SK_BLACKLIST));

                Self {
                    package_loader: crate::sk::packages::fetch_package_records,
                    package_scanner: Box::new(SkPackageScanner),
                    package_manager: package_manager.clone(),
                    profile_manager: Box::new(SkProfileManager::new(
                        Config::config_dir(),
                        package_manager,
                        crate::sk::packages::fetch_package_records,
                        SK_BLACKLIST,
                    )),
                    game_runner: Box::new(SkGameRunner),
                    blacklist: SK_BLACKLIST,
                }
            }

            GameSwitcher::HollowKnight => {
                // The SK package implementation remains the temporary package-operation
                // fallback until its HK equivalent is introduced.
                let package_manager = Rc::new(PackageManager::new(HK_BLACKLIST));

                Self {
                    package_loader: crate::hk::packages::fetch_package_records,
                    package_scanner: Box::new(SkPackageScanner),
                    package_manager,
                    profile_manager: Box::new(HkProfileManager::new(Config::config_dir())),
                    game_runner: Box::new(SkGameRunner),
                    blacklist: HK_BLACKLIST,
                }
            }
        }
    }
}
