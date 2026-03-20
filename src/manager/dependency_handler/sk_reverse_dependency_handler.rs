use crate::util::context::Context;

pub struct SilkSongReverseDependencyHandler;

impl SilkSongReverseDependencyHandler {
    pub fn package_is_required(ctx: &Context, target: &str) -> bool {
        ctx.tracker.get_all().values().any(|installed_pkg| {
            match ctx.index.get_package_by_full_name_with_version(
                &installed_pkg.package_full_name_with_version,
            ) {
                Some(installed_info) => installed_info.dependencies.iter().any(|dep| {
                    let dep_name = dep.rsplitn(2, '-').nth(1).unwrap_or(dep);
                    dep_name == target
                }),
                None => false,
            }
        })
    }

    pub fn dependency_is_required(ctx: &Context, target: &str) -> bool {
        ctx.tracker.get_all().values().any(|installed_pkg| {
            match ctx.index.get_package_by_full_name_with_version(
                &installed_pkg.package_full_name_with_version,
            ) {
                Some(installed_package_info) => installed_package_info
                    .dependencies
                    .iter()
                    .any(|dep| dep == target),
                None => false,
            }
        })
    }
}
