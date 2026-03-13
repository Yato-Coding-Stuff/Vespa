use crate::cli::presenter::{events::event::Event, presenter::Presenter};

pub enum ProfileManagerEvent {
    CreatingProfileDirectory {
        name: String,
        game: String,
        path: String,
    },
    InstallingBepInEx {
        name: String,
        game: String,
        path: String,
    },
}

impl Event for ProfileManagerEvent {
    fn render(&self, _presenter: &mut Presenter) {
        match self {
            ProfileManagerEvent::CreatingProfileDirectory { name, game, path } => {
                println!(
                    "==> Creating profile directory for profile {} ({}) at {}",
                    name, game, path
                );
            }
            ProfileManagerEvent::InstallingBepInEx { name, game, path } => {
                println!(
                    "==> Installing BepInEx for profile {} ({}) at {}",
                    name, game, path
                );
            }
        }
    }
}
