use thiserror::Error;

use crate::packages::sk_package::{SilkSongFlattenedPackage, SilkSongPackage};

#[derive(Debug, Error)]
pub(crate) enum SilkSongFetcherError {
    #[error("reqwest error: {0}")]
    SilkSongFetcherError(String),
    #[error("Parsing error: {0}")]
    JsonError(String),
}

pub(crate) struct SilkSongPackageFetcher;

impl SilkSongPackageFetcher {
    const URL: &'static str = "https://thunderstore.io/c/hollow-knight-silksong/api/v1/package/";

    /// Fetch packages and immediately flatten them into SilkSongFlattenedPackage
    pub fn fetch() -> Result<Vec<SilkSongPackage>, SilkSongFetcherError> {
        let resp = reqwest::blocking::get(Self::URL)
            .map_err(|e| SilkSongFetcherError::SilkSongFetcherError(e.to_string()))?;

        let packages: Vec<SilkSongPackage> = resp
            .json()
            .map_err(|e| SilkSongFetcherError::JsonError(e.to_string()))?;


        Ok(packages)
    }
}
