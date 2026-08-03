use crate::{
    app::application::App,
    base::{
        cli::presenter::presenter::Presenter,
    },
};

pub fn list(app: &mut App) {
    let ctx = &mut app.context;
    let game = &app.active_game;
    let profile_manager = app.game_services.profile_manager.as_ref();

    match profile_manager.list_profiles(game) {
        Ok(profiles) => {
            let default_profile = ctx.config.get_default_profile(game);

            if profiles.is_empty() {
                println!("==> No profiles found for {}", game);
                return;
            }

            println!("==> Profiles for {}:", game);
            for profile in profiles {
                let suffix = if default_profile.as_deref() == Some(profile.as_str()) {
                    " [default]"
                } else {
                    ""
                };
                println!("-> {}{}", profile, suffix);
            }
        }
        Err(error) => {
            println!("Failed to list profiles: {}", error);
        }
    }
}

pub fn create(app: &mut App, presenter: &mut Presenter, profile: String) {
    let ctx = &mut app.context;
    let game = &app.active_game;
    let profile_manager = app.game_services.profile_manager.as_ref();

    match profile_manager.create_profile(ctx, presenter, game, &profile) {
        Ok(_) => (),
        Err(error) => {
            println!("Failed to create profile: {}", error);
        }
    }
}

pub fn delete(app: &mut App, presenter: &mut Presenter, profile: String) {
    let ctx = &mut app.context;
    let game = &app.active_game;
    let profile_manager = app.game_services.profile_manager.as_ref();

    match profile_manager.delete_profile(ctx, presenter, game, &profile) {
        Ok(_) => (),
        Err(error) => {
            println!("Failed to delete profile: {}", error);
        }
    }
}

pub fn set_default(app: &mut App, presenter: &mut Presenter, profile: String) {
    let ctx = &mut app.context;
    let game = &app.active_game;
    let profile_manager = app.game_services.profile_manager.as_ref();

    match profile_manager.set_profile_as_default(ctx, presenter, game, &profile) {
        Ok(_) => (),
        Err(error) => {
            println!("Failed to set profile: {} as the default", error);
        }
    }
}
