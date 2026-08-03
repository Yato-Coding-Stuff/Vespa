use crate::{
    app::application::App,
    base::cli::{
        args::{Arg, ProfileArgs, SubArgs},
        commands::{
            command_utils, disable_enable_command, install_command, install_local_command,
            list_command, profile_command, run_command, show_command, uninstall_command,
            update_command,
        },
        presenter::presenter::Presenter,
    },
};
use std::process::exit;

pub fn run(app: &mut App, args: Arg) {
    let mut presenter = Presenter::new();

    match args.sub {
        SubArgs::Install { packages } => {
            let profile_path = require_profile_path_or_exit(app);
            app.initialize_tracker(&profile_path);
            install_command::install(app, &mut presenter, packages, &profile_path);
        }
        SubArgs::InstallLocal { package_paths } => {
            let profile_path = require_profile_path_or_exit(app);
            app.initialize_tracker(&profile_path);
            install_local_command::install(app, package_paths, &profile_path);
        }
        SubArgs::Uninstall { packages, force } => {
            let profile_path = require_profile_path_or_exit(app);
            app.initialize_tracker(&profile_path);
            uninstall_command::uninstall(app, &mut presenter, packages, force, &profile_path);
        }
        SubArgs::Disable { packages } => {
            let profile_path = require_profile_path_or_exit(app);
            app.initialize_tracker(&profile_path);
            disable_enable_command::disable(app, &mut presenter, packages);
        }
        SubArgs::Enable { packages } => {
            let profile_path = require_profile_path_or_exit(app);
            app.initialize_tracker(&profile_path);
            disable_enable_command::enable(app, &mut presenter, packages);
        }
        SubArgs::List {
            packages,
            available,
            all_versions,
        } => {
            if !available {
                let profile_path = require_profile_path_or_exit(app);
                app.initialize_tracker(&profile_path);
            }
            list_command::list(app, packages, available, all_versions);
        }
        SubArgs::Show { package } => {
            show_command::show(app, package);
        }
        SubArgs::Run {} => {
            let profile_path = require_profile_path_or_exit(app);
            run_command::run(app, &profile_path);
        }
        SubArgs::Update { packages } => {
            let profile_path = require_profile_path_or_exit(app);
            app.initialize_tracker(&profile_path);
            update_command::update(app, &mut presenter, packages, &profile_path);
        }
        SubArgs::Profile { args } => match args {
            ProfileArgs::List => {
                profile_command::list(app);
            }
            ProfileArgs::Create { profile } => {
                profile_command::create(app, &mut presenter,profile);
            }
            ProfileArgs::Delete { profile } => {
                profile_command::delete(app, &mut presenter, profile);
            }
            ProfileArgs::SetDefault { profile } => {
                profile_command::set_default(app, &mut presenter, profile);
            }
        },
    }
}

fn require_profile_path_or_exit(app: &App) -> std::path::PathBuf {
    match command_utils::require_profile_path(&app.context, &app.active_game) {
        Ok(profile_path) => profile_path,
        Err(error) => {
            eprintln!("{error}");
            exit(1);
        }
    }
}
