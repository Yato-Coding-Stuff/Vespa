use std::{collections::HashMap, fs, io, path::Path};

use crate::base::{
    manager::package_layout::PackageLayout, packages::package::InstalledPackageRecord,
};

pub struct PackageScanner {
    package_layout: Box<dyn PackageLayout>,
}

impl PackageScanner {
    pub fn new(package_layout: Box<dyn PackageLayout>) -> Self {
        Self { package_layout }
    }

    pub fn scan_plugins(
        &self,
        profile_path: &Path,
    ) -> Result<HashMap<String, InstalledPackageRecord>, io::Error> {
        let mut packages = HashMap::new();
        let plugins_path = self.package_layout.package_root(profile_path);

        let entries = match fs::read_dir(plugins_path) {
            Ok(entries) => Some(entries),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };

        if let Some(entries) = entries {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if entry.file_type()?.is_dir() {
                    let folder_name = path.file_name().unwrap().to_string_lossy();
                    let marker_version = fs::read_to_string(path.join(".vespa-version"))
                        .ok()
                        .map(|version| version.trim().to_string())
                        .filter(|version| !version.is_empty());
                    // The marker is the only source of truth for versions. The
                    // directory name is used for the package name only, with
                    // the legacy `name-version` shape retained when it agrees
                    // with the marker.
                    let name = match marker_version.as_deref() {
                        Some(version) => match folder_name.rsplit_once('-') {
                            Some((name, suffix)) if suffix == version => name.to_string(),
                            _ => folder_name.to_string(),
                        },
                        None => folder_name.to_string(),
                    };
                    let record = InstalledPackageRecord {
                        identifier: folder_name.to_string(),
                        name: name.clone(),
                        version: marker_version,
                        file_path: path.clone(),
                    };
                    packages.insert(name, record);
                }
            }
        }

        for package in self.package_layout.scan_special_packages(profile_path) {
            packages.insert(package.name.clone(), package);
        }
        Ok(packages)
    }
}

#[cfg(test)]
mod tests {
    use super::PackageScanner;
    use crate::hk::manager::package_layout::HkPackageLayout;
    use std::fs;

    #[test]
    fn reads_version_marker_for_packages_without_versioned_directory_names() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package_dir = temp_dir.path().join("mods").join("DebugMod");
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(package_dir.join(".vespa-version"), "1.4.10+2\n").unwrap();

        let scanner = PackageScanner::new(Box::new(HkPackageLayout));
        let packages = scanner.scan_plugins(temp_dir.path()).unwrap();

        assert_eq!(
            packages.get("DebugMod").unwrap().version.as_deref(),
            Some("1.4.10+2")
        );
    }

    #[test]
    fn does_not_split_hyphenated_package_name_when_marker_is_present() {
        let temp_dir = tempfile::tempdir().unwrap();
        let package_dir = temp_dir.path().join("mods").join("Debug-Mod");
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(package_dir.join(".vespa-version"), "1.2.3\n").unwrap();

        let scanner = PackageScanner::new(Box::new(HkPackageLayout));
        let packages = scanner.scan_plugins(temp_dir.path()).unwrap();

        let package = packages.get("Debug-Mod").unwrap();
        assert_eq!(package.name, "Debug-Mod");
        assert_eq!(package.version.as_deref(), Some("1.2.3"));
    }
}
