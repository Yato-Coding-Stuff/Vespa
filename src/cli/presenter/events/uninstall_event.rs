use crate::cli::presenter::{events::event::Event, presenter::Presenter};

pub enum UninstallEvent {
    UninstallingMod { name: String },
    UninstallingDependencies { dependencies: Vec<String> },
    UninstallingDependency { name: String },
    DependencyAlreadyUninstalled { name: String },
    Finished,
}

impl Event for UninstallEvent {
    fn render(&self, _presenter: &mut Presenter) {
        match self {
            UninstallEvent::UninstallingMod { name } => {
                println!("==> Uninstalling mod {name}...");
            }
            UninstallEvent::UninstallingDependencies { dependencies } => {
                println!("==> Uninstalling dependencies: {dependencies:?}");
            }
            UninstallEvent::UninstallingDependency { name } => {
                println!("-> Uninstalling dependency {name}...");
            }
            UninstallEvent::DependencyAlreadyUninstalled { name } => {
                println!("-> Dependency {name} is already uninstalled");
                println!("-> Skipping...");
            }
            UninstallEvent::Finished => {
                println!("==> Finished");
            }
        }
    }
}
