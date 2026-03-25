use crate::cli::presenter::presenter::Presenter;

pub trait Event {
    fn render(&self, presenter: &mut Presenter);
}

pub enum InstallEvent {
    InstallingDependencies {
        dependencies: Vec<String>,
    },
    InstallingDependency {
        name: String,
    },
    UpdatingDependency {
        name: String,
        old_version: String,
        new_version: String,
    },
    DependencyAlreadyInstalled,
    ModAlreadyInstalled,
    DownloadingMod {
        name: String,
    },
    StartingDownload {
        total: u64,
    },
    DownloadProgress {
        downloaded: u64,
    },
    FinishedDownloadingMod {
        name: String,
    },
    InstallingMod {
        name: String,
    },
    Finished,
}

impl Event for InstallEvent {
    fn render(&self, presenter: &mut Presenter) {
        match self {
            InstallEvent::InstallingDependencies { dependencies } => {
                println!("==> Installing dependencies: {dependencies:?}");
            }
            InstallEvent::InstallingDependency { name } => {
                println!("-> Installing dependency {name}...");
            }
            InstallEvent::UpdatingDependency {
                name,
                old_version,
                new_version,
            } => {
                println!("-> Updating dependency {name} from {old_version} to {new_version}...");
            }
            InstallEvent::DependencyAlreadyInstalled => {
                println!("-> Dependency already installed!");
            }
            InstallEvent::ModAlreadyInstalled => {
                println!("-> Mod already installed!")
            }
            InstallEvent::DownloadingMod { name } => {
                println!("-> Downloading mod {name}...");
            }
            InstallEvent::StartingDownload { total } => {
                presenter.start_download(total.clone());
            }
            InstallEvent::DownloadProgress { downloaded } => {
                presenter.update_download(downloaded.clone());
            }
            InstallEvent::FinishedDownloadingMod { name } => {
                presenter.finish_download();
                println!("-> Finished downloading mod {name}");
            }
            InstallEvent::InstallingMod { name } => {
                println!("-> Installing mod {name}...")
            }
            InstallEvent::Finished => println!("==> Finished"),
        }
    }
}

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
                    "==> Creating profile directory for profile {} ({}) at wouldnt you like to know where?",
                    name, game
                );
            }
            ProfileManagerEvent::InstallingBepInEx { name, game, path } => {
                println!(
                    "==> Installing BepInEx for profile {} ({}) at wouldnt you like to know where?",
                    name, game
                );
            }
        }
    }
}
