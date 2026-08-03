use std::collections::HashMap;

use thiserror::Error;

use crate::base::packages::package::PackageRecord;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("fetcher error: {0}")]
    FetcherError(String),
}

#[derive(Debug, Default)]
pub struct Index {
    pub packages_by_name: HashMap<String, Vec<PackageRecord>>,
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace(&mut self, packages: impl IntoIterator<Item = PackageRecord>) {
        self.packages_by_name.clear();

        for package in packages {
            self.packages_by_name
                .entry(package.name.clone())
                .or_default()
                .push(package);
        }
    }

    pub fn insert(&mut self, package: PackageRecord) {
        self.packages_by_name
            .entry(package.name.clone())
            .or_default()
            .push(package);
    }

    pub fn get_package_by_identifier(&self, identifier: &str) -> Option<PackageRecord> {
        self.packages_by_name
            .values()
            .flatten()
            .find(|package| package.identifier == identifier)
            .cloned()
    }

    pub fn get_latest_package_by_package_name(
        &self,
        package_name: &str,
    ) -> Option<PackageRecord> {
        self.packages_by_name
            .get(package_name)
            .and_then(|versions| versions.first())
            .cloned()
    }

    pub fn get_versions_by_name(&self, name: &str) -> Option<Vec<PackageRecord>> {
        self.packages_by_name.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::Index;
    use crate::base::packages::package::PackageRecord;

    fn package(name: &str, version: &str) -> PackageRecord {
        PackageRecord {
            name: name.to_string(),
            identifier: format!("{name}-{version}"),
            version: version.to_string(),
            description: String::new(),
            download_url: String::new(),
            dependencies: vec![],
        }
    }

    #[test]
    fn groups_versions_and_keeps_first_as_latest() {
        let mut index = Index::new();
        index.replace([
            package("Author-Mod", "2.0.0"),
            package("Author-Mod", "1.0.0"),
        ]);

        assert_eq!(
            index
                .get_latest_package_by_package_name("Author-Mod")
                .unwrap()
                .version,
            "2.0.0"
        );
        assert_eq!(index.get_versions_by_name("Author-Mod").unwrap().len(), 2);
        assert!(
            index
                .get_package_by_identifier("Author-Mod-1.0.0")
                .is_some()
        );
    }
}
