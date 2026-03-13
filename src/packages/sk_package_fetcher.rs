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
    pub fn fetch() -> Result<Vec<SilkSongFlattenedPackage>, SilkSongFetcherError> {
        let resp = reqwest::blocking::get(Self::URL)
            .map_err(|e| SilkSongFetcherError::SilkSongFetcherError(e.to_string()))?;

        let packages: Vec<SilkSongPackage> = resp
            .json()
            .map_err(|e| SilkSongFetcherError::JsonError(e.to_string()))?;

        let flattened: Vec<SilkSongFlattenedPackage> = packages
            .into_iter()
            .flat_map(|p| {
                p.versions
                    .into_iter()
                    .map(move |v| SilkSongFlattenedPackage {
                        package_name: p.name.clone(),
                        owner: p.owner.clone(),
                        package_name_with_version: v.full_name,
                        description: v.description,
                        download_url: v.download_url,
                        version_number: v.version_number,
                        dependencies: v.dependencies,
                    })
            })
            .collect();

        Ok(flattened)
    }
}
