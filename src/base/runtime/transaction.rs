use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::base::util::config::GameSwitcher;

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub game: GameSwitcher,
    pub profile: String,
    pub tracked_files: Vec<TrackedFile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackedFile {
    pub profile_path: PathBuf,
    pub game_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub existed: bool,
}

impl Transaction {
    pub fn new(game: GameSwitcher, profile: String) -> Transaction {
        Transaction {
            game,
            profile,
            tracked_files: Vec::new(),
        }
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let serialized = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&serialized).map_err(|e| e.to_string())
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let serialized = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serialized).map_err(|e| e.to_string())?;
        fs::rename(&temporary, path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::util::config::GameSwitcher;
    use tempfile::tempdir;

    #[test]
    fn transaction_round_trips_optional_backup_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("transaction.json");
        let mut transaction = Transaction::new(GameSwitcher::HollowKnight, "Default".into());
        transaction.tracked_files.push(TrackedFile {
            profile_path: "/profile/mod.dll".into(),
            game_path: "/game/mod.dll".into(),
            backup_path: None,
            existed: false,
        });

        transaction.save_to_path(&path).unwrap();
        let loaded = Transaction::load_from_path(&path).unwrap();
        assert_eq!(loaded.tracked_files[0].backup_path, None);
        assert!(!loaded.tracked_files[0].existed);
    }
}
