use crate::base::manager::package_installer::PackageInstaller;
use dialoguer::Confirm;
use std::path::PathBuf;

use crate::app::application::App;

pub fn install(app: &mut App, package_paths: Vec<PathBuf>, profile_path: &PathBuf) {
    match app.initialize_index() {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return;
        }
    }

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to install the following local packages?\n - {}",
            package_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join("\n - ")
        ))
        .report(false)
        .interact()
        .unwrap()
    {
        println!("==> Aborted installation");
        return;
    }

    for package in package_paths {
        let package_installer = PackageInstaller::new();
        match package_installer.install_local_package(&package, profile_path) {
            Ok(()) => {
                println!("==> Installed {}", package.to_string_lossy());
            }
            Err(e) => {
                println!("==> {}", e);
                return;
            }
        }
    }
}
