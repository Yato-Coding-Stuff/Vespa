use clap::Parser;
use std::process::exit;

use crate::{
    cli::{
        args::{Arg, ProfileArgs, SubArgs},
        commands::{
            command_utils, disable_enable_command, install_command, install_local_command,
            list_command, profile_command, show_command, uninstall_command, update_command,
        },
        presenter::presenter::Presenter,
    },
    util::context::Context,
};

pub fn run(ctx: &mut Context) {
    let args = Arg::parse();
    let mut presenter = Presenter::new();

    let game = match args.game {
        Some(game) => game.into(),
        None => ctx.config.game_switcher.clone(),
    };

    match args.sub {
        SubArgs::Install { packages } => {
            let profile_path = require_profile_path_or_exit(ctx, &game);
            ctx.tracker.scan_plugins(&profile_path);
            install_command::install(ctx, &mut presenter, packages, &profile_path);
        }
        SubArgs::InstallLocal { package_paths } => {
            let profile_path = require_profile_path_or_exit(ctx, &game);
            ctx.tracker.scan_plugins(&profile_path);
            install_local_command::install(ctx, package_paths, &profile_path);
        }
        SubArgs::Uninstall { packages, force } => {
            let profile_path = require_profile_path_or_exit(ctx, &game);
            ctx.tracker.scan_plugins(&profile_path);
            uninstall_command::uninstall(ctx, &mut presenter, packages, force, &profile_path);
        }
        SubArgs::Disable { packages } => {
            let profile_path = require_profile_path_or_exit(ctx, &game);
            ctx.tracker.scan_plugins(&profile_path);
            disable_enable_command::disable(ctx, &mut presenter, packages);
        }
        SubArgs::Enable { packages } => {
            let profile_path = require_profile_path_or_exit(ctx, &game);
            ctx.tracker.scan_plugins(&profile_path);
            disable_enable_command::enable(ctx, &mut presenter, packages);
        }
        SubArgs::List {
            packages,
            available,
            all_versions,
        } => {
            if !available {
                let profile_path = require_profile_path_or_exit(ctx, &game);
                ctx.tracker.scan_plugins(&profile_path);
            }
            list_command::list(ctx, packages, available, all_versions);
        }
        SubArgs::Show { package } => {
            show_command::show(ctx, package);
        }
        SubArgs::Update { packages } => {
            let profile_path = require_profile_path_or_exit(ctx, &game);
            ctx.tracker.scan_plugins(&profile_path);
            update_command::update(ctx, &mut presenter, packages, &profile_path);
        }
        SubArgs::Profile { args } => match args {
            ProfileArgs::List => {
                profile_command::list(ctx, &game);
            }
            ProfileArgs::Create { profile } => {
                profile_command::create(ctx, &mut presenter, &game, profile);
            }
            ProfileArgs::Delete { profile } => {
                profile_command::delete(ctx, &mut presenter, &game, profile);
            }
            ProfileArgs::SetDefault { profile } => {
                profile_command::set_default(ctx, &mut presenter, &game, profile);
            }
        },
    }
}

fn require_profile_path_or_exit(
    ctx: &Context,
    game: &crate::util::config::GameSwitcher,
) -> std::path::PathBuf {
    match command_utils::require_profile_path(ctx, game) {
        Ok(profile_path) => profile_path,
        Err(error) => {
            eprintln!("{error}");
            exit(1);
        }
    }
}
