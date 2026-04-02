use crate::{
    cli::presenter::presenter::Presenter,
    profile_manager::sk_profile_manager::SilkSongProfileManager,
    util::{
        config::{Config, GameSwitcher},
        context::Context,
    },
};

pub fn list(ctx: &Context, game: &GameSwitcher) {
    let profile_manager = SilkSongProfileManager::new(Config::config_dir());

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

pub fn create(ctx: &mut Context, presenter: &mut Presenter, game: &GameSwitcher, profile: String) {
    let profile_manager = SilkSongProfileManager::new(Config::config_dir());

    match profile_manager.create_profile(ctx, presenter, game, &profile) {
        Ok(_) => (),
        Err(error) => {
            println!("Failed to create profile: {}", error);
        }
    }
}

pub fn delete(ctx: &mut Context, presenter: &mut Presenter, game: &GameSwitcher, profile: String) {
    let profile_manager = SilkSongProfileManager::new(Config::config_dir());

    match profile_manager.delete_profile(ctx, presenter, game, &profile) {
        Ok(_) => (),
        Err(error) => {
            println!("Failed to delete profile: {}", error);
        }
    }
}

pub fn set_default(
    ctx: &mut Context,
    presenter: &mut Presenter,
    game: &GameSwitcher,
    profile: String,
) {
    let profile_manager = SilkSongProfileManager::new(Config::config_dir());

    match profile_manager.set_profile_as_default(ctx, presenter, game, &profile) {
        Ok(_) => (),
        Err(error) => {
            println!("Failed to set profile: {} as the default", error);
        }
    }
}
