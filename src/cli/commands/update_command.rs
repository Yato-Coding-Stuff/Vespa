use dialoguer::Confirm;
use semver::Version;
use std::path::PathBuf;

use crate::cli::commands::command_utils;
use crate::handlers::update_handler::{self, UpdateResult};
use crate::{
    cli::presenter::presenter::Presenter, packages::SilkSongFlattenedPackage,
    util::context::Context,
};

pub fn update(
    ctx: &mut Context,
    presenter: &mut Presenter,
    packages: Vec<String>,
    profile_path: &PathBuf,
) {
    match ctx.index.initialize(&ctx.black_list) {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return;
        }
    }

    if !packages.is_empty() {
        let packages = match command_utils::input_handling(ctx, packages) {
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
                "Do you want to update the following packages?\n - {}",
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
            match update_handler::run(ctx, package, presenter, false, profile_path) {
                Ok(UpdateResult::NotInstalled) => {
                    println!(
                        "==> {} is not installed",
                        package.package_full_name_with_version
                    );
                }
                Ok(UpdateResult::Updated) => {
                    println!("==> Updated {}", package.package_full_name_with_version);
                }
                Ok(UpdateResult::AlreadyNewestVersion) => {
                    println!(
                        "==> Already newest version of {}",
                        package.package_full_name_with_version
                    );
                }
                Err(e) => {
                    println!("==> {}", e);
                }
            }
        }
    } else {
        // Collect all installed packages
        let packages = ctx
            .tracker
            .get_all()
            .iter()
            .filter_map(|(_, installed)| {
                let latest = ctx
                    .index
                    .get_latest_package_by_package_name(&installed.package_full_name)?;

                let installed_version = installed
                    .version_number
                    .as_deref()
                    .unwrap_or("0.0.0")
                    .parse::<Version>()
                    .unwrap_or_else(|_| Version::new(0, 0, 0));

                let latest_version = latest.version_number.parse::<Version>().unwrap();

                if installed_version < latest_version {
                    Some(latest)
                } else {
                    None
                }
            })
            .collect::<Vec<SilkSongFlattenedPackage>>();

        if packages.is_empty() {
            println!("==> No packages to update");
            return;
        }

        if !Confirm::new()
            .with_prompt(format!(
                "Do you want to update the following installed packages?\n - {}",
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
            match update_handler::run(ctx, &package, presenter, true, profile_path) {
                Ok(UpdateResult::NotInstalled) => {
                    println!("==> {} is not installed", package.package_full_name);
                }
                Ok(UpdateResult::Updated) => {
                    println!("==> Updated {}", package.package_full_name);
                }
                Ok(UpdateResult::AlreadyNewestVersion) => {
                    println!(
                        "==> Already newest version of {}",
                        package.package_full_name
                    );
                }
                Err(e) => {
                    println!("==> {}", e);
                }
            }
        }
    }
}
