use dialoguer::Confirm;

use crate::{
    app::application::App,
    base::{
        cli::{commands::command_utils, presenter::presenter::Presenter},
        packages::InstalledPackageRecord,
    },
};

pub fn disable(app: &mut App, presenter: &mut Presenter, packages: Vec<String>) {
    let mut presenter = |event| presenter.display(&event);

    let pm = app.game_services.package_manager.as_ref();
    let ctx = &mut app.context;

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
        .collect::<Vec<&InstalledPackageRecord>>();

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to disable the following packages?\n - {}",
            packages
                .iter()
                .map(|p| p.identifier.clone())
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

pub fn enable(app: &mut App, presenter: &mut Presenter, packages: Vec<String>) {
    let mut presenter = |event| presenter.display(&event);

    let pm = app.game_services.package_manager.as_ref();
    let ctx = &mut app.context;

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
        .collect::<Vec<&InstalledPackageRecord>>();

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to enable the following packages?\n - {}",
            packages
                .iter()
                .map(|p| p.identifier.clone())
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
