use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::base::packages::package::InstalledPackageRecord;

pub struct PackageScanner {
    package_root: fn(&Path) -> PathBuf,
}

impl PackageScanner {
    pub fn new(package_root: fn(&Path) -> PathBuf) -> Self {
        Self { package_root }
    }

    pub fn scan_plugins(&self, profile_path: &Path) -> HashMap<String, InstalledPackageRecord> {
        let mut packages = HashMap::new();
        let plugins_path = (self.package_root)(profile_path);

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
