use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    Windows,
    Unsupported,
}

impl OperatingSystem {
    pub fn current() -> OperatingSystem {
        if cfg!(target_os = "linux") {
            OperatingSystem::Linux
        } else if cfg!(target_os = "windows") {
            OperatingSystem::Windows
        } else {
            OperatingSystem::Unsupported
        }
    }
}

pub struct LaunchRequest<'a> {
    pub os: OperatingSystem,
    pub game_path: &'a Path,
    pub profile_path: &'a Path,
}

pub struct LaunchPlan {
    pub program: PathBuf,
    pub working_dir: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
}

#[derive(Debug, thiserror::Error)]
pub enum GameRunnerError {
    #[error("unsupported operating system")]
    UnsupportedOperatingSystem,

    #[error("game path does not exist: {0}")]
    GamePathMissing(PathBuf),

    #[error("profile path does not exist: {0}")]
    ProfilePathMissing(PathBuf),

    #[error("required runtime file does not exist: {0}")]
    RuntimeFileMissing(PathBuf),

    #[error("game executable was not found in: {0}")]
    ExecutableMissing(PathBuf),

    #[error("failed to read game directory: {0}")]
    GameDirectoryRead(#[source] std::io::Error),
}

pub trait GameRunner {
    fn build_launch_plan(&self, request: &LaunchRequest) -> Result<LaunchPlan, GameRunnerError>;
}
