use std::collections::HashMap;

use crate::base::packages::package::InstalledPackageRecord;

#[derive(Debug, Default)]
pub struct PackageTracker {
    packages: HashMap<String, InstalledPackageRecord>,
}

impl PackageTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace(&mut self, packages: HashMap<String, InstalledPackageRecord>) {
        self.packages = packages;
    }

    pub fn insert(&mut self, package: InstalledPackageRecord) {
        self.packages.insert(package.name.clone(), package);
    }

    pub fn remove(&mut self, package_full_name: &str) {
        self.packages.remove(package_full_name);
    }

    pub fn get(&self, package_name: &str) -> Option<&InstalledPackageRecord> {
        self.packages.get(package_name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &InstalledPackageRecord)> {
        self.packages.iter()
    }

    pub fn get_all(&self) -> HashMap<String, InstalledPackageRecord> {
        self.packages.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::PackageTracker;
    use crate::base::packages::InstalledPackageRecord;

    #[test]
    fn insert_get_and_remove_track_package_records() {
        let mut tracker = PackageTracker::new();
        tracker.insert(InstalledPackageRecord {
            name: "Author-Mod".to_string(),
            identifier: "Author-Mod-1.0.0".to_string(),
            version: Some("1.0.0".to_string()),
            file_path: "/mods/Author-Mod-1.0.0".into(),
        });

        let record = tracker.get("Author-Mod").unwrap();
        assert_eq!(record.identifier, "Author-Mod-1.0.0");
        assert_eq!(record.version.as_deref(), Some("1.0.0"));

        tracker.remove("Author-Mod");
        assert!(tracker.get("Author-Mod").is_none());
    }
}
