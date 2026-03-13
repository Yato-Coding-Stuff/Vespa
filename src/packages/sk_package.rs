use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/*
 * Thunderstore's API sucks, so we're forced to do it like this.
 * */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SilkSongPackage {
    pub name: String,
    pub owner: String,
    pub package_url: String,
    pub versions: Vec<SilkSongVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SilkSongVersion {
    pub full_name: String,
    pub description: String,
    pub download_url: String,
    pub version_number: String,
    pub dependencies: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silksong_indexer_builds_maps_correctly() {
        // Create fake versions
        let version1 = SilkSongVersion {
            full_name: "TestPackage-1.0.0".to_string(),
            description: "First version".to_string(),
            download_url: "https://example.com/package1.zip".to_string(),
            version_number: "1.0.0".to_string(),
            dependencies: vec!["dep1".to_string()],
        };

        let version2 = SilkSongVersion {
            full_name: "TestPackage-1.1.0".to_string(),
            description: "Second version".to_string(),
            download_url: "https://example.com/package2.zip".to_string(),
            version_number: "1.1.0".to_string(),
            dependencies: vec![],
        };

        // Create fake package
        let package = SilkSongPackage {
            name: "TestPackage".to_string(),
            owner: "TestOwner".to_string(),
            package_url: "https://example.com/package".to_string(),
            versions: vec![version1.clone(), version2.clone()],
        };

        // Build the index
        let index = SilkSongIndex::new(vec![package.clone()]);

        // Assert packages_by_name
        assert_eq!(index.packages_by_name.len(), 1);
        let pkg = index
            .packages_by_name
            .get("TestPackage")
            .expect("Package missing");
        assert_eq!(pkg.name, "TestPackage");
        assert_eq!(pkg.owner, "TestOwner");
        assert_eq!(pkg.versions.len(), 2);

        // Assert versions_by_full_name
        assert_eq!(index.versions_by_full_name.len(), 2);
        let v1 = index
            .versions_by_full_name
            .get("TestPackage-1.0.0")
            .expect("Version missing");
        let v2 = index
            .versions_by_full_name
            .get("TestPackage-1.1.0")
            .expect("Version missing");

        assert_eq!(v1.download_url, "https://example.com/package1.zip");
        assert_eq!(v2.download_url, "https://example.com/package2.zip");

        // Optional: check dependencies
        assert_eq!(v1.dependencies, vec!["dep1"]);
        assert!(v2.dependencies.is_empty());
    }
}
