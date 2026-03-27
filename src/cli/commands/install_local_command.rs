use dialoguer::Confirm;
use std::path::PathBuf;

use crate::manager::sk_package_installer;
use crate::util::context::Context;

pub fn install(ctx: &mut Context, package_paths: Vec<PathBuf>, profile_path: &PathBuf) {
    match ctx.index.initialize(&ctx.black_list) {
        Ok(_) => (),
        Err(e) => {
            println!("==> {}", e);
            return;
        }
    }

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to install the following local packages?\n - {}",
            package_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
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

    for package in package_paths {
        let package_installer = sk_package_installer::SilkSongPackageInstaller::new();
        match package_installer.install_local_package(&package, profile_path) {
            Ok(()) => {
                println!("==> Installed {}", package.to_string_lossy());
            }
            Err(e) => {
                println!("==> {}", e);
                return;
            }
        }
    }
}
