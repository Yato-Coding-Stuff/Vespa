use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::util::{config::GameSwitcher, context::Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperatingSystem {
    Linux,
    Mac,
    Windows,
    Unsupported,
}

#[derive(Debug, Clone)]
struct LaunchPlan {
    working_dir: PathBuf,
    program: String,
    args: Vec<String>,
}

pub struct RunCommandContext {
    pub game: GameSwitcher,
    pub game_path: PathBuf,
    pub profile_path: PathBuf,
}

impl RunCommandContext {
    pub fn from_context(ctx: &Context, game: &GameSwitcher, profile_path: &Path) -> Self {
        Self {
            game: game.clone(),
            game_path: ctx.config.get_game_path(game).clone(),
            profile_path: profile_path.to_path_buf(),
        }
    }
}

pub fn run(ctx: &Context, game: &GameSwitcher, profile_path: &Path) {
    let run_ctx = RunCommandContext::from_context(ctx, game, profile_path);

    if let Err(error) = validate_run_context(&run_ctx) {
        eprintln!("Failed to prepare run command: {error}");
        return;
    }

    let os = current_os();
    let launch_plan = match build_launch_plan(&run_ctx, os) {
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
            println!("==> Launched {} (pid: {})", run_ctx.game, child.id());
        }
        Err(error) => {
            eprintln!("Failed to launch {}: {}", run_ctx.game, error);
        }
    }
}

fn validate_run_context(run_ctx: &RunCommandContext) -> Result<(), String> {
    if !run_ctx.game_path.exists() {
        return Err(format!(
            "Game path does not exist: {}",
            run_ctx.game_path.display()
        ));
    }

    if !run_ctx.profile_path.exists() {
        return Err(format!(
            "Profile path does not exist: {}",
            run_ctx.profile_path.display()
        ));
    }

    let launcher = linux_profile_launcher(&run_ctx.profile_path);
    if !launcher.exists() {
        return Err(format!(
            "Profile launcher does not exist: {}",
            launcher.display()
        ));
    }

    let preloader = doorstop_target_assembly(&run_ctx.profile_path);
    if !preloader.exists() {
        return Err(format!(
            "Doorstop target assembly does not exist: {}",
            preloader.display()
        ));
    }

    Ok(())
}

fn build_launch_plan(
    run_ctx: &RunCommandContext,
    os: OperatingSystem,
) -> Result<LaunchPlan, String> {
    match os {
        OperatingSystem::Linux => build_linux_launch_plan(run_ctx),
        OperatingSystem::Mac => Err("macOS launch is not implemented yet".to_string()),
        OperatingSystem::Windows => Err("Windows launch is not implemented yet".to_string()),
        OperatingSystem::Unsupported => Err("Unsupported operating system".to_string()),
    }
}

fn build_linux_launch_plan(run_ctx: &RunCommandContext) -> Result<LaunchPlan, String> {
    let game_executable = resolve_linux_game_executable(&run_ctx.game, &run_ctx.game_path)?;

    Ok(LaunchPlan {
        working_dir: run_ctx.profile_path.clone(),
        program: "setsid".to_string(),
        args: vec![
            "sh".to_string(),
            "./run_bepinex.sh".to_string(),
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

fn linux_profile_launcher(profile_path: &Path) -> PathBuf {
    profile_path.join("run_bepinex.sh")
}

fn resolve_linux_game_executable(game: &GameSwitcher, game_path: &Path) -> Result<PathBuf, String> {
    if game_path.is_file() {
        return Ok(game_path.to_path_buf());
    }

    if !game_path.is_dir() {
        return Err(format!(
            "Game path is neither a file nor a directory: {}",
            game_path.display()
        ));
    }

    for candidate in linux_game_executable_candidates(game) {
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
                "Could not find a Linux game executable in: {}",
                game_path.display()
            )
        })
}

fn linux_game_executable_candidates(game: &GameSwitcher) -> &'static [&'static str] {
    match game {
        GameSwitcher::HollowKnight => &["hollow_knight", "Hollow Knight"],
        GameSwitcher::SilkSong => &["silksong", "Silksong"],
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
    } else if cfg!(target_os = "macos") {
        OperatingSystem::Mac
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
        linux_profile_launcher, resolve_linux_game_executable,
    };
    use crate::util::config::GameSwitcher;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn linux_profile_launcher_uses_profile_run_script() {
        let temp_dir = tempdir().unwrap();
        let profile_dir = temp_dir.path().join("profile");

        let launcher = linux_profile_launcher(&profile_dir);

        assert_eq!(launcher, profile_dir.join("run_bepinex.sh"));
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
            game: GameSwitcher::HollowKnight,
            game_path: game_dir.clone(),
            profile_path: profile_dir.clone(),
        };

        let plan = build_launch_plan(&run_ctx, OperatingSystem::Linux).unwrap();

        assert_eq!(plan.program, "setsid");
        assert_eq!(plan.working_dir, profile_dir);
        assert_eq!(plan.args[0], "sh");
        assert_eq!(plan.args[1], "./run_bepinex.sh");
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

    #[test]
    fn resolve_linux_game_executable_accepts_file_path_directly() {
        let temp_dir = tempdir().unwrap();
        let executable = temp_dir.path().join("hollow_knight");
        fs::write(&executable, "bin").unwrap();

        let resolved =
            resolve_linux_game_executable(&GameSwitcher::HollowKnight, &executable).unwrap();

        assert_eq!(resolved, executable);
    }
}
