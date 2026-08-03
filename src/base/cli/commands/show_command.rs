use crate::app::application::App;

pub fn show(app: &mut App, package: String) {
    match app.initialize_index() {
        Ok(_) => {}
        Err(e) => {
            println!("==> {}", e);
            return;
        }
    }
    let package_opt = app
        .context
        .index
        .get_package_by_identifier(&package)
        .or_else(|| {
            app.context
                .index
                .get_latest_package_by_package_name(&package)
        });

    if let Some(package) = package_opt {
        println!("==> package name: {}", package.name);
        println!("==> package version: {}", package.version);
        println!("==> package description: {}", package.description);
        println!("==> package download url: {}", package.download_url);
        println!("==> package dependencies: {:?}", package.dependencies);
    } else {
        println!("==> Mod not found: {}", package);
    }
}
