use dialoguer::Confirm;

use crate::{
    app::application::App,
    base::{
        cli::{commands::command_utils, presenter::presenter::Presenter},
        packages::InstalledPackageRecord,
    },
};

pub fn disable(
    app: &mut App,
    presenter: &mut Presenter,
    packages: Vec<String>,
) -> Result<(), String> {
    let mut presenter = |event| presenter.display(&event);

    let pm = app.game_services.package_manager.as_ref();
    let ctx = &mut app.context;

    let packages = match command_utils::get_input_in_tracker(ctx, packages) {
        Ok(packages) => packages,
        Err(e) => {
            println!("{}", e);
            return Err(e);
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
        .map_err(|error| error.to_string())?
    {
        println!("==> Aborted installation");
        return Ok(());
    }

    let mut errors = Vec::new();
    for package in packages {
        match pm.disable_package(&mut presenter, package) {
            Ok(_) => (),
            Err(e) => errors.push(e.to_string()),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub fn enable(
    app: &mut App,
    presenter: &mut Presenter,
    packages: Vec<String>,
) -> Result<(), String> {
    let mut presenter = |event| presenter.display(&event);

    let pm = app.game_services.package_manager.as_ref();
    let ctx = &mut app.context;

    let packages = match command_utils::get_input_in_tracker(ctx, packages) {
        Ok(packages) => packages,
        Err(e) => {
            println!("{}", e);
            return Err(e);
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
        .map_err(|error| error.to_string())?
    {
        println!("==> Aborted installation");
        return Ok(());
    }

    let mut errors = Vec::new();
    for package in packages {
        match pm.enable_package(&mut presenter, package) {
            Ok(_) => (),
            Err(e) => errors.push(e.to_string()),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
