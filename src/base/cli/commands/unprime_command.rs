use crate::{app::application::App, base::runtime::Runtime};

pub fn unprime(app: &App) {
    let runtime = Runtime::new(
        app.context.config.get_game_path(&app.active_game).clone(),
        Runtime::transaction_path(&app.active_game),
    );

    match runtime.recover() {
        Ok(true) => println!("==> Unprimed {}", app.active_game),
        Ok(false) => println!("==> {} is not primed", app.active_game),
        Err(error) => eprintln!("Failed to unprime {}: {error}", app.active_game),
    }
}
