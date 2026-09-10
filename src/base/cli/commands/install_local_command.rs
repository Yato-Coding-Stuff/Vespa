use dialoguer::Confirm;
use std::path::PathBuf;

use crate::app::application::App;

pub fn install(
    app: &mut App,
    package_paths: Vec<PathBuf>,
    profile_path: &PathBuf,
) -> Result<(), String> {
    match app.initialize_index() {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return Err(e.to_string());
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
        .map_err(|error| error.to_string())?
    {
        println!("==> Aborted installation");
        return Ok(());
    }

    for package in package_paths {
        match app.game_services.package_manager.install_local_package(
            &mut app.context,
            &package,
            profile_path,
        ) {
            Ok(()) => {
                println!("==> Installed {}", package.to_string_lossy());
            }
            Err(e) => {
                println!("==> {}", e);
                return Err(e.to_string());
            }
        }
    }
    Ok(())
}
