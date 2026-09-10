use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::base::runtime::{
    file_synchronizer::{DesiredFile, FileSynchronizer},
    transaction::Transaction,
};

pub struct Runtime {
    synchronizer: FileSynchronizer,
    transaction_path: PathBuf,
}

impl Runtime {
    pub fn new(game_dir: PathBuf, transaction_path: PathBuf) -> Self {
        Self {
            synchronizer: FileSynchronizer::new(game_dir.clone()),
            transaction_path,
        }
    }

    pub fn transaction_path(game: &crate::base::util::config::GameSwitcher) -> PathBuf {
        crate::base::util::config::Config::config_dir()
            .join("runtime")
            .join(format!("{}.json", game.profile_dir_name()))
    }

    pub fn recover(&self) -> Result<bool, String> {
        if !self.transaction_path.exists() {
            return Ok(false);
        }
        let transaction = Transaction::load_from_path(&self.transaction_path)?;
        let expected_game = self
            .transaction_path
            .file_stem()
            .and_then(|name| name.to_str());
        if expected_game != Some(transaction.game.profile_dir_name()) {
            return Err("runtime transaction belongs to a different game".to_string());
        }
        self.synchronizer.restore(&transaction)?;
        self.remove_transaction(&transaction)?;
        Ok(true)
    }

    pub fn prime<I>(
        &self,
        game: crate::base::util::config::GameSwitcher,
        profile: String,
        desired: I,
    ) -> Result<(), String>
    where
        I: IntoIterator<Item = DesiredFile>,
    {
        self.recover()?;
        let mut transaction = Transaction::new(game, profile);
        transaction.save_to_path(&self.transaction_path)?;
        self.synchronizer
            .sync_with_checkpoint(desired, &mut transaction, |transaction| {
                transaction.save_to_path(&self.transaction_path)
            })
    }

    pub fn with_transaction<I, F, T>(
        &self,
        game: crate::base::util::config::GameSwitcher,
        profile: String,
        desired: I,
        operation: F,
    ) -> Result<T, String>
    where
        I: IntoIterator<Item = DesiredFile>,
        F: FnOnce() -> Result<T, String>,
    {
        self.recover()?;
        let mut transaction = Transaction::new(game, profile);
        transaction.save_to_path(&self.transaction_path)?;
        self.synchronizer
            .sync_with_checkpoint(desired, &mut transaction, |transaction| {
                transaction.save_to_path(&self.transaction_path)
            })?;
        let result = operation();
        let restored = self.synchronizer.restore(&transaction);
        if restored.is_ok() {
            self.remove_transaction(&transaction)?;
        }
        restored.and(result)
    }

    fn remove_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        if self.transaction_path.exists() {
            fs::remove_file(&self.transaction_path).map_err(|e| e.to_string())?;
        }
        for tracked in &transaction.tracked_files {
            if let Some(backup) = &tracked.backup_path {
                if backup.exists() {
                    fs::remove_file(backup).map_err(|e| e.to_string())?;
                }
                let mut parent = backup.parent();
                while let Some(path) = parent {
                    if path == Path::new(".") || !path.ends_with(".vespa-backup") {
                        break;
                    }
                    match fs::remove_dir(path) {
                        Ok(()) => parent = path.parent(),
                        Err(_) => break,
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::base::{runtime::file_synchronizer::DesiredFile, util::config::GameSwitcher};

    #[test]
    fn prime_then_recover_restores_the_game_and_removes_state() {
        let dir = tempdir().unwrap();
        let profile = dir.path().join("profile.dll");
        let game_dir = dir.path().join("game");
        let state = dir.path().join("runtime.json");
        fs::create_dir_all(&game_dir).unwrap();
        fs::write(&profile, b"profile").unwrap();
        fs::write(game_dir.join("mod.dll"), b"original").unwrap();

        let runtime = Runtime::new(game_dir.clone(), state.clone());
        runtime
            .prime(
                GameSwitcher::HollowKnight,
                "Default".into(),
                [DesiredFile {
                    source: profile,
                    destination: "mod.dll".into(),
                }],
            )
            .unwrap();
        assert!(state.exists());
        assert_eq!(fs::read(game_dir.join("mod.dll")).unwrap(), b"profile");

        runtime.recover().unwrap();
        assert!(!state.exists());
        assert_eq!(fs::read(game_dir.join("mod.dll")).unwrap(), b"original");
    }
}
