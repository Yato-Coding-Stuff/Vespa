#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
    Unsupported,
}

impl OperatingSystem {
    pub fn current() -> OperatingSystem {
        if cfg!(target_os = "linux") {
            OperatingSystem::Linux
        } else if cfg!(target_os = "macos") {
            OperatingSystem::MacOS
        } else if cfg!(target_os = "windows") {
            OperatingSystem::Windows
        } else {
            OperatingSystem::Unsupported
        }
    }
}
