use dialoguer::Input;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use crate::{packages::SilkSongFlattenedPackage, util::context::Context};

pub fn input_handling(
    ctx: &mut Context,
    packages: Vec<String>,
) -> Vec<Option<SilkSongFlattenedPackage>> {
    let mut optional_packages: Vec<Option<SilkSongFlattenedPackage>> = Vec::new();

    for package in packages {
        if let Some(package) = ctx.index.get_package_by_full_name(&package) {
            println!(
                "==> Exact match found: {}",
                package.package_full_name_with_version
            );
            optional_packages.push(Some(package));
        } else {
            println!("==> No exact match found for: {}", package);
            let matcher = SkimMatcherV2::default();

            let mut matches = ctx
                .index
                .latest_full_name_by_package_name
                .iter()
                .filter_map(|(name, pkg)| {
                    matcher
                        .fuzzy_match(name, &package)
                        .map(|score| (name, pkg, score))
                })
                .collect::<Vec<_>>();

            matches.sort_by(|a, b| b.2.cmp(&a.2));

            if matches.is_empty() {
                optional_packages.push(None);
                continue;
            }

            let matches: Vec<_> = matches.iter().take(3).cloned().collect();

            println!("Multiple matches found:");
            for (i, (name, _, score)) in matches.iter().enumerate() {
                println!("{}) {} (score {})", i + 1, name, score);
            }
            println!("{}) None", matches.len() + 1);

            let selection: usize = Input::new()
                .with_prompt("==> Select a package by number")
                .interact()
                .unwrap();

            if selection == matches.len() + 1 {
                optional_packages.push(None);
                continue;
            }

            let selected_package = ctx
                .index
                .get_package_by_full_name(&matches[selection - 1].1.clone())
                .unwrap();

            optional_packages.push(Some(selected_package));
        }
    }

    optional_packages
}
