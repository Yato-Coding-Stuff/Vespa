use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::packages::SilkSongInstalledPackageRecord;


#[derive(Debug, Error)]
pub enum SilkSongPackageTrackerError {
    #[error("Error handling json: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to write to file: {0}")]
    WriteError(#[from] std::io::Error),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SilkSongPackageTracker {
    pub packages: HashMap<String, SilkSongInstalledPackageRecord>,
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

    pub fn add_installed_package_record(
        &mut self,
        package_name: String,
        package: &SilkSongInstalledPackageRecord,
    ) {
        self.packages.insert(package_name, package.clone());
    }

    pub fn get_installed_package_record(
        &self,
        package_name: &str,
    ) -> Option<&SilkSongInstalledPackageRecord> {
        self.packages.get(package_name)
    }

    pub fn is_installed(&self, package_name: &str) -> bool {
        self.packages.contains_key(package_name)
    }
}
