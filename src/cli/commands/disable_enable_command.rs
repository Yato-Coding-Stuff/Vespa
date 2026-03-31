use dialoguer::Confirm;

use crate::{
    cli::{commands::command_utils, presenter::presenter::Presenter},
    manager::sk_package_manager::SilkSongPackageManager,
    packages::SilkSongInstalledPackageRecord,
    util::context::Context,
};

pub fn disable(ctx: &mut Context, presenter: &mut Presenter, packages: Vec<String>) {
    let mut presenter = |event| presenter.display(&event);

    let pm = SilkSongPackageManager::new();

    let packages = match command_utils::get_input_in_tracker(ctx, packages) {
        Ok(packages) => packages,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let packages = packages
        .iter()
        .filter_map(|p| p.as_ref())
        .collect::<Vec<&SilkSongInstalledPackageRecord>>();

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to disable the following packages?\n - {}",
            packages
                .iter()
                .map(|p| p.package_full_name_with_version.clone())
                .collect::<Vec<String>>()
                .join("\n - ")
        ))
        .report(false)
        .interact()
        .unwrap()
    {
        println!("==> Aborted installation");
        return;
    }

    for package in packages {
        match pm.disable_package(&mut presenter, package) {
            Ok(_) => (),
            Err(e) => println!("==> {}", e),
        }
    }
}

pub fn enable(ctx: &mut Context, presenter: &mut Presenter, packages: Vec<String>) {
    let mut presenter = |event| presenter.display(&event);

    let pm = SilkSongPackageManager::new();

    let packages = match command_utils::get_input_in_tracker(ctx, packages) {
        Ok(packages) => packages,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let packages = packages
        .iter()
        .filter_map(|p| p.as_ref())
        .collect::<Vec<&SilkSongInstalledPackageRecord>>();

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to enable the following packages?\n - {}",
            packages
                .iter()
                .map(|p| p.package_full_name_with_version.clone())
                .collect::<Vec<String>>()
                .join("\n - ")
        ))
        .report(false)
        .interact()
        .unwrap()
    {
        println!("==> Aborted installation");
        return;
    }

    for package in packages {
        match pm.enable_package(&mut presenter, package) {
            Ok(_) => (),
            Err(e) => println!("==> {}", e),
        }
    }
}
