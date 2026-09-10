use std::{collections::HashMap, fs, path::Path};

use crate::base::packages::{package::InstalledPackageRecord, package_scanner::PackageScanner};

use super::package_layout::package_root;

#[derive(Debug, Default)]
pub struct SkPackageScanner;

impl PackageScanner for SkPackageScanner {
    fn scan_plugins(&self, profile_path: &Path) -> HashMap<String, InstalledPackageRecord> {
        let mut packages = HashMap::new();
        let plugins_path = package_root(profile_path);

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
                    let record = InstalledPackageRecord {
                        identifier: folder_name.to_string(),
                        name: name.to_string(),
                        version: version.map(str::to_string),
                        file_path: path.clone(),
                    };
                    packages.insert(name.to_string(), record);
                }
            }
        }
        packages
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use tempfile::tempdir;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::{SkPackageScanner, package_root};
    use crate::base::{
        manager::package_manager::PackageManager, packages::package_scanner::PackageScanner,
        tracker::package_tracker::PackageTracker,
    };

    #[test]
    fn discovers_installed_packages_from_profile() {
        let temp_dir = tempdir().unwrap();
        let plugins_dir = temp_dir.path().join("BepInEx").join("plugins");
        fs::create_dir_all(plugins_dir.join("Author-Mod-1.2.3")).unwrap();
        fs::create_dir_all(plugins_dir.join("NoVersionMod")).unwrap();

        let mut tracker = PackageTracker::new();
        tracker.replace(SkPackageScanner.scan_plugins(temp_dir.path()));

        let versioned = tracker.get("Author-Mod").unwrap();
        let unversioned = tracker.get("NoVersionMod").unwrap();

        assert_eq!(versioned.version.as_deref(), Some("1.2.3"));
        assert_eq!(unversioned.version, None);
        assert_eq!(tracker.get_all().len(), 2);
    }

    #[test]
    fn local_install_is_discovered_after_rebuilding_tracker() {
        let temp_dir = tempdir().unwrap();
        let archive_path = temp_dir.path().join("Author-Mod-1.2.3.zip");
        let archive_file = fs::File::create(&archive_path).unwrap();
        let mut archive = ZipWriter::new(archive_file);
        archive
            .start_file("mod.dll", SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"binary").unwrap();
        archive.finish().unwrap();

        let profile_path = temp_dir.path().join("profile");
        let manager = PackageManager::new(&[], package_root);
        manager
            .install_local_package(&archive_path, &profile_path)
            .unwrap();

        let mut tracker = PackageTracker::new();
        tracker.replace(SkPackageScanner.scan_plugins(&profile_path));

        let installed = tracker.get("Author-Mod").unwrap();
        assert_eq!(installed.identifier, "Author-Mod-1.2.3");
        assert_eq!(installed.version.as_deref(), Some("1.2.3"));
        assert_eq!(
            installed.file_path,
            package_root(&profile_path).join("Author-Mod-1.2.3")
        );
    }
}
