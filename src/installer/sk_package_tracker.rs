use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::packages::sk_package::SilkSongPackage;

#[derive(Debug, Error)]
pub enum SilkSongPackageTrackerError {
    #[error("Error handling json: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to write to file: {0}")]
    WriteError(#[from] std::io::Error),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SilkSongPackageTracker {
    pub packages: HashMap<String, SilkSongPackage>,
}

impl SilkSongPackageTracker {
    pub fn load(path: &str) -> Result<Self, SilkSongPackageTrackerError> {
        match fs::read_to_string(path) {
            Ok(serialized) => Ok(serde_json::from_str(&serialized)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(SilkSongPackageTracker::default())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self, path: &str) -> Result<(), SilkSongPackageTrackerError> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn add_package(&mut self, package: SilkSongPackage) {
        self.packages.insert(package.name.clone(), package);
    }

    pub fn get_package(&self, name: &str) -> Option<&SilkSongPackage> {
        self.packages.get(name)
    }
}
