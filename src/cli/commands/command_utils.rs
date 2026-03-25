use dialoguer::Input;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use crate::{packages::SilkSongFlattenedPackage, util::context::Context};

pub fn input_handling(
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

            let mut matches = ctx
                .index
                .latest_full_name_by_package_name
                .iter()
                .filter_map(|(name, pkg)| {
                    matcher
                        .fuzzy_match(name, &package)
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
