use serde::Deserialize;

use crate::base::packages::PackageRecord;

#[derive(Debug, Deserialize, Clone)]
pub(super) struct HkPackageDto {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "Version")]
    pub version: String,

    #[serde(rename = "Link", default, deserialize_with = "deserialize_trimmed")]
    pub link: String,

    #[serde(rename = "Dependencies")]
    pub dependencies: HkDependenciesDto,
}

// custom deserializer for link trimming
fn deserialize_trimmed<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.trim().to_string())
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct HkDependenciesDto {
    #[serde(rename = "Dependency", default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ModLinksDto {
    #[serde(rename = "Manifest")]
    pub packages: Vec<HkPackageDto>,
}

impl From<HkPackageDto> for PackageRecord {
    fn from(package: HkPackageDto) -> Self {
        let identifier = format!("{}-{}", package.name, package.version);

        Self {
            name: package.name,
            identifier,
            version: package.version,
            description: package.description,
            download_url: package.link,
            dependencies: package.dependencies.dependencies,
        }
    }
}
