use std::collections::HashMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::packages::sk_package_fetcher::{SilkSongFetcherError, SilkSongPackageFetcher};

/*
 * Thunderstore's API sucks, so we're forced to do it like this.
 * */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct SilkSongPackage {
    pub(super) name: String,
    pub(super) owner: String,
    pub(super) package_url: String,
    pub(super) versions: Vec<SilkSongVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct SilkSongVersion {
    pub(super) full_name: String,
    pub(super) description: String,
    pub(super) download_url: String,
    pub(super) version_number: String,
    pub(super) dependencies: Vec<String>,
}

// TODO:
// simplify tracker to use file-based source-of-truth
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SilkSongInstalledPackageRecord {
    pub version_full_name: String,
    pub version_number: Version,
}

#[derive(Debug, Clone)]
pub struct SilkSongFlattenedPackage {
    pub package_name: String,
    pub owner: String,
    pub package_name_with_version: String,
    pub description: String,
    pub download_url: String,
    pub version_number: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Error)]
pub enum SilkSongIndexError {
    #[error("fetcher error: {0}")]
    SilkSongFetcherError(#[from] SilkSongFetcherError),
}

#[derive(Debug)]
pub struct SilkSongIndex {
    pub packages_by_full_name: HashMap<String, SilkSongFlattenedPackage>,
    pub full_name_by_package_name: HashMap<String, String>,
}

impl SilkSongIndex {
    pub fn new() -> Result<Self, SilkSongIndexError> {
        let flattened_packages = SilkSongPackageFetcher::fetch()?;

        let mut packages_by_full_name = HashMap::new();
        let mut full_name_by_package_name = HashMap::new();

        for package in flattened_packages {
            packages_by_full_name
                .insert(package.package_name_with_version.clone(), package.clone());

            full_name_by_package_name.insert(
                package.package_name.clone(),
                package.package_name_with_version.clone(),
            );
        }

        Ok(Self {
            packages_by_full_name,
            full_name_by_package_name,
        })
    }

    pub fn get_package_by_full_name(&self, full_name: &str) -> Option<SilkSongFlattenedPackage> {
        self.packages_by_full_name.get(full_name).cloned()
    }

    pub fn get_full_name_by_package_name(&self, package_name: &str) -> Option<String> {
        self.full_name_by_package_name.get(package_name).cloned()
    }
}
