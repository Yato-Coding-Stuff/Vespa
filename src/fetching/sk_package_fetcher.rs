use thiserror::Error;

use crate::packages::sk_package::SilkSongPackage;

#[derive(Debug, Error)]
pub enum SilkSongFetcherError {
    #[error("reqwest error: {0}")]
    SilkSongFetcherError(String),
    #[error("Parsing error: {0}")]
    JsonError(String),
}

pub struct SilkSongPackageFetcher;

impl SilkSongPackageFetcher {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_fetch_packages() {
        let fetcher = SilkSongPackageFetcher::new();

        let result = fetcher.fetch();

        // Assert the request succeeded
        assert!(
            result.is_ok(),
            "Fetcher returned an error: {:?}",
            result.err()
        );

        // Assert that we got at least one package
        let packages = result.unwrap();
        assert!(
            !packages.is_empty(),
            "Expected at least one package from Thunderstore"
        );

        // Optional: print the first package for debugging
        let first = &packages[0];
        println!(
            "First package: name='{}', version='{}'",
            first.name, first.versions[0].version_number
        );

        // Quick structural check
        assert!(!first.name.is_empty(), "Package name should not be empty");
        assert!(
            !first.versions.is_empty(),
            "Package versions should not be empty"
        );
    }
}
