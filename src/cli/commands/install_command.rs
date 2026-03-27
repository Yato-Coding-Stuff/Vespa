use dialoguer::Confirm;
use std::path::PathBuf;

use crate::cli::commands::command_utils;
use crate::{
    cli::presenter::{events::InstallEvent, presenter::Presenter},
    handlers::install_handler::{self, InstallResult},
    packages::SilkSongFlattenedPackage,
    util::context::Context,
};

fn handle_user_choice_installation(
    ctx: &mut Context,
    package: &SilkSongFlattenedPackage,
    presenter: &mut impl FnMut(InstallEvent),
    profile_path: &PathBuf,
    action_name: &str,
) {
    let confirm = Confirm::new()
        .with_prompt(format!(
            "Do you want to {} {}?",
            action_name, package.package_full_name_with_version
        ))
        .report(false)
        .interact()
        .unwrap();

    if confirm {
        match install_handler::run(ctx, package, true, presenter, profile_path) {
            Ok(_) => println!(
                "==> {} {}",
                action_name, package.package_full_name_with_version
            ),
            Err(e) => println!(
                "==> Failed to {} {}: {e}",
                action_name, package.package_full_name_with_version
            ),
        }
    }
}

pub fn install(
    ctx: &mut Context,
    presenter: &mut Presenter,
    packages: Vec<String>,
    profile_path: &PathBuf,
) {
    let mut presenter = |event| presenter.display(&event);

    match ctx.index.initialize(&ctx.black_list) {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return;
        }
    }

    let packages = match command_utils::install_input_handling(ctx, packages) {
        Ok(packages) => packages,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let packages = packages
        .iter()
        .filter_map(|p| p.as_ref())
        .collect::<Vec<&SilkSongFlattenedPackage>>();

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to install the following packages?\n - {}",
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
        match install_handler::run(ctx, package, false, &mut presenter, profile_path) {
            Ok(InstallResult::Installed) => {
                println!("==> Installed {}", package.package_full_name_with_version);
            }
            Ok(InstallResult::AlreadyInstalled) => {
                println!(
                    "==> {} is already installed",
                    package.package_full_name_with_version
                );
                handle_user_choice_installation(
                    ctx,
                    package,
                    &mut presenter,
                    profile_path,
                    "reinstall",
                );
            }
            Ok(InstallResult::NewerVersionInstalled) => {
                println!(
                    "==> {} has an older version installed",
                    package.package_full_name_with_version
                );
                handle_user_choice_installation(
                    ctx,
                    package,
                    &mut presenter,
                    profile_path,
                    "upgrade",
                );
            }
            Ok(InstallResult::OlderVersionInstalled) => {
                println!(
                    "==> {} has a newer version installed",
                    package.package_full_name_with_version
                );
                handle_user_choice_installation(
                    ctx,
                    package,
                    &mut presenter,
                    profile_path,
                    "downgrade",
                );
            }
            Err(e) => {
                println!(
                    "==> Failed to install {}: {e}",
                    package.package_full_name_with_version
                );
            }
        }
    }
}
