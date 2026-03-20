mod cli;
mod handlers;
mod manager;
mod packages;
mod profile_manager;
mod tracker;
mod util;

use crate::{cli::command_dispatcher, util::context::Context};

fn main() {
    let mut context = Context::new().expect("Failed to create context");

    command_dispatcher::run(&mut context);
}
