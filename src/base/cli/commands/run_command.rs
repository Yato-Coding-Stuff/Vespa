use std::{path::Path, process::Command};

use crate::{
    app::application::App,
    base::{
        runner::game_runner::LaunchRequest,
        runtime::{Runtime, primer::Primer},
        util::platform::OperatingSystem,
    },
};

pub fn run(app: &App, profile_path: &Path) {
    let request = LaunchRequest {
        os: OperatingSystem::current(),
        game_path: app.context.config.get_game_path(&app.active_game),
        profile_path,
    };

    let runtime = Runtime::new(
        app.context.config.get_game_path(&app.active_game).clone(),
        Runtime::transaction_path(&app.active_game),
    );
    if let Err(error) = runtime.recover() {
        eprintln!("Failed to recover the previous runtime transaction: {error}");
        return;
    }

    let plan = match app.game_services.game_runner.build_launch_plan(&request) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("Failed to prepare game launch: {error}");
            return;
        }
    };

    let desired = match (Primer {}).files_from_profile(profile_path) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("Failed to inspect profile: {error}");
            return;
        }
    };
    let game = app.active_game.clone();
    let profile = profile_display_name(profile_path);
    let result = runtime.with_transaction(game.clone(), profile.clone(), desired, || {
        let mut command = Command::new(&plan.program);
        command
            .current_dir(&plan.working_dir)
            .args(&plan.args)
            .envs(plan.env);
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        println!(
            "==> Launched {} with profile {} (pid: {})",
            game,
            profile,
            child.id()
        );
        let status = child.wait().map_err(|error| error.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("game exited with status {status}"))
        }
    });
    if let Err(error) = result {
        eprintln!("Failed to run {}: {}", app.active_game, error);
    }
}

fn profile_display_name(profile_path: &Path) -> String {
    profile_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| profile_path.display().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::profile_display_name;

    #[test]
    fn profile_display_name_uses_last_path_component() {
        assert_eq!(
            profile_display_name(Path::new("/profiles/silksong/default")),
            "default"
        );
    }
}
