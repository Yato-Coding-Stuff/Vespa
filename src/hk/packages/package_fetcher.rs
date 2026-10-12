use quick_xml::de::from_str;
use thiserror::Error;

use crate::{
    base::util::platform::OperatingSystem, hk::packages::package::{ApiLinksDto, HkDependenciesDto, HkPackageDto, ModLinksDto}
};

#[derive(Debug, Error)]
pub(crate) enum FetcherError {
    #[error("reqwest error: {0}")]
    FetcherError(String),
    #[error("Parsing error: {0}")]
    XmlError(String),
    #[error("unsupported operating system")]
    UnsupportedOperatingSystem,
}

pub(crate) struct PackageFetcher;

impl PackageFetcher {
    const MOD_URL: &'static str =
        "https://raw.githubusercontent.com/hk-modding/modlinks/main/ModLinks.xml";

    const API_URL: &'static str =
        "https://raw.githubusercontent.com/hk-modding/modlinks/main/ApiLinks.xml";

    pub(super) fn fetch() -> Result<Vec<HkPackageDto>, FetcherError> {
        let mut packages = Self::fetch_mods()?;
        packages.push(Self::fetch_api()?);
        Ok(packages)
    }

    fn fetch_mods() -> Result<Vec<HkPackageDto>, FetcherError> {
        let resp = reqwest::blocking::get(Self::MOD_URL)
            .map_err(|e| FetcherError::FetcherError(e.to_string()))?
            .text()
            .map_err(|e| FetcherError::FetcherError(e.to_string()))?;

        let mod_links: ModLinksDto =
            from_str(&resp).map_err(|e| FetcherError::XmlError(e.to_string()))?;

        Ok(mod_links.packages)
    }

    // yes, this is hacky...
    // no, i dont care...
    fn fetch_api() -> Result<HkPackageDto, FetcherError> {
        let resp = reqwest::blocking::get(Self::API_URL)
            .map_err(|e| FetcherError::FetcherError(e.to_string()))?
            .text()
            .map_err(|e| FetcherError::FetcherError(e.to_string()))?;

        let api_links: ApiLinksDto =
            from_str(&resp).map_err(|e| FetcherError::XmlError(e.to_string()))?;

        let link = match OperatingSystem::current() {
            OperatingSystem::Linux => api_links.manifest.links.linux.url,
            OperatingSystem::MacOS => api_links.manifest.links.mac.url,
            OperatingSystem::Windows => api_links.manifest.links.windows.url,
            OperatingSystem::Unsupported => return Err(FetcherError::UnsupportedOperatingSystem),
        };

        Ok(HkPackageDto {
            name: "HollowKnight.ModdingApi".to_string(),
            description: "Hollow Knight Modding API".to_string(),
            version: format!("{}.0.0", api_links.manifest.version),
            link,
            dependencies: HkDependenciesDto {
                dependencies: Vec::new(),
            },
        })
    }
}
