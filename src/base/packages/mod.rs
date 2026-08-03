pub mod package;
pub mod package_index;
pub mod package_scanner;

pub use package::{InstalledPackageRecord, PackageRecord};
pub use package_index::{Index, IndexError};

pub type PackageLoader = fn(&[&str]) -> Result<Vec<PackageRecord>, IndexError>;
