use thiserror::Error;

use crate::packages::package::SilkSongPackage;

#[derive(Debug, Error)]
pub enum FetcherError {
    #[error("reqwest error: {0}")]
    SilkSongFetcherError(String),
    #[error("Parsing error: {0}")]
    JsonError(String),
}

pub struct SilkSongFetcher;

impl SilkSongFetcher {
    const URL: &'static str = "https://thunderstore.io/c/hollow-knight-silksong/api/v1/package/";

    pub fn new() -> Self {
        Self
    }

    pub fn fetch(&self) -> Result<Vec<SilkSongPackage>, FetcherError> {
        let resp = reqwest::blocking::get(Self::URL)
            .map_err(|e| FetcherError::SilkSongFetcherError(e.to_string()))?;

        let packages: Vec<SilkSongPackage> = resp
            .json()
            .map_err(|e| FetcherError::JsonError(e.to_string()))?;

        Ok(packages)
    }
}
