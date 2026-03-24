use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::packages::sk_package_fetcher::{SilkSongFetcherError, SilkSongPackageFetcher};

/*
 * Thunderstore's API sucks, so we're forced to do it like this.
 * */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct SilkSongPackage {
    pub(super) full_name: String,
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
    pub package_full_name_with_version: String,
    pub package_full_name: String,
    pub version_number: Option<String>,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SilkSongFlattenedPackage {
    pub package_full_name: String,
    pub owner: String,
    pub package_full_name_with_version: String,
    pub description: String,
    pub download_url: String,
    pub version_number: String,
    pub dependencies: Vec<String>,
}

pub fn split_package_name_with_version(package_name_with_version: &str) -> (&str, &str) {
    if let Some((name, version)) = package_name_with_version.rsplit_once('-') {
        (name, version)
    } else {
        (package_name_with_version, "0.0.0")
    }
}

#[derive(Debug, Error)]
pub enum SilkSongIndexError {
    #[error("fetcher error: {0}")]
    SilkSongFetcherError(#[from] SilkSongFetcherError),
}

#[derive(Debug)]
pub struct SilkSongIndex {
    pub packages_by_full_name: HashMap<String, SilkSongFlattenedPackage>,
    pub latest_full_name_by_package_name: HashMap<String, String>,
}

impl SilkSongIndex {
    pub fn new(blacklist: &[&str]) -> Result<Self, SilkSongIndexError> {
        let packages = SilkSongPackageFetcher::fetch()?;

        let mut packages_by_full_name = HashMap::new();
        let mut latest_full_name_by_package_name = HashMap::new();

        let blacklist: HashSet<&str> = blacklist.iter().copied().collect();

        for package in packages {
            for (i, ver) in package.versions.into_iter().enumerate() {
                let filtered_deps = ver.dependencies.into_iter().filter(|dep| {
                    let (base_name, _) = split_package_name_with_version(dep);
                    !blacklist.contains(base_name)
                });

                let flattened = SilkSongFlattenedPackage {
                    package_full_name: package.full_name.clone(),
                    owner: package.owner.clone(),
                    package_full_name_with_version: ver.full_name.clone(),
                    description: ver.description,
                    download_url: ver.download_url,
                    version_number: ver.version_number,
                    dependencies: filtered_deps.collect(),
                };

                packages_by_full_name.insert(ver.full_name.clone(), flattened.clone());
                if i == 0 {
                    latest_full_name_by_package_name
                        .insert(package.full_name.clone(), ver.full_name.clone());
                }
            }
        }

        Ok(Self {
            packages_by_full_name,
            latest_full_name_by_package_name,
        })
    }

    pub fn get_package_by_full_name_with_version(&self, full_name_with_version: &str) -> Option<SilkSongFlattenedPackage> {
        self.packages_by_full_name.get(full_name_with_version).cloned()
    }

    pub fn get_latest_package_by_package_name(
        &self,
        package_name: &str,
    ) -> Option<SilkSongFlattenedPackage> {
        self.latest_full_name_by_package_name
            .get(package_name)
            .and_then(|full_name| self.packages_by_full_name.get(full_name))
            .cloned()
    }
}
