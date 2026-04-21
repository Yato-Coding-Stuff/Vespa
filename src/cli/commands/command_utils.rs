use std::path::PathBuf;

use dialoguer::Input;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use crate::{
    packages::{SilkSongFlattenedPackage, SilkSongInstalledPackageRecord},
    util::{config::GameSwitcher, context::Context},
};

pub fn get_input_in_index(
    ctx: &mut Context,
    packages: Vec<String>,
) -> Result<Vec<Option<SilkSongFlattenedPackage>>, String> {
    let mut optional_packages: Vec<Option<SilkSongFlattenedPackage>> = Vec::new();

    for package in packages {
        if let Some(package) = ctx
            .index
            .get_package_by_full_name_with_version(&package)
            .or_else(|| ctx.index.get_latest_package_by_package_name(&package))
        {
            println!(
                "==> Exact match found: {}",
                package.package_full_name_with_version
            );
            optional_packages.push(Some(package));
        } else {
            let matcher = SkimMatcherV2::default();

            let package = package.to_lowercase();
            let mut matches = ctx
                .index
                .latest_full_name_by_package_name
                .iter()
                .filter_map(|(name, pkg)| {
                    matcher
                        .fuzzy_match(&name.to_lowercase(), &package)
                        .filter(|score| *score > 100)
                        .map(|score| (name, pkg, score))
                })
                .collect::<Vec<_>>();

            matches.sort_by(|a, b| b.2.cmp(&a.2));

            if matches.is_empty() {
                println!("==> No matches found for: {}", package);
                optional_packages.push(None);
                continue;
            }

            if matches.len() == 1 {
                let selected_package = ctx
                    .index
                    .get_package_by_full_name_with_version(&matches[0].1.clone())
                    .unwrap();
                optional_packages.push(Some(selected_package));
                continue;
            }

            let matches: Vec<_> = matches.iter().take(3).cloned().collect();

            println!("Multiple matches found:");
            for (i, (name, _, _)) in matches.iter().enumerate() {
                println!("{}) {}", i + 1, name);
            }
            println!("{}) None", matches.len() + 1);

            let selection: usize = Input::new()
                .with_prompt("==> Select a package by number")
                .validate_with(|input: &usize| -> Result<(), String> {
                    if *input == 0 || *input > matches.len() + 1 {
                        Err("==> Invalid selection".to_string())
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .unwrap();

            if selection == matches.len() + 1 {
                optional_packages.push(None);
                continue;
            }

            let selected_package = ctx
                .index
                .get_package_by_full_name_with_version(&matches[selection - 1].1.clone())
                .unwrap();

            optional_packages.push(Some(selected_package));
        }
    }
    if optional_packages.iter().all(|p| p.is_none()) {
        return Err("==> No matches found".to_string());
    }

    Ok(optional_packages)
}

pub fn get_input_in_tracker(
    ctx: &mut Context,
    packages: Vec<String>,
) -> Result<Vec<Option<SilkSongInstalledPackageRecord>>, String> {
    let mut optional_packages: Vec<Option<SilkSongInstalledPackageRecord>> = Vec::new();
    for package in packages {
        if let Some(package) = ctx.tracker.get(&package) {
            println!(
                "==> Exact match found: {}",
                package.package_full_name_with_version
            );
            optional_packages.push(Some(package.clone()));
        } else {
            let matcher = SkimMatcherV2::default();

            let package = package.to_lowercase();
            let all_records = ctx.tracker.get_all();
            let mut matches = all_records
                .iter()
                .filter_map(|(name, pkg)| {
                    matcher
                        .fuzzy_match(&name.to_lowercase(), &package)
                        .filter(|score| *score > 100)
                        .map(|score| (name, pkg, score))
                })
                .collect::<Vec<_>>();

            matches.sort_by(|a, b| b.2.cmp(&a.2));

            if matches.is_empty() {
                println!("==> No matches found for: {}", package);
                optional_packages.push(None);
                continue;
            }

            if matches.len() == 1 {
                optional_packages.push(Some(matches[0].1.clone()));
                continue;
            }

            let matches: Vec<_> = matches.iter().take(3).cloned().collect();

            println!("Multiple matches found:");
            for (i, (name, _, _)) in matches.iter().enumerate() {
                println!("{}) {}", i + 1, name);
            }
            println!("{}) None", matches.len() + 1);

            let selection: usize = Input::new()
                .with_prompt("==> Select a package by number")
                .validate_with(|input: &usize| -> Result<(), String> {
                    if *input == 0 || *input > matches.len() + 1 {
                        Err("==> Invalid selection".to_string())
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .unwrap();

            if selection == matches.len() + 1 {
                optional_packages.push(None);
                continue;
            }

            let selected_package = ctx.tracker.get(&matches[selection - 1].0.clone()).unwrap();

            optional_packages.push(Some(selected_package.clone()));
        }
    }
    if optional_packages.iter().all(|p| p.is_none()) {
        return Err("==> No matches found".to_string());
    }

    Ok(optional_packages)
}

pub fn resolve_profile_path(ctx: &Context, game: &GameSwitcher) -> Option<PathBuf> {
    ctx.config.get_default_profile(game).map(|profile| {
        let profile_path = PathBuf::from(&profile);
        if profile_path.is_absolute() {
            profile_path
        } else {
            crate::util::config::Config::config_dir()
                .join(game.profile_dir_name())
                .join(profile)
        }
    })
}

pub fn require_profile_path(ctx: &Context, game: &GameSwitcher) -> Result<PathBuf, String> {
    resolve_profile_path(ctx, game).ok_or_else(|| {
        "No default profile set. Run `vespa profile set-default <name>` or create one first."
            .to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::{require_profile_path, resolve_profile_path};
    use crate::{
        packages::SilkSongIndex,
        tracker::sk_package_tracker::SilkSongPackageTracker,
        util::{
            config::{Config, GameSwitcher},
            context::Context,
        },
    };

    fn context_with_defaults() -> Context {
        Context {
            config: Config {
                game_switcher: GameSwitcher::SilkSong,
                sk_default_profile: Some("default".to_string()),
                hk_default_profile: Some("default".to_string()),
                hollow_knight_path: "/games/hk".into(),
                silk_song_path: "/games/sk".into(),
                index_path: "/config/index.json".into(),
            },
            tracker: SilkSongPackageTracker::new(),
            index: SilkSongIndex::new(),
            black_list: vec![],
        }
    }

    #[test]
    fn resolve_profile_path_uses_game_specific_default() {
        let ctx = context_with_defaults();

        assert_eq!(
            resolve_profile_path(&ctx, &GameSwitcher::SilkSong).unwrap(),
            Config::config_dir().join("SK").join("default")
        );
        assert_eq!(
            resolve_profile_path(&ctx, &GameSwitcher::HollowKnight).unwrap(),
            Config::config_dir().join("HK").join("default")
        );
    }

    #[test]
    fn resolve_profile_path_keeps_absolute_profile_paths() {
        let mut ctx = context_with_defaults();
        ctx.config.sk_default_profile = Some("/profiles/SK/default".to_string());

        assert_eq!(
            resolve_profile_path(&ctx, &GameSwitcher::SilkSong).unwrap(),
            std::path::PathBuf::from("/profiles/SK/default")
        );
    }

    #[test]
    fn require_profile_path_returns_error_when_missing() {
        let mut ctx = context_with_defaults();
        ctx.config.sk_default_profile = None;

        let error = require_profile_path(&ctx, &GameSwitcher::SilkSong).unwrap_err();

        assert!(error.contains("No default profile set"));
    }
}
