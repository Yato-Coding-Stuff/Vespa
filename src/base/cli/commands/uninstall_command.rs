use std::path::PathBuf;

use dialoguer::Confirm;

use crate::{
    app::application::App,
    base::handlers::uninstall_handler::{self, UninstallResult},
    base::{
        cli::{commands::command_utils, presenter::presenter::Presenter},
        packages::InstalledPackageRecord,
    },
};

pub fn uninstall(
    app: &mut App,
    presenter: &mut Presenter,
    packages: Vec<String>,
    force: bool,
    profile_path: &PathBuf,
) {
    let mut presenter = |event| presenter.display(&event);

    match app.initialize_index() {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return;
        }
    }

    let packages = match command_utils::get_input_in_tracker(&mut app.context, packages) {
        Ok(packages) => packages,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let packages = packages
        .iter()
        .filter_map(|p| p.as_ref())
        .collect::<Vec<&InstalledPackageRecord>>();

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to uninstall the following packages?\n - {}",
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
        match uninstall_handler::run(app, package, force, &mut presenter, profile_path) {
            Ok(UninstallResult::Uninstalled) => {
                println!("==> Uninstalled {}", package.identifier);
            }
            Ok(UninstallResult::NotInstalled) => {
                println!("==> {} is not installed", package.identifier);
            }
            Ok(UninstallResult::PackageStillRequired { packages }) => {
                println!(
                    "==> {} is required by the following packages: {:?}",
                    package.identifier,
                    packages.join(", ")
                );
                println!("==> Please re-run with --force if you want to uninstall anyway");
            }
            Err(e) => {
                println!("==> Failed to uninstall {}: {e}", package.identifier);
            }
        }
    }
}
