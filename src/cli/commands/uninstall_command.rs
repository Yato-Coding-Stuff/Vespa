use std::path::PathBuf;

use dialoguer::Confirm;

use crate::{
    cli::{commands::command_utils, presenter::presenter::Presenter}, handlers::uninstall_handler::{self, UninstallResult}, packages::SilkSongFlattenedPackage, util::context::Context
};

pub fn uninstall(
    ctx: &mut Context,
    presenter: &mut Presenter,
    packages: Vec<String>,
    force: bool,
    profile_path: &PathBuf,
) {
    let mut presenter = |event| presenter.display(&event);

    let packages = command_utils::input_handling(ctx, packages);

    let packages = packages
        .iter()
        .filter_map(|p| p.as_ref())
        .collect::<Vec<&SilkSongFlattenedPackage>>();

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to uninstall the following packages?\n - {}",
            packages
                .iter()
                .map(|p| p.package_full_name_with_version.clone())
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
        match uninstall_handler::run(ctx, package, force, &mut presenter, profile_path) {
            Ok(UninstallResult::Uninstalled) => {
                println!("==> Uninstalled {}", package.package_full_name_with_version);
            }
            Ok(UninstallResult::NotInstalled) => {
                println!(
                    "==> {} is not installed",
                    package.package_full_name_with_version
                );
            }
            Ok(UninstallResult::PackageStillRequired) => {
                println!(
                    "==> {} is required by another package",
                    package.package_full_name_with_version
                );
                println!("==> Please re-run with --force if you want to uninstall anyway");
            }
            Err(e) => {
                println!(
                    "==> Failed to uninstall {}: {e}",
                    package.package_full_name_with_version
                );
            }
        }
    }
}
