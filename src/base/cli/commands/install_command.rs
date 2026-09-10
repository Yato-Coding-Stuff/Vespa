use dialoguer::Confirm;
use std::path::PathBuf;

use crate::{
    app::application::App,
    base::handlers::install_handler::{self, InstallResult},
    base::{
        cli::{
            commands::command_utils,
            presenter::{events::InstallEvent, presenter::Presenter},
        },
        packages::PackageRecord,
    },
};

fn handle_user_choice_installation(
    app: &mut App,
    package: &PackageRecord,
    presenter: &mut impl FnMut(InstallEvent),
    profile_path: &PathBuf,
    action_name: &str,
) -> Result<(), String> {
    let confirm = Confirm::new()
        .with_prompt(format!(
            "Do you want to {} {}?",
            action_name, package.identifier
        ))
        .report(false)
        .interact()
        .map_err(|error| error.to_string())?;

    if confirm {
        match install_handler::run(app, package, true, presenter, profile_path) {
            Ok(_) => println!("==> {} {}", action_name, package.identifier),
            Err(e) => {
                return Err(format!(
                    "Failed to {} {}: {e}",
                    action_name, package.identifier
                ));
            }
        }
    }
    Ok(())
}

pub fn install(
    app: &mut App,
    presenter: &mut Presenter,
    packages: Vec<String>,
    profile_path: &PathBuf,
) -> Result<(), String> {
    let mut presenter = |event| presenter.display(&event);

    match app.initialize_index() {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return Err(e.to_string());
        }
    }

    let packages = match command_utils::get_input_in_index(&mut app.context, packages) {
        Ok(packages) => packages,
        Err(e) => {
            println!("{}", e);
            return Err(e);
        }
    };

    let packages = packages
        .iter()
        .filter_map(|p| p.as_ref())
        .collect::<Vec<&PackageRecord>>();

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to install the following packages?\n - {}",
            packages
                .iter()
                .map(|p| p.identifier.clone())
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

    let mut errors = Vec::new();
    for package in packages {
        match install_handler::run(app, package, false, &mut presenter, profile_path) {
            Ok(InstallResult::Installed) => {
                println!("==> Installed {}", package.identifier);
            }
            Ok(InstallResult::AlreadyInstalled) => {
                println!("==> {} is already installed", package.identifier);
                if let Err(error) = handle_user_choice_installation(
                    app,
                    package,
                    &mut presenter,
                    profile_path,
                    "reinstall",
                ) {
                    errors.push(error);
                }
            }
            Ok(InstallResult::NewerVersionInstalled) => {
                println!("==> {} has an older version installed", package.identifier);
                if let Err(error) = handle_user_choice_installation(
                    app,
                    package,
                    &mut presenter,
                    profile_path,
                    "upgrade",
                ) {
                    errors.push(error);
                }
            }
            Ok(InstallResult::OlderVersionInstalled) => {
                println!("==> {} has a newer version installed", package.identifier);
                if let Err(error) = handle_user_choice_installation(
                    app,
                    package,
                    &mut presenter,
                    profile_path,
                    "downgrade",
                ) {
                    errors.push(error);
                }
            }
            Err(e) => {
                errors.push(format!("Failed to install {}: {e}", package.identifier));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
