use std::path::{Path, PathBuf};

pub(crate) fn package_root(profile_path: &Path) -> PathBuf {
    profile_path.join("BepInEx").join("plugins")
}
