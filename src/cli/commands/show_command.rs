use crate::util::context::Context;

pub fn show(ctx: &mut Context, package: String) {
    match ctx.index.initialize(&ctx.black_list) {
        Ok(_) => {}
        Err(e) => {
            println!("==> {}", e);
            return;
        }
    }
    let package_opt = ctx
        .index
        .get_package_by_full_name_with_version(&package)
        .or_else(|| ctx.index.get_latest_package_by_package_name(&package));

    if let Some(package) = package_opt {
        println!("==> package name: {}", package.package_full_name);
        println!("==> package version: {}", package.version_number);
        println!("==> package description: {}", package.description);
        println!("==> package download url: {}", package.download_url);
        println!("==> package dependencies: {:?}", package.dependencies);
    } else {
        println!("==> Mod not found: {}", package);
    }
}
