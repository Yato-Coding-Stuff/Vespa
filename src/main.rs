mod app;
mod base;
mod hk;
mod sk;

use crate::{
    app::{application::App, game_services::GameServices},
    base::{
        cli::{args::Arg, command_dispatcher},
        util::context::Context,
    },
};
use clap::Parser;

fn main() {
    let args = Arg::parse();
    let context = Context::new().expect("Failed to create context");

    let active_game = match args.game {
        Some(game) => game.into(),
        None => context.config.game_switcher.clone(),
    };

    let game_services = GameServices::for_game(&active_game);
    let mut app = App::new(active_game, context, game_services);

    command_dispatcher::run(&mut app, args);
}
