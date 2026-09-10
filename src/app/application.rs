use crate::{
    app::game_services::GameServices,
    base::{
        packages::IndexError,
        util::{config::GameSwitcher, context::Context},
    },
};

pub struct App {
    pub active_game: GameSwitcher,
    pub context: Context,
    pub game_services: GameServices,
}

impl App {
    pub fn new(active_game: GameSwitcher, context: Context, game_services: GameServices) -> Self {
        Self {
            active_game,
            context,
            game_services,
        }
    }

    pub fn initialize_tracker(
        &mut self,
        profile_path: &std::path::Path,
    ) -> Result<(), std::io::Error> {
        let packages = self
            .game_services
            .package_scanner
            .scan_plugins(profile_path)?;
        self.context.tracker.replace(packages);
        Ok(())
    }

    pub fn initialize_index(&mut self) -> Result<(), IndexError> {
        let packages = match (self.game_services.package_loader)(self.game_services.blacklist) {
            Ok(packages) => {
                if let Some(parent) = self.context.config.index_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(
                    &self.context.config.index_path,
                    serde_json::to_vec_pretty(&packages).unwrap_or_default(),
                );
                packages
            }
            Err(fetch_error) => {
                let cached = std::fs::read(&self.context.config.index_path)
                    .ok()
                    .and_then(|bytes| {
                        serde_json::from_slice::<Vec<crate::base::packages::PackageRecord>>(&bytes)
                            .ok()
                    });
                cached.ok_or(fetch_error)?
            }
        };
        self.context.index.replace(packages);
        Ok(())
    }
}
