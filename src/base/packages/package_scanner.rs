use std::collections::HashMap;

use crate::base::packages::package::InstalledPackageRecord;

pub trait PackageScanner {
    fn scan_plugins(
        &self,
        profile_path: &std::path::Path,
    ) -> HashMap<String, InstalledPackageRecord>;
}
