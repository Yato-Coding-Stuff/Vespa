use crate::{installer::sk_package_tracker::SilkSongPackageTracker, util::config::Config};

pub struct Context {
    pub config: Config,
    pub tracker: SilkSongPackageTracker,
}

impl Context {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;

        let tracker =
            SilkSongPackageTracker::load(config.index_path.to_str().unwrap()).unwrap_or_default();

        Ok(Self { config, tracker })
    }
}
