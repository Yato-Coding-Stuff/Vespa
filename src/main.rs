mod cli;
mod handlers;
mod manager;
mod packages;
mod profile_manager;
mod tracker;
mod util;

use crate::{cli::commands, util::context::Context};

fn main() {
    let mut context = Context::new().expect("Failed to create context");

    commands::run(&mut context);
}
