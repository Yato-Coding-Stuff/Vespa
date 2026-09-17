use std::{path::Path, rc::Rc};

use crate::{
    base::{
        manager::package_manager::PackageManager,
        packages::{PackageLoader, package_scanner::PackageScanner},
        profile_manager::profile_manager::ProfileManager,
        runner::game_runner::GameRunner,
        util::config::{Config, GameSwitcher},
    },
    hk::profile_initializer::profile_initializer::HkProfileInitializer,
    sk::{
        profile_initializer::profile_initializer::SkProfileInitializer,
        runner::game_runner::SkGameRunner,
    },
};

const SK_BLACKLIST: &[&str] = &["BepInEx-BepInExPack_Silksong"];

const HK_BLACKLIST: &[&str] = &[];

pub struct GameServices {
    pub package_loader: PackageLoader,
    pub package_scanner: Box<PackageScanner>,
    pub package_manager: Rc<PackageManager>,
    pub profile_manager: Box<ProfileManager>,
    pub game_runner: Box<dyn GameRunner>,
    pub blacklist: &'static [&'static str],
}

impl GameServices {
    pub fn for_game(game: &GameSwitcher) -> Self {
        match game {
            GameSwitcher::SilkSong => {
                let package_loader = crate::sk::packages::fetch_package_records;
                let package_root = crate::sk::packages::package_layout::package_root;
                let blacklist = SK_BLACKLIST;

                let package_manager = Rc::new(PackageManager::new(blacklist, package_root));

                let package_scanner = Box::new(PackageScanner::new(package_root));

                let profile_initializer = Box::new(SkProfileInitializer {
                    package_manager: package_manager.clone(),
                    package_loader,
                    blacklist,
                });

                let profile_manager = Box::new(ProfileManager::new(
                    Config::config_dir(),
                    profile_initializer,
                ));

                Self {
                    package_loader,
                    package_scanner,
                    package_manager,
                    profile_manager,
                    game_runner: Box::new(SkGameRunner),
                    blacklist,
                }
            }

            GameSwitcher::HollowKnight => {
                let package_loader = crate::hk::packages::fetch_package_records;
                let package_root = Path::to_path_buf;
                let blacklist = HK_BLACKLIST;

                let package_manager = Rc::new(PackageManager::new(blacklist, package_root));

                let package_scanner = Box::new(PackageScanner::new(package_root));

                let profile_initializer = Box::new(HkProfileInitializer {});

                let profile_manager = Box::new(ProfileManager::new(
                    Config::config_dir(),
                    profile_initializer,
                ));

                Self {
                    package_loader,
                    package_scanner,
                    package_manager,
                    profile_manager,
                    game_runner: Box::new(SkGameRunner),
                    blacklist,
                }
            }
        }
    }
}
