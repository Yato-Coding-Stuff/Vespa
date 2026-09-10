use std::path::Path;

use crate::{
    app::application::App,
    base::runtime::{Runtime, primer::Primer},
};

/// Runs a profile mutation while preserving the primed game state.
pub fn run<F>(app: &mut App, profile_path: &Path, operation: F) -> Result<(), String>
where
    F: FnOnce(&mut App) -> Result<(), String>,
{
    let runtime = Runtime::new(
        app.context.config.get_game_path(&app.active_game).clone(),
        Runtime::transaction_path(&app.active_game),
    );
    let was_primed = runtime.recover()?;

    let operation_result = operation(app);

    if !was_primed {
        return operation_result;
    }
    let desired = (Primer {}).files_from_profile(profile_path)?;
    let runtime = Runtime::new(
        app.context.config.get_game_path(&app.active_game).clone(),
        Runtime::transaction_path(&app.active_game),
    );
    let prime_result = runtime.prime(
        app.active_game.clone(),
        profile_path.display().to_string(),
        desired,
    );
    match (operation_result, prime_result) {
        (Err(operation_error), Err(prime_error)) => Err(format!(
            "{operation_error}; additionally failed to restore priming: {prime_error}"
        )),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}
