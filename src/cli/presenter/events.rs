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

pub enum UpdateEvent {
    UpdateMod {
        name: String,
        old_version: String,
        new_version: String,
    },
    CleaningUpOldMod {
        name: String,
    },
    DownloadingMod {
        name: String,
    },
    InstallingMod {
        name: String,
    },
    UpdatingDependencies {
        dependencies: Vec<String>,
    },
    DependencyAlreadyNewestVersion {
        name: String,
    },
    UpdatingDependency {
        name: String,
        old_version: String,
        new_version: String,
    },
    InstallingDependency {
        name: String,
    },
}

impl Event for UpdateEvent {
    fn render(&self, _presenter: &mut Presenter) {
        match self {
            UpdateEvent::UpdateMod {
                name,
                old_version,
                new_version,
            } => {
                println!("==> Updating mod {name} from {old_version} to {new_version}...");
            }
            UpdateEvent::CleaningUpOldMod { name } => {
                println!("-> Cleaning up old mod {name}...");
            }
            UpdateEvent::DownloadingMod { name } => {
                println!("-> Downloading mod {name}...");
            }
            UpdateEvent::InstallingMod { name } => {
                println!("-> Installing mod {name}...")
            }
            UpdateEvent::UpdatingDependencies { dependencies } => {
                println!("==> Updating dependencies: {dependencies:?}");
            }
            UpdateEvent::DependencyAlreadyNewestVersion { name } => {
                println!("-> Dependency {name} is already the newest version");
                println!("-> Skipping...");
            }
            UpdateEvent::UpdatingDependency {
                name,
                old_version,
                new_version,
            } => {
                println!("-> Updating dependency {name} from {old_version} to {new_version}...");
            }
            UpdateEvent::InstallingDependency { name } => {
                println!("-> Dependency {name} not installed. installing...")
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
    ProfileDirectoryAlreadyExists {
        name: String,
        game: String,
        path: String,
    },
    InstallingBepInEx {
        name: String,
        game: String,
        path: String,
    },
    DeletingProfileDirectory {
        name: String,
        game: String,
        path: String,
    },
    ProfileDirectoryDoesNotExist {
        name: String,
        game: String,
        path: String,
    },
    SettingProfileAsDefault {
        name: String,
        game: String,
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
            ProfileManagerEvent::ProfileDirectoryAlreadyExists { name, game, path } => {
                println!(
                    "==> Profile directory for profile {} ({}) already exists at {}",
                    name, game, path
                );
            }
            ProfileManagerEvent::InstallingBepInEx { name, game, path } => {
                println!(
                    "==> Installing BepInEx for profile {} ({}) at {}",
                    name, game, path
                );
            }
            ProfileManagerEvent::DeletingProfileDirectory { name, game, path } => {
                println!(
                    "==> Deleting profile directory for profile {} ({}) at {}",
                    name, game, path
                );
            }
            ProfileManagerEvent::ProfileDirectoryDoesNotExist { name, game, path } => {
                println!(
                    "==> Profile directory for profile {} ({}) does not exist at {}",
                    name, game, path
                );
            }
            ProfileManagerEvent::SettingProfileAsDefault { name, game } => {
                println!("==> Setting profile {} ({}) as default", name, game);
            }
        }
    }
}

pub enum DisableEnableEvent {
    DisablingMod { name: String },
    ModAlreadyDisabled { name: String },
    EnablingMod { name: String },
    ModAlreadyEnabled { name: String },
}

impl Event for DisableEnableEvent {
    fn render(&self, _presenter: &mut Presenter) {
        match self {
            DisableEnableEvent::DisablingMod { name } => {
                println!("==> Disabling mod {name}...");
            }
            DisableEnableEvent::ModAlreadyDisabled { name } => {
                println!("-> Mod {name} is already disabled");
                println!("-> Skipping...");
            }
            DisableEnableEvent::EnablingMod { name } => {
                println!("==> Enabling mod {name}...");
            }
            DisableEnableEvent::ModAlreadyEnabled { name } => {
                println!("-> Mod {name} is already enabled");
                println!("-> Skipping...");
            }
        }
    }
}
