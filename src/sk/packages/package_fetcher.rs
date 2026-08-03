use thiserror::Error;

use crate::sk::packages::package::SkPackageDto;

#[derive(Debug, Error)]
pub(crate) enum FetcherError {
    #[error("reqwest error: {0}")]
    FetcherError(String),
    #[error("Parsing error: {0}")]
    JsonError(String),
}

pub(crate) struct PackageFetcher;

impl PackageFetcher {
    const URL: &'static str = "https://thunderstore.io/c/hollow-knight-silksong/api/v1/package/";

    /// Fetch packages and immediately flatten them into PackageRecord
    pub fn fetch() -> Result<Vec<SkPackageDto>, FetcherError> {
        let resp = reqwest::blocking::get(Self::URL)
            .map_err(|e| FetcherError::FetcherError(e.to_string()))?;

        let packages: Vec<SkPackageDto> = resp
            .json()
            .map_err(|e| FetcherError::JsonError(e.to_string()))?;

        Ok(packages)
    }
}
