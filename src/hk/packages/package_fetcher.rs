use quick_xml::de::from_str;
use thiserror::Error;

use crate::hk::packages::package::{HkPackageDto, ModLinksDto};

#[derive(Debug, Error)]
pub(crate) enum FetcherError {
    #[error("reqwest error: {0}")]
    FetcherError(String),
    #[error("Parsing error: {0}")]
    XmlError(String),
}

pub(crate) struct PackageFetcher;

impl PackageFetcher {
    const URL: &'static str =
        "https://raw.githubusercontent.com/hk-modding/modlinks/main/ModLinks.xml";

    pub fn fetch() -> Result<Vec<HkPackageDto>, FetcherError> {
        let resp = reqwest::blocking::get(Self::URL)
            .map_err(|e| FetcherError::FetcherError(e.to_string()))?
            .text()
            .map_err(|e| FetcherError::FetcherError(e.to_string()))?;

        let mod_links: ModLinksDto =
            from_str(&resp).map_err(|e| FetcherError::XmlError(e.to_string()))?;

        Ok(mod_links.packages)
    }
}
