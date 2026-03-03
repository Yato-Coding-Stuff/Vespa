use std::collections::HashMap;

use serde::Deserialize;

/*
 * Thunderstore's API sucks, so we're forced to do it like this.
 * */

#[derive(Debug, Deserialize)]
pub struct SilkSongPackage {
    name: String,
    owner: String,
    package_url: String,
    versions: Vec<SilkSongVersion>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SilkSongVersion {
    full_name: String,
    description: String,
    download_url: String,
    version_number: String,
    dependencies: Vec<String>,
}

#[derive(Debug)]
pub struct SilkSongIndex {
    pub packages_by_name: HashMap<String, SilkSongPackage>,
    pub versions_by_full_name: HashMap<String, SilkSongVersion>,
}

impl SilkSongIndex {
    pub fn new(packages: Vec<SilkSongPackage>) -> Self {
        let mut packages_by_name = HashMap::new();
        let mut versions_by_full_name = HashMap::new();

        for package in packages {
            for version in &package.versions {
                versions_by_full_name.insert(version.full_name.clone(), version.clone());
            }
            packages_by_name.insert(package.name.clone(), package);
        }

        Self {
            packages_by_name,
            versions_by_full_name,
        }
    }
}
