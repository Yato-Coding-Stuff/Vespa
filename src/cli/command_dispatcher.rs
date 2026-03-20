use clap::Parser;

use crate::{
    cli::{
        args::{Arg, SubArgs},
        commands::install_command,
        presenter::presenter::Presenter,
    },
    profile_manager::sk_profile_manager::SilkSongProfileManager,
    util::{config::Config, context::Context},
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
        None => "default".to_string(),
    };


    let profile_manager = SilkSongProfileManager::new(Config::config_dir());
    let profile_path = profile_manager
        .ensure_profile(ctx, &mut presenter, &game, &profile)
        .map_err(|err| {
            println!("{}", err);
            std::process::exit(1);
        })
        .unwrap();

    ctx.tracker.scan_plugins(&profile_path);

    match args.sub {
        SubArgs::Install { packages } => {
            install_command::install(ctx, &mut presenter, packages, &profile_path);
        }
        _ => {
            todo!()
        }
    }
}
