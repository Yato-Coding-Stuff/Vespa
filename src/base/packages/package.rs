use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageRecord {
    pub name: String,
    pub identifier: String,
    pub version: String,
    pub description: String,
    pub download_url: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledPackageRecord {
    pub name: String,
    pub identifier: String,
    pub version: Option<String>,
    pub file_path: PathBuf,
}
