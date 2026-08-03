use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::{
    app::application::App,
    base::runner::game_runner::{LaunchRequest, OperatingSystem},
};

pub fn run(app: &App, profile_path: &Path) {
    let request = LaunchRequest {
        os: OperatingSystem::current(),
        game_path: app.context.config.get_game_path(&app.active_game),
        profile_path,
    };

    let plan = match app.game_services.game_runner.build_launch_plan(&request) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("Failed to prepare game launch: {error}");
            return;
        }
    };

    let mut command = Command::new(&plan.program);
    command
        .current_dir(&plan.working_dir)
        .args(&plan.args)
        .envs(plan.env)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    match command.spawn() {
        Ok(child) => {
            println!(
                "==> Launched {} with profile {} (pid: {})",
                app.active_game,
                profile_display_name(profile_path),
                child.id()
            );
        }
        Err(error) => {
            eprintln!("Failed to launch {}: {}", app.active_game, error);
        }
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
