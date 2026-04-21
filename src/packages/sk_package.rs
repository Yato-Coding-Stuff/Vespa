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
    pub all_versions_by_full_name: HashMap<String, Vec<SilkSongFlattenedPackage>>,
    pub latest_full_name_by_package_name: HashMap<String, String>,
}

impl SilkSongIndex {
    pub fn new() -> Self {
        SilkSongIndex {
            packages_by_full_name: HashMap::new(),
            all_versions_by_full_name: HashMap::new(),
            latest_full_name_by_package_name: HashMap::new(),
        }
    }

    pub fn initialize(&mut self, blacklist: &[&str]) -> Result<(), SilkSongIndexError> {
        let packages = SilkSongPackageFetcher::fetch()?;
        self.initialize_from_packages(packages, blacklist);
        Ok(())
    }

    fn initialize_from_packages(&mut self, packages: Vec<SilkSongPackage>, blacklist: &[&str]) {
        let mut packages_by_full_name = HashMap::new();
        let mut latest_full_name_by_package_name = HashMap::new();
        let mut all_versions_by_full_name: HashMap<String, Vec<SilkSongFlattenedPackage>> =
            HashMap::new();

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

                all_versions_by_full_name
                    .entry(flattened.package_full_name.clone())
                    .or_default()
                    .push(flattened.clone());
                packages_by_full_name.insert(ver.full_name.clone(), flattened.clone());
                if i == 0 {
                    latest_full_name_by_package_name
                        .insert(package.full_name.clone(), ver.full_name.clone());
                }
            }
        }

        self.packages_by_full_name = packages_by_full_name;
        self.latest_full_name_by_package_name = latest_full_name_by_package_name;
        self.all_versions_by_full_name = all_versions_by_full_name;
    }

    pub fn get_package_by_full_name_with_version(
        &self,
        full_name_with_version: &str,
    ) -> Option<SilkSongFlattenedPackage> {
        self.packages_by_full_name
            .get(full_name_with_version)
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

    pub fn get_versions_by_full_name(
        &self,
        full_name: &str,
    ) -> Option<Vec<SilkSongFlattenedPackage>> {
        self.all_versions_by_full_name.get(full_name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::{SilkSongIndex, SilkSongPackage, SilkSongVersion, split_package_name_with_version};

    fn package(
        full_name: &str,
        owner: &str,
        versions: Vec<(&str, &str, &str, Vec<&str>)>,
    ) -> SilkSongPackage {
        SilkSongPackage {
            full_name: full_name.to_string(),
            owner: owner.to_string(),
            package_url: "https://example.test".to_string(),
            versions: versions
                .into_iter()
                .map(
                    |(full_name, version_number, description, dependencies)| SilkSongVersion {
                        full_name: full_name.to_string(),
                        description: description.to_string(),
                        download_url: format!("https://example.test/{full_name}.zip"),
                        version_number: version_number.to_string(),
                        dependencies: dependencies.into_iter().map(str::to_string).collect(),
                    },
                )
                .collect(),
        }
    }

    #[test]
    fn split_package_name_with_version_splits_last_dash_only() {
        assert_eq!(
            split_package_name_with_version("Author-MyMod-1.2.3"),
            ("Author-MyMod", "1.2.3")
        );
        assert_eq!(
            split_package_name_with_version("NoVersion"),
            ("NoVersion", "0.0.0")
        );
    }

    #[test]
    fn initialize_from_packages_builds_all_indexes_and_filters_blacklist() {
        let mut index = SilkSongIndex::new();
        index.initialize_from_packages(
            vec![package(
                "Author-Mod",
                "Author",
                vec![
                    (
                        "Author-Mod-2.0.0",
                        "2.0.0",
                        "latest",
                        vec!["Keep-Dep-1.0.0", "Blocked-Dep-3.0.0"],
                    ),
                    ("Author-Mod-1.0.0", "1.0.0", "old", vec![]),
                ],
            )],
            &["Blocked-Dep"],
        );

        let latest = index
            .get_latest_package_by_package_name("Author-Mod")
            .unwrap();
        let all_versions = index.get_versions_by_full_name("Author-Mod").unwrap();

        assert_eq!(latest.package_full_name_with_version, "Author-Mod-2.0.0");
        assert_eq!(latest.dependencies, vec!["Keep-Dep-1.0.0".to_string()]);
        assert_eq!(all_versions.len(), 2);
        assert!(
            index
                .get_package_by_full_name_with_version("Author-Mod-1.0.0")
                .is_some()
        );
    }
}
