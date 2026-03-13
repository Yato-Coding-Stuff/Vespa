use std::path::PathBuf;

use clap::Parser;
use dialoguer::{Confirm, Input};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use crate::{
    cli::{
        args::{Arg, SubArgs},
        presenter::presenter::Presenter,
    }, handlers::install_handler::{self, InstallResult}, packages::SilkSongFlattenedPackage, profile_manager::sk_profile_manager::SilkSongProfileManager, util::{
        config::{Config, GameSwitcher},
        context::Context,
    }
};

pub fn run(ctx: &mut Context) {
    let args = Arg::parse();
    let mut presenter = Presenter::new();

    let game = match args.game {
        Some(game) => game.into(),
        None => ctx.config.game_switcher.clone(),
    };

    let profile = match args.profile {
        Some(profile) => profile,
        None => match game {
            GameSwitcher::HollowKnight => ctx.config.default_hollow_knight_profile.clone(),
            GameSwitcher::SilkSong => ctx.config.default_silk_song_profile.clone(),
        },
    };

    // TODO
    // own function
    let profile_manager = SilkSongProfileManager::new(Config::config_dir());
    let profile_path = profile_manager
        .ensure_profile(ctx, &mut presenter, &game, &profile)
        .map_err(|err| {
            println!("{}", err);
            std::process::exit(1);
        })
        .unwrap();

    match args.sub {
        SubArgs::Install { packages } => {
            // TODO
            // profile path should not be last argument
            // carries on through the entire install-chain
            install(ctx, &mut presenter, packages, &profile_path);
        }
        _ => {
            todo!()
        }
    }
}

fn input_handling(
    ctx: &mut Context,
    packages: Vec<String>,
) -> Vec<Option<SilkSongFlattenedPackage>> {
    let mut optional_packages: Vec<Option<SilkSongFlattenedPackage>> = Vec::new();

    for package in packages {
        if let Some(package) = ctx.index.get_package_by_full_name(&package) {
            println!(
                "==> Exact match found: {}",
                package.package_name_with_version
            );
            optional_packages.push(Some(package));
        } else {
            println!("==> No exact match found for: {}", package);
            let matcher = SkimMatcherV2::default();

            let mut matches = ctx
                .index
                .full_name_by_package_name
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

fn install(
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
                .map(|p| p.package_name_with_version.clone())
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
                println!(
                    "==> Installed {}",
                    package.package_name_with_version.clone()
                );
            }
            Ok(InstallResult::AlreadyInstalled) => {
                let reinstall = Confirm::new()
                    .with_prompt("Package already installed. Reinstall?")
                    .interact()
                    .unwrap();

                if !reinstall {
                    continue;
                }

                match install_handler::run(ctx, package, true, &mut presenter, profile_path) {
                    Ok(_) => println!("==> Reinstalled {}", package.package_name_with_version),
                    Err(e) => println!(
                        "==> Failed to reinstall {}: {e}",
                        package.package_name_with_version
                    ),
                }
            }

            Err(e) => {
                println!(
                    "==> Failed to install {}: {e}",
                    package.package_name_with_version
                );
            }
        }
    }
}
