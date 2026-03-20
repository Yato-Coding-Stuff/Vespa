use dialoguer::Confirm;
use dialoguer::Input;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::path::PathBuf;

use crate::{
    cli::presenter::{events::install_event::InstallEvent, presenter::Presenter},
    handlers::install_handler::{self, InstallResult},
    packages::SilkSongFlattenedPackage,
    util::context::Context,
};

fn input_handling(
    ctx: &mut Context,
    packages: Vec<String>,
) -> Vec<Option<SilkSongFlattenedPackage>> {
    let mut optional_packages: Vec<Option<SilkSongFlattenedPackage>> = Vec::new();

    for package in packages {
        if let Some(package) = ctx.index.get_package_by_full_name(&package) {
            println!(
                "==> Exact match found: {}",
                package.package_full_name_with_version
            );
            optional_packages.push(Some(package));
        } else {
            println!("==> No exact match found for: {}", package);
            let matcher = SkimMatcherV2::default();

            let mut matches = ctx
                .index
                .latest_full_name_by_package_name
                .iter()
                .filter_map(|(name, pkg)| {
                    matcher
                        .fuzzy_match(name, &package)
                        .map(|score| (name, pkg, score))
                })
                .collect::<Vec<_>>();

            matches.sort_by(|a, b| b.2.cmp(&a.2));

            if matches.is_empty() {
                optional_packages.push(None);
                continue;
            }

            let matches: Vec<_> = matches.iter().take(3).cloned().collect();

            println!("Multiple matches found:");
            for (i, (name, _, score)) in matches.iter().enumerate() {
                println!("{}) {} (score {})", i + 1, name, score);
            }
            println!("{}) None", matches.len() + 1);

            let selection: usize = Input::new()
                .with_prompt("==> Select a package by number")
                .interact()
                .unwrap();

            if selection == matches.len() + 1 {
                optional_packages.push(None);
                continue;
            }

            let selected_package = ctx
                .index
                .get_package_by_full_name(&matches[selection - 1].1.clone())
                .unwrap();

            optional_packages.push(Some(selected_package));
        }
    }

    optional_packages
}

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

    let packages = input_handling(ctx, packages);

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
