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
    pub latest_full_name_by_package_name: HashMap<String, String>,
}

impl SilkSongIndex {
    pub fn new() -> Result<Self, SilkSongIndexError> {
        let packages = SilkSongPackageFetcher::fetch()?;

        let mut packages_by_full_name = HashMap::new();
        let mut latest_full_name_by_package_name = HashMap::new();

        for package in packages {
            for (i, ver) in package.versions.into_iter().enumerate() {
                let flattened = SilkSongFlattenedPackage {
                    package_name: package.name.clone(),
                    owner: package.owner.clone(),
                    package_name_with_version: ver.full_name.clone(),
                    description: ver.description,
                    download_url: ver.download_url,
                    version_number: ver.version_number,
                    dependencies: ver.dependencies,
                };

                packages_by_full_name.insert(ver.full_name.clone(), flattened.clone());
                if i == 0 {
                    latest_full_name_by_package_name
                        .insert(package.name.clone(), ver.full_name.clone());
                }
            }
        }

        Ok(Self {
            packages_by_full_name,
            latest_full_name_by_package_name,
        })
    }

    pub fn get_package_by_full_name(&self, full_name: &str) -> Option<SilkSongFlattenedPackage> {
        self.packages_by_full_name.get(full_name).cloned()
    }

    pub fn get_latest_full_name_by_package_name(&self, package_name: &str) -> Option<String> {
        self.latest_full_name_by_package_name
            .get(package_name)
            .cloned()
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
