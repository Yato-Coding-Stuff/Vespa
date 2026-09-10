use std::path::Path;

use crate::{
    app::application::App,
    base::runtime::{Runtime, primer::Primer},
};

pub fn prime(app: &App, profile_path: &Path) {
    let desired = match (Primer {}).files_from_profile(profile_path) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("Failed to inspect profile: {error}");
            return;
        }
    };
    let transaction_path = Runtime::transaction_path(&app.active_game);
    let runtime = Runtime::new(
        app.context.config.get_game_path(&app.active_game).clone(),
        transaction_path,
    );
    match runtime.prime(
        app.active_game.clone(),
        profile_path.display().to_string(),
        desired,
    ) {
        Ok(()) => println!(
            "==> Primed {} with profile {}",
            app.active_game,
            profile_path.display()
        ),
        Err(error) => eprintln!("Failed to prime profile: {error}"),
    }
}
