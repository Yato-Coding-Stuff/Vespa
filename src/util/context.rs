use thiserror::Error;

use crate::{
    packages::{SilkSongIndex, SilkSongIndexError},
    tracker::sk_package_tracker::SilkSongPackageTracker,
    util::config::Config,
};

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("config error: {0}")]
    ConfigError(#[from] Box<dyn std::error::Error>),
    #[error("index error: {0}")]
    IndexError(#[from] SilkSongIndexError),
}

pub struct Context {
    pub config: Config,
    pub tracker: SilkSongPackageTracker,
    pub index: SilkSongIndex,
    pub black_list: Vec<&'static str>,
}

impl Context {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;

        let tracker = SilkSongPackageTracker::new();

        let black_list = vec!["BepInEx-BepInExPack_Silksong"];

        let index = SilkSongIndex::new(&black_list)?;

        Ok(Self {
            config,
            tracker,
            index,
            black_list,
        })
    }
}
