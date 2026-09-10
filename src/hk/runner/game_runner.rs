use std::{fs, path::PathBuf};

use crate::base::{
    runner::game_runner::{GameRunner, GameRunnerError, LaunchPlan, LaunchRequest},
    util::platform::OperatingSystem,
};

pub struct HkGameRunner;

impl GameRunner for HkGameRunner {
    fn build_launch_plan(&self, request: &LaunchRequest) -> Result<LaunchPlan, GameRunnerError> {
        validate_request(request)?;

        match request.os {
            OperatingSystem::Linux => build_linux_launch_plan(request),
            OperatingSystem::Windows => build_windows_launch_plan(request),
            OperatingSystem::MacOS => build_macos_launch_plan(request),
            OperatingSystem::Unsupported => Err(GameRunnerError::UnsupportedOperatingSystem),
        }
    }
}

fn validate_request(request: &LaunchRequest) -> Result<(), GameRunnerError> {
    if !request.game_path.exists() {
        return Err(GameRunnerError::GamePathMissing(
            request.game_path.to_path_buf(),
        ));
    }

    if !request.profile_path.exists() {
        return Err(GameRunnerError::ProfilePathMissing(
            request.profile_path.to_path_buf(),
        ));
    }

    Ok(())
}

fn build_linux_launch_plan(request: &LaunchRequest) -> Result<LaunchPlan, GameRunnerError> {
    let game_executable = resolve_game_executable(request)?;

    Ok(LaunchPlan {
        program: PathBuf::from("setsid"),
        working_dir: game_working_directory(request),
        args: vec![game_executable.into_os_string()],
        env: Vec::new(),
    })
}

fn build_windows_launch_plan(request: &LaunchRequest) -> Result<LaunchPlan, GameRunnerError> {
    let game_executable = resolve_game_executable(request)?;

    Ok(LaunchPlan {
        program: game_executable,
        working_dir: game_working_directory(request),
        args: Vec::new(),
        env: Vec::new(),
    })
}

fn build_macos_launch_plan(request: &LaunchRequest) -> Result<LaunchPlan, GameRunnerError> {
    let game_executable = resolve_game_executable(request)?;

    Ok(LaunchPlan {
        program: game_executable,
        working_dir: game_working_directory(request),
        args: Vec::new(),
        env: Vec::new(),
    })
}

fn game_working_directory(request: &LaunchRequest) -> PathBuf {
    if request.game_path.is_dir() {
        request.game_path.to_path_buf()
    } else {
        request
            .game_path
            .parent()
            .unwrap_or(request.game_path)
            .to_path_buf()
    }
}

fn resolve_game_executable(request: &LaunchRequest) -> Result<PathBuf, GameRunnerError> {
    let game_path = request.game_path;

    if game_path.is_file() {
        return Ok(game_path.to_path_buf());
    }

    if !game_path.is_dir() {
        return Err(GameRunnerError::GamePathMissing(game_path.to_path_buf()));
    }

    for candidate in game_executable_candidates(request.os) {
        let candidate_path = game_path.join(candidate);
        if candidate_path.is_file() {
            return Ok(candidate_path);
        }
    }

    let mut files = fs::read_dir(game_path)
        .map_err(GameRunnerError::GameDirectoryRead)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();

    files.sort();

    files
        .into_iter()
        .find(|path| path.extension().is_none())
        .ok_or_else(|| GameRunnerError::ExecutableMissing(game_path.to_path_buf()))
}

fn game_executable_candidates(os: OperatingSystem) -> &'static [&'static str] {
    match os {
        OperatingSystem::Linux => &["Hollow Knight.x86_64", "hollow_knight.x86_64"],
        OperatingSystem::Windows => &["hollow_knight.exe", "Hollow Knight.exe"],
        OperatingSystem::MacOS => &["Hollow Knight.app/Contents/MacOS/Hollow Knight"],
        OperatingSystem::Unsupported => &[],
    }
}
