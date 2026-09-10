use dialoguer::Confirm;
use std::path::PathBuf;

use crate::{
    app::application::App,
    base::handlers::update_handler::{self, UpdateResult},
    base::{
        cli::{commands::command_utils, presenter::presenter::Presenter},
        packages::PackageRecord,
    },
};

pub fn update(
    app: &mut App,
    presenter: &mut Presenter,
    packages: Vec<String>,
    profile_path: &PathBuf,
) -> Result<(), String> {
    match app.initialize_index() {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return Err(e.to_string());
        }
    }

    if !packages.is_empty() {
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
                "Do you want to update the following packages?\n - {}",
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
            match update_handler::run(app, package, presenter, false, profile_path) {
                Ok(UpdateResult::NotInstalled) => {
                    println!("==> {} is not installed", package.identifier);
                }
                Ok(UpdateResult::Updated) => {
                    println!("==> Updated {}", package.identifier);
                }
                Ok(UpdateResult::AlreadyNewestVersion) => {
                    println!("==> Already newest version of {}", package.identifier);
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors.join("; "));
        }
    } else {
        let packages = update_handler::packages_to_update(&app.context);

        if packages.is_empty() {
            println!("==> No packages to update");
            return Ok(());
        }

        if !Confirm::new()
            .with_prompt(format!(
                "Do you want to update the following installed packages?\n - {}",
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
            match update_handler::run(app, &package, presenter, true, profile_path) {
                Ok(UpdateResult::NotInstalled) => {
                    println!("==> {} is not installed", package.name);
                }
                Ok(UpdateResult::Updated) => {
                    println!("==> Updated {}", package.name);
                }
                Ok(UpdateResult::AlreadyNewestVersion) => {
                    println!("==> Already newest version of {}", package.name);
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors.join("; "));
        }
    }
    Ok(())
}
