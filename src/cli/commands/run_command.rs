use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::util::{config::GameSwitcher, context::Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    Windows,
    Unsupported,
}

#[derive(Debug, Clone)]
struct LaunchPlan {
    working_dir: PathBuf,
    program: String,
    args: Vec<String>,
}

struct RunCommandContext {
    pub os: OperatingSystem,
    pub game: GameSwitcher,
    pub game_path: PathBuf,
    pub profile_path: PathBuf,
}

impl RunCommandContext {
    fn new(ctx: &Context, game: &GameSwitcher, profile_path: &Path) -> Self {
        Self {
            os: current_os(),
            game: game.clone(),
            game_path: ctx.config.get_game_path(game).clone(),
            profile_path: profile_path.to_path_buf(),
        }
    }

    fn validate_run_context(&self) -> Result<(), String> {
        if !self.game_path.exists() {
            return Err(format!(
                "Game path does not exist: {}",
                self.game_path.display()
            ));
        }

        if !self.profile_path.exists() {
            return Err(format!(
                "Profile path does not exist: {}",
                self.profile_path.display()
            ));
        }

        let launcher = &self.profile_path.join("run_bepinex.sh");
        if !launcher.exists() {
            return Err(format!(
                "Profile launcher does not exist: {}",
                launcher.display()
            ));
        }

        let preloader = doorstop_target_assembly(&self.profile_path);
        if !preloader.exists() {
            return Err(format!(
                "Doorstop target assembly does not exist: {}",
                preloader.display()
            ));
        }

        Ok(())
    }
}

pub fn run(ctx: &Context, game: &GameSwitcher, profile_path: &Path) {
    let run_ctx = RunCommandContext::new(ctx, game, profile_path);

    if let Err(error) = run_ctx.validate_run_context() {
        eprintln!("Failed to prepare run command: {error}");
        return;
    }

    let launch_plan = match build_launch_plan(&run_ctx) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("Failed to build run command: {error}");
            return;
        }
    };

    let mut command = Command::new(&launch_plan.program);
    command
        .current_dir(&launch_plan.working_dir)
        .args(&launch_plan.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    match command.spawn() {
        Ok(child) => {
            println!(
                "==> Launched {} with profile {} (pid: {})",
                run_ctx.game,
                profile_display_name(&run_ctx.profile_path),
                child.id()
            );
        }
        Err(error) => {
            eprintln!("Failed to launch {}: {}", run_ctx.game, error);
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

fn build_launch_plan(run_ctx: &RunCommandContext) -> Result<LaunchPlan, String> {
    match run_ctx.os {
        OperatingSystem::Linux => build_linux_launch_plan(run_ctx),
        OperatingSystem::Windows => build_windows_launch_plan(run_ctx),
        OperatingSystem::Unsupported => Err("Unsupported operating system".to_string()),
    }
}

fn build_linux_launch_plan(run_ctx: &RunCommandContext) -> Result<LaunchPlan, String> {
    let game_executable = resolve_game_executable(run_ctx)?;

    Ok(LaunchPlan {
        working_dir: run_ctx.profile_path.clone(),
        program: "setsid".to_string(),
        args: vec![
            "sh".to_string(),
            run_ctx
                .profile_path
                .join("run_bepinex.sh")
                .to_string_lossy()
                .to_string(),
            game_executable.to_string_lossy().to_string(),
            "--doorstop-enabled".to_string(),
            "true".to_string(),
            "--doorstop-target-assembly".to_string(),
            doorstop_target_assembly(&run_ctx.profile_path)
                .to_string_lossy()
                .to_string(),
        ],
    })
}

fn build_windows_launch_plan(run_ctx: &RunCommandContext) -> Result<LaunchPlan, String> {
    let game_executable = resolve_game_executable(run_ctx)?;

    Ok(LaunchPlan {
        working_dir: run_ctx.profile_path.clone(),
        program: "cmd".to_string(),
        args: vec![
            "/c".to_string(),
            "start".to_string(),
            "\"\"".to_string(),
            game_executable.to_string_lossy().to_string(),
            "--doorstop-enabled".to_string(),
            "true".to_string(),
            "--doorstop-target-assembly".to_string(),
            doorstop_target_assembly(&run_ctx.profile_path)
                .to_string_lossy()
                .to_string(),
        ],
    })
}

fn resolve_game_executable(run_ctx: &RunCommandContext) -> Result<PathBuf, String> {
    let game_path = &run_ctx.game_path;

    if game_path.is_file() {
        return Ok(game_path.to_path_buf());
    }

    if !game_path.is_dir() {
        return Err(format!(
            "Game path is neither a file nor a directory: {}",
            game_path.display()
        ));
    }

    for candidate in game_executable_candidates(run_ctx) {
        let candidate_path = game_path.join(candidate);
        if candidate_path.is_file() {
            return Ok(candidate_path);
        }
    }

    let mut files = fs::read_dir(game_path)
        .map_err(|error| format!("Failed to read game directory: {error}"))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();

    files.sort();

    files
        .into_iter()
        .find(|path| path.extension().is_none())
        .ok_or_else(|| {
            format!(
                "Could not find a game executable in: {}",
                game_path.display()
            )
        })
}

fn game_executable_candidates(run_ctx: &RunCommandContext) -> &'static [&'static str] {
    match run_ctx.os {
        OperatingSystem::Linux => match run_ctx.game {
            GameSwitcher::HollowKnight => &["hollow_knight", "Hollow Knight"],
            GameSwitcher::SilkSong => &["silksong", "Silksong"],
        },
        OperatingSystem::Windows => match run_ctx.game {
            GameSwitcher::HollowKnight => &["hollow_knight.exe", "Hollow Knight.exe"],
            GameSwitcher::SilkSong => &["silksong.exe", "Silksong.exe"],
        },
        OperatingSystem::Unsupported => &[],
    }
}

fn doorstop_target_assembly(profile_path: &Path) -> PathBuf {
    profile_path
        .join("BepInEx")
        .join("core")
        .join("BepInEx.Preloader.dll")
}

fn current_os() -> OperatingSystem {
    if cfg!(target_os = "linux") {
        OperatingSystem::Linux
    } else if cfg!(target_os = "windows") {
        OperatingSystem::Windows
    } else {
        OperatingSystem::Unsupported
    }
}

#[cfg(test)]
mod tests {
    use super::{
        OperatingSystem, RunCommandContext, build_launch_plan, doorstop_target_assembly,
        profile_display_name,
    };
    use crate::util::config::GameSwitcher;
    use std::{fs, path::Path};
    use tempfile::tempdir;

    #[test]
    fn profile_display_name_uses_last_path_component() {
        assert_eq!(
            profile_display_name(Path::new("/profiles/silksong/default")),
            "default"
        );
    }

    #[test]
    fn build_launch_plan_for_linux_includes_doorstop_arguments() {
        let temp_dir = tempdir().unwrap();
        let game_dir = temp_dir.path().join("game");
        let profile_dir = temp_dir.path().join("profile");
        fs::create_dir_all(&game_dir).unwrap();
        fs::create_dir_all(profile_dir.join("BepInEx").join("core")).unwrap();
        fs::write(game_dir.join("hollow_knight"), "bin").unwrap();
        fs::write(profile_dir.join("run_bepinex.sh"), "#!/bin/sh\n").unwrap();
        fs::write(doorstop_target_assembly(&profile_dir), "dll").unwrap();

        let run_ctx = RunCommandContext {
            os: OperatingSystem::Linux,
            game: GameSwitcher::HollowKnight,
            game_path: game_dir.clone(),
            profile_path: profile_dir.clone(),
        };

        let plan = build_launch_plan(&run_ctx).unwrap();

        assert_eq!(plan.program, "setsid");
        assert_eq!(plan.working_dir, profile_dir);
        assert_eq!(plan.args[0], "sh");
        assert_eq!(
            plan.args[1],
            profile_dir
                .join("run_bepinex.sh")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(
            plan.args[2],
            game_dir.join("hollow_knight").to_string_lossy().to_string()
        );
        assert!(plan.args.contains(&"--doorstop-enabled".to_string()));
        assert!(plan.args.contains(&"true".to_string()));
        assert!(
            plan.args.contains(
                &doorstop_target_assembly(&profile_dir)
                    .to_string_lossy()
                    .to_string()
            )
        );
    }
}
