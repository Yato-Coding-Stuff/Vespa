use std::{fs, path::Path};

use crate::base::runtime::file_synchronizer::DesiredFile;

pub struct Primer {}

impl Primer {
    pub fn files_from_profile(&self, profile: &Path) -> Result<Vec<DesiredFile>, String> {
        if !profile.is_dir() {
            return Err(format!(
                "profile directory does not exist: {}",
                profile.display()
            ));
        }
        let mut files = Vec::new();
        self.visit(profile, profile, &mut files)?;
        files.sort_by(|left, right| left.destination.cmp(&right.destination));
        Ok(files)
    }

    fn visit(
        &self,
        root: &Path,
        directory: &Path,
        files: &mut Vec<DesiredFile>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                self.visit(root, &path, files)?;
            } else if kind.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_path_buf();
                files.push(DesiredFile {
                    source: path,
                    destination: relative,
                });
            }
        }
        Ok(())
    }
}
