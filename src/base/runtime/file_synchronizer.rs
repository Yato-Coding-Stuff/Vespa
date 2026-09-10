use std::{
    collections::HashSet,
    fs,
    path::{Component, Path, PathBuf},
};

use crate::base::runtime::transaction::{TrackedFile, Transaction};

pub struct DesiredFile {
    pub source: PathBuf,
    pub destination: PathBuf,
}

pub struct FileSynchronizer {
    game_dir: PathBuf,
}

impl FileSynchronizer {
    pub fn new(game_dir: PathBuf) -> FileSynchronizer {
        FileSynchronizer { game_dir }
    }

    pub fn sync_with_checkpoint<I, F>(
        &self,
        desired: I,
        transaction: &mut Transaction,
        mut checkpoint: F,
    ) -> Result<(), String>
    where
        I: IntoIterator<Item = DesiredFile>,
        F: FnMut(&Transaction) -> Result<(), String>,
    {
        let desired: Vec<_> = desired.into_iter().collect();

        let desired_paths: HashSet<_> = desired
            .iter()
            .map(|file| file.destination.clone())
            .collect();

        for tracked in transaction.tracked_files.iter() {
            if !desired_paths.contains(&tracked.game_path) {
                let path = self.game_path(&tracked.game_path)?;
                if path.is_file() {
                    fs::remove_file(path).map_err(|e| e.to_string())?;
                } else if path.exists() {
                    return Err(format!("game path is not a file: {}", path.display()));
                }
            }
        }

        for file in desired {
            Self::validate_source(&file.source)?;
            let game_path = self.game_path(&file.destination)?;
            if let Some(tracked) = transaction
                .tracked_files
                .iter_mut()
                .find(|tracked| tracked.game_path == file.destination)
            {
                if Self::files_equal(&file.source, &game_path)? {
                    continue;
                }
                Self::copy_file(&file.source, &game_path)?;
                tracked.profile_path = file.source;
                continue;
            }

            let existed = game_path.exists();
            if existed && !game_path.is_file() {
                return Err(format!("game path is not a file: {}", game_path.display()));
            }
            let backup_path = if existed {
                let backup = self.backup_path(transaction.tracked_files.len(), &file.destination);
                Self::copy_file(&game_path, &backup)?;
                Some(backup)
            } else {
                None
            };

            transaction.tracked_files.push(TrackedFile {
                profile_path: file.source,
                game_path: file.destination,
                backup_path,
                existed,
            });
            //
            // Persist the restore information before touching the game file.
            // If the following copy is interrupted, recovery can still undo it.
            checkpoint(transaction)?;
            Self::copy_file(
                &transaction.tracked_files.last().unwrap().profile_path,
                &game_path,
            )?;
        }

        Ok(())
    }

    pub fn restore(&self, transaction: &Transaction) -> Result<(), String> {
        for tracked in transaction.tracked_files.iter().rev() {
            let game_path = self.game_path(&tracked.game_path)?;
            if game_path.is_file() {
                fs::remove_file(&game_path).map_err(|e| e.to_string())?;
            } else if game_path.exists() {
                return Err(format!("game path is not a file: {}", game_path.display()));
            }

            if tracked.existed {
                let backup = tracked
                    .backup_path
                    .as_ref()
                    .ok_or_else(|| format!("missing backup path for {}", game_path.display()))?;
                Self::copy_file(backup, &game_path)?;
            }
        }
        Ok(())
    }

    fn game_path(&self, path: &Path) -> Result<PathBuf, String> {
        if path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(format!(
                "game path must be relative and stay within game directory: {}",
                path.display()
            ));
        }
        Ok(self.game_dir.join(path))
    }

    fn backup_path(&self, index: usize, destination: &Path) -> PathBuf {
        self.game_dir
            .join(".vespa-backup")
            .join(index.to_string())
            .join(destination)
    }

    fn validate_source(path: &Path) -> Result<(), String> {
        if !path.is_file() {
            return Err(format!("profile file does not exist: {}", path.display()));
        }
        Ok(())
    }

    fn files_equal(left: &Path, right: &Path) -> Result<bool, String> {
        if !right.is_file() {
            return Ok(false);
        }
        let left = fs::read(left).map_err(|e| e.to_string())?;
        let right = fs::read(right).map_err(|e| e.to_string())?;
        Ok(left == right)
    }

    fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
        let parent = destination
            .parent()
            .ok_or_else(|| "destination has no parent".to_string())?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        fs::copy(source, destination).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::runtime::transaction::Transaction;
    use crate::base::util::config::GameSwitcher;
    use tempfile::tempdir;

    #[test]
    fn sync_backs_up_existing_files_and_restore_returns_original_state() {
        let dir = tempdir().unwrap();
        let profile = dir.path().join("profile");
        let game = dir.path().join("game");
        fs::create_dir_all(&profile).unwrap();
        fs::create_dir_all(&game).unwrap();
        fs::write(profile.join("mod.dll"), b"profile").unwrap();
        fs::write(game.join("mod.dll"), b"original").unwrap();

        let synchronizer = FileSynchronizer::new(game.clone());
        let mut transaction = Transaction::new(GameSwitcher::HollowKnight, "Default".into());
        synchronizer
            .sync_with_checkpoint(
                [DesiredFile {
                    source: profile.join("mod.dll"),
                    destination: "mod.dll".into(),
                }],
                &mut transaction,
                |_| Ok(()),
            )
            .unwrap();
        assert_eq!(fs::read(game.join("mod.dll")).unwrap(), b"profile");
        assert!(transaction.tracked_files[0].backup_path.is_some());

        synchronizer.restore(&transaction).unwrap();
        assert_eq!(fs::read(game.join("mod.dll")).unwrap(), b"original");
    }

    #[test]
    fn sync_removes_profile_only_files_when_they_are_no_longer_desired() {
        let dir = tempdir().unwrap();
        let profile = dir.path().join("profile");
        let game = dir.path().join("game");
        fs::create_dir_all(&profile).unwrap();
        fs::create_dir_all(&game).unwrap();
        fs::write(profile.join("mod.dll"), b"profile").unwrap();
        let synchronizer = FileSynchronizer::new(game.clone());
        let mut transaction = Transaction::new(GameSwitcher::HollowKnight, "Default".into());
        synchronizer
            .sync_with_checkpoint(
                [DesiredFile {
                    source: profile.join("mod.dll"),
                    destination: "mod.dll".into(),
                }],
                &mut transaction,
                |_| Ok(()),
            )
            .unwrap();
        synchronizer
            .sync_with_checkpoint(std::iter::empty(), &mut transaction, |_| Ok(()))
            .unwrap();
        assert!(!game.join("mod.dll").exists());
        synchronizer.restore(&transaction).unwrap();
        assert!(!game.join("mod.dll").exists());
    }
}
