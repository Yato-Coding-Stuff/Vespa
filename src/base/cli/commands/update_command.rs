use dialoguer::Confirm;
use semver::Version;
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
) {
    match app.initialize_index() {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return;
        }
    }

    if !packages.is_empty() {
        let packages = match command_utils::get_input_in_index(&mut app.context, packages) {
            Ok(packages) => packages,
            Err(e) => {
                println!("{}", e);
                return;
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
            .unwrap()
        {
            println!("==> Aborted installation");
            return;
        }

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
                    println!("==> {}", e);
                }
            }
        }
    } else {
        // Collect all installed packages
        let packages = app
            .context
            .tracker
            .get_all()
            .iter()
            .filter_map(|(_, installed)| {
                let latest = app
                    .context
                    .index
                    .get_latest_package_by_package_name(&installed.name)?;

                let installed_version = installed
                    .version
                    .as_deref()
                    .unwrap_or("0.0.0")
                    .parse::<Version>()
                    .unwrap_or_else(|_| Version::new(0, 0, 0));

                let latest_version = latest.version.parse::<Version>().unwrap();

                if installed_version < latest_version {
                    Some(latest)
                } else {
                    None
                }
            })
            .collect::<Vec<PackageRecord>>();

        if packages.is_empty() {
            println!("==> No packages to update");
            return;
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
            .unwrap()
        {
            println!("==> Aborted installation");
            return;
        }

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
                    println!("==> {}", e);
                }
            }
        }
    }
}
