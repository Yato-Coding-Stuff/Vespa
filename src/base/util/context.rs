use thiserror::Error;

use crate::base::{
    packages::Index,
    tracker::package_tracker::PackageTracker,
    util::config::{Config, ConfigError},
};

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("config error: {0}")]
    ConfigError(#[from] ConfigError),
}

pub struct Context {
    pub config: Config,
    pub tracker: PackageTracker,
    pub index: Index,
}

impl Context {
    pub fn new() -> Result<Self, ContextError> {
        let config = Config::load()?;

        let tracker = PackageTracker::new();
        let index = Index::new();

        Ok(Self {
            config,
            tracker,
            index,
        })
    }
}
