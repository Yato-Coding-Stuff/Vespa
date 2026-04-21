use std::{collections::HashMap, fs, path::Path};

use crate::packages::{SilkSongFlattenedPackage, SilkSongInstalledPackageRecord};

#[derive(Debug, Default)]
pub struct SilkSongPackageTracker {
    packages: HashMap<String, SilkSongInstalledPackageRecord>,
}

impl SilkSongPackageTracker {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    pub fn scan_plugins(&mut self, profile_path: &Path) {
        let plugins_path = profile_path.join("BepInEx").join("plugins");

        if let Ok(entries) = fs::read_dir(plugins_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path.file_name().unwrap().to_string_lossy();
                    // Parse name and version from folder_name
                    let (name, version) = if let Some(pos) = folder_name.rfind('-') {
                        (&folder_name[..pos], Some(&folder_name[pos + 1..]))
                    } else {
                        (&folder_name[..], None)
                    };
                    let record = SilkSongInstalledPackageRecord {
                        package_full_name_with_version: folder_name.to_string(),
                        package_full_name: name.to_string(),
                        version_number: version.map(|v| v.to_string()),
                        file_path: path.clone(),
                    };
                    self.packages.insert(name.to_string(), record);
                }
            }
        }
    }

    pub fn add(&mut self, package: &SilkSongFlattenedPackage, file_path: &Path) {
        let record = SilkSongInstalledPackageRecord {
            package_full_name_with_version: package.package_full_name_with_version.clone(),
            package_full_name: package.package_full_name.clone(),
            version_number: Some(package.version_number.clone()),
            file_path: file_path.to_path_buf(),
        };
        self.packages
            .insert(package.package_full_name.clone(), record);
    }

    pub fn remove(&mut self, package_full_name: &str) {
        self.packages.remove(package_full_name);
    }

    pub fn get(&self, package_full_name: &str) -> Option<&SilkSongInstalledPackageRecord> {
        self.packages.get(package_full_name)
    }

    pub fn get_all(&self) -> HashMap<String, SilkSongInstalledPackageRecord> {
        self.packages.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::SilkSongPackageTracker;
    use crate::packages::SilkSongFlattenedPackage;
    use std::fs;
    use tempfile::tempdir;

    fn package(name: &str, version: &str) -> SilkSongFlattenedPackage {
        SilkSongFlattenedPackage {
            package_full_name: name.to_string(),
            owner: "Author".to_string(),
            package_full_name_with_version: format!("{name}-{version}"),
            description: "desc".to_string(),
            download_url: "https://example.test/mod.zip".to_string(),
            version_number: version.to_string(),
            dependencies: vec![],
        }
    }

    #[test]
    fn add_get_and_remove_track_package_records() {
        let temp_dir = tempdir().unwrap();
        let mod_path = temp_dir.path().join("Author-Mod-1.0.0");
        let package = package("Author-Mod", "1.0.0");
        let mut tracker = SilkSongPackageTracker::new();

        tracker.add(&package, &mod_path);

        let record = tracker.get("Author-Mod").unwrap();
        assert_eq!(record.package_full_name_with_version, "Author-Mod-1.0.0");
        assert_eq!(record.version_number.as_deref(), Some("1.0.0"));

        tracker.remove("Author-Mod");
        assert!(tracker.get("Author-Mod").is_none());
    }

    #[test]
    fn scan_plugins_discovers_installed_packages_from_profile() {
        let temp_dir = tempdir().unwrap();
        let plugins_dir = temp_dir.path().join("BepInEx").join("plugins");
        fs::create_dir_all(plugins_dir.join("Author-Mod-1.2.3")).unwrap();
        fs::create_dir_all(plugins_dir.join("NoVersionMod")).unwrap();

        let mut tracker = SilkSongPackageTracker::new();
        tracker.scan_plugins(temp_dir.path());

        let versioned = tracker.get("Author-Mod").unwrap();
        let unversioned = tracker.get("NoVersionMod").unwrap();

        assert_eq!(versioned.version_number.as_deref(), Some("1.2.3"));
        assert_eq!(unversioned.version_number, None);
        assert_eq!(tracker.get_all().len(), 2);
    }
}
