use thiserror::Error;

use crate::packages::sk_package::SilkSongPackage;

#[derive(Debug, Error)]
pub enum SilkSongFetcherError {
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

    pub fn fetch(&self) -> Result<Vec<SilkSongPackage>, SilkSongFetcherError> {
        let resp = reqwest::blocking::get(Self::URL)
            .map_err(|e| SilkSongFetcherError::SilkSongFetcherError(e.to_string()))?;

        let packages: Vec<SilkSongPackage> = resp
            .json()
            .map_err(|e| SilkSongFetcherError::JsonError(e.to_string()))?;

        Ok(packages)
    }
}
