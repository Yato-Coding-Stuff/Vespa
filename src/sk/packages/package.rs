use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::base::packages::PackageRecord;

/*
 * Thunderstore's API sucks, so we're forced to do it like this.
 * */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct SkPackageDto {
    pub(super) full_name: String,
    pub(super) owner: String,
    pub(super) package_url: String,
    pub(super) versions: Vec<VersionDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct VersionDto {
    pub(super) full_name: String,
    pub(super) description: String,
    pub(super) download_url: String,
    pub(super) version_number: String,
    pub(super) dependencies: Vec<String>,
}

pub fn split_package_name_with_version(package_name_with_version: &str) -> (&str, &str) {
    if let Some((name, version)) = package_name_with_version.rsplit_once('-') {
        (name, version)
    } else {
        (package_name_with_version, "0.0.0")
    }
}

pub(super) fn flatten_packages(
    packages: Vec<SkPackageDto>,
    blacklist: &[&str],
) -> Vec<PackageRecord> {
    let blacklist: HashSet<&str> = blacklist.iter().copied().collect();

    packages
        .into_iter()
        .flat_map(|package| {
            let package_name = package.full_name;
            let blacklist = &blacklist;

            package.versions.into_iter().map(move |version| {
                let dependencies = version
                    .dependencies
                    .into_iter()
                    .filter(|dependency| {
                        let (dependency_name, _) =
                            split_package_name_with_version(dependency);
                        !blacklist.contains(dependency_name)
                    })
                    .collect();

                PackageRecord {
                    name: package_name.clone(),
                    identifier: version.full_name,
                    version: version.version_number,
                    description: version.description,
                    download_url: version.download_url,
                    dependencies,
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        SkPackageDto, VersionDto, flatten_packages, split_package_name_with_version,
    };
    use crate::base::packages::Index;

    fn package(
        full_name: &str,
        owner: &str,
        versions: Vec<(&str, &str, &str, Vec<&str>)>,
    ) -> SkPackageDto {
        SkPackageDto {
            full_name: full_name.to_string(),
            owner: owner.to_string(),
            package_url: "https://example.test".to_string(),
            versions: versions
                .into_iter()
                .map(
                    |(full_name, version_number, description, dependencies)| VersionDto {
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
    fn flatten_packages_builds_records_and_filters_blacklist() {
        let mut index = Index::new();
        index.replace(flatten_packages(
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
        ));

        let latest = index
            .get_latest_package_by_package_name("Author-Mod")
            .unwrap();
        let all_versions = index.get_versions_by_name("Author-Mod").unwrap();

        assert_eq!(latest.identifier, "Author-Mod-2.0.0");
        assert_eq!(latest.dependencies, vec!["Keep-Dep-1.0.0".to_string()]);
        assert_eq!(all_versions.len(), 2);
        assert!(
            index
                .get_package_by_identifier("Author-Mod-1.0.0")
                .is_some()
        );
    }
}
