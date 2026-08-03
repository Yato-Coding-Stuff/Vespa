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

    pub fn initialize_tracker(&mut self, profile_path: &std::path::Path) {
        self.context.tracker.replace(
            self.game_services
                .package_scanner
                .scan_plugins(profile_path),
        );
    }

    pub fn initialize_index(&mut self) -> Result<(), IndexError> {
        let packages = (self.game_services.package_loader)(self.game_services.blacklist)?;
        self.context.index.replace(packages);
        Ok(())
    }
}
