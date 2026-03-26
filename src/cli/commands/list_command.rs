use crate::util::context::Context;

pub fn list(ctx: &mut Context, packages: Vec<String>, available: bool, all_versions: bool) {
    if available {
        match ctx.index.initialize(&ctx.black_list) {
            Ok(_) => {}
            Err(e) => {
                println!("==> {}", e);
                return;
            }
        }

        list_available(ctx, packages, all_versions);
    } else {
        list_installed(ctx, packages);
    }
}

pub fn list_available(ctx: &mut Context, queries: Vec<String>, all_versions: bool) {
    let matches: Vec<_> = ctx
        .index
        .all_versions_by_full_name
        .iter()
        .filter(|(package_name, _)| {
            queries.is_empty()
                || queries
                    .iter()
                    .any(|q| package_name.to_lowercase().contains(&q.to_lowercase()))
        })
        .collect();

    if matches.is_empty() {
        for query in queries {
            println!("==> No matches found for '{}'", query);
        }
        return;
    }

    for (package_name, versions) in matches {
        if !queries.is_empty()
            && !queries
                .iter()
                .any(|q| package_name.to_lowercase().contains(&q.to_lowercase()))
        {
            continue;
        }

        if all_versions {
            println!("==> {}", package_name);
            for ver in versions {
                println!("-> {}", ver.package_full_name_with_version);
            }
        } else if let Some(latest) = versions
            .iter()
            .max_by(|a, b| a.version_number.cmp(&b.version_number))
        {
            println!("==> {}", latest.package_full_name_with_version);
        }
    }
}

pub fn list_installed(ctx: &mut Context, query: Vec<String>) {
    let all_packages = ctx.tracker.get_all();
    let filtered: Vec<_> = all_packages
        .values()
        .filter(|p| {
            query.is_empty()
                || query.iter().any(|q| {
                    p.package_full_name
                        .to_lowercase()
                        .contains(&q.to_lowercase())
                })
        })
        .collect();

    if filtered.is_empty() {
        for query in query {
            println!("==> No matches found for '{}'", query);
        }
        return;
    }

    for pkg in filtered {
        println!("==> {}", pkg.package_full_name_with_version);
    }
}
