use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use crate::base::runner::game_runner::{
    GameRunner, GameRunnerError, LaunchPlan, LaunchRequest, OperatingSystem,
};

pub struct SkGameRunner;

impl GameRunner for SkGameRunner {
    fn build_launch_plan(&self, request: &LaunchRequest) -> Result<LaunchPlan, GameRunnerError> {
        validate_request(request)?;

        match request.os {
            OperatingSystem::Linux => build_linux_launch_plan(request),
            OperatingSystem::Windows => build_windows_launch_plan(request),
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

    let launcher = request.profile_path.join("run_bepinex.sh");
    if !launcher.exists() {
        return Err(GameRunnerError::RuntimeFileMissing(launcher));
    }

    let preloader = doorstop_target_assembly(request.profile_path);
    if !preloader.exists() {
        return Err(GameRunnerError::RuntimeFileMissing(preloader));
    }

    Ok(())
}

fn build_linux_launch_plan(request: &LaunchRequest) -> Result<LaunchPlan, GameRunnerError> {
    let game_executable = resolve_game_executable(request)?;

    Ok(LaunchPlan {
        program: PathBuf::from("setsid"),
        working_dir: request.profile_path.to_path_buf(),
        args: vec![
            OsString::from("sh"),
            request.profile_path.join("run_bepinex.sh").into_os_string(),
            game_executable.into_os_string(),
            OsString::from("--doorstop-enabled"),
            OsString::from("true"),
            OsString::from("--doorstop-target-assembly"),
            doorstop_target_assembly(request.profile_path).into_os_string(),
        ],
        env: Vec::new(),
    })
}

fn build_windows_launch_plan(request: &LaunchRequest) -> Result<LaunchPlan, GameRunnerError> {
    let game_executable = resolve_game_executable(request)?;

    Ok(LaunchPlan {
        program: PathBuf::from("cmd"),
        working_dir: request.profile_path.to_path_buf(),
        args: vec![
            OsString::from("/c"),
            OsString::from("start"),
            OsString::from("\"\""),
            game_executable.into_os_string(),
            OsString::from("--doorstop-enabled"),
            OsString::from("true"),
            OsString::from("--doorstop-target-assembly"),
            doorstop_target_assembly(request.profile_path).into_os_string(),
        ],
        env: Vec::new(),
    })
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
        OperatingSystem::Linux => &["silksong", "Silksong"],
        OperatingSystem::Windows => &["silksong.exe", "Silksong.exe"],
        OperatingSystem::Unsupported => &[],
    }
}

fn doorstop_target_assembly(profile_path: &Path) -> PathBuf {
    profile_path
        .join("BepInEx")
        .join("core")
        .join("BepInEx.Preloader.dll")
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs, path::PathBuf};

    use tempfile::tempdir;

    use super::{SkGameRunner, doorstop_target_assembly};
    use crate::base::runner::game_runner::{GameRunner, LaunchRequest, OperatingSystem};

    #[test]
    fn linux_plan_includes_doorstop_arguments() {
        let temp_dir = tempdir().unwrap();
        let game_dir = temp_dir.path().join("game");
        let profile_dir = temp_dir.path().join("profile");
        fs::create_dir_all(&game_dir).unwrap();
        fs::create_dir_all(profile_dir.join("BepInEx").join("core")).unwrap();
        fs::write(game_dir.join("silksong"), "bin").unwrap();
        fs::write(profile_dir.join("run_bepinex.sh"), "#!/bin/sh\n").unwrap();
        fs::write(doorstop_target_assembly(&profile_dir), "dll").unwrap();

        let request = LaunchRequest {
            os: OperatingSystem::Linux,
            game_path: &game_dir,
            profile_path: &profile_dir,
        };

        let plan = SkGameRunner.build_launch_plan(&request).unwrap();

        assert_eq!(plan.program, PathBuf::from("setsid"));
        assert_eq!(plan.working_dir, profile_dir);
        assert_eq!(plan.args[0], OsString::from("sh"));
        assert_eq!(
            plan.args[1],
            profile_dir.join("run_bepinex.sh").into_os_string()
        );
        assert_eq!(plan.args[2], game_dir.join("silksong").into_os_string());
        assert!(plan.args.contains(&OsString::from("--doorstop-enabled")));
        assert!(plan.args.contains(&OsString::from("true")));
        assert!(
            plan.args
                .contains(&doorstop_target_assembly(&profile_dir).into_os_string())
        );
    }
}
