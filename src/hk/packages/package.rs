use semver::Version;
use serde::Deserialize;
use serde::de::Error;

use crate::base::packages::PackageRecord;

#[derive(Debug, Deserialize, Clone)]
pub(super) struct HkPackageDto {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "Version", default, deserialize_with = "deserialize_version")]
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

// custom deserializer for parsing hk's versioning to SemVer compatible formatting
fn deserialize_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;

    let parts = raw
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(D::Error::custom)?;

    let [major, minor, patch, revision] = parts.as_slice() else {
        return Err(D::Error::custom(format!(
            "expected four-part HK version, got `{raw}`"
        )));
    };

    let version =
        Version::parse(&format!("{major}.{minor}.{patch}+{revision}")).map_err(D::Error::custom)?;
    Ok(version.to_string())
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
        Self {
            name: package.name.clone(),
            identifier: package.name,
            version: package.version,
            description: package.description,
            download_url: package.link,
            dependencies: package.dependencies.dependencies,
        }
    }
}
