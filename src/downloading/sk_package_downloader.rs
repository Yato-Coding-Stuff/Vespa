use std::io::copy;

use reqwest::blocking::Client;
use tempfile::TempDir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SilkSongPackageDownloaderError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to create or write to file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

pub struct SilkSongPackageDownloader;

impl SilkSongPackageDownloader {
    pub fn new() -> Self {
        SilkSongPackageDownloader
    }

    pub fn download(url: &str) -> Result<TempDir, SilkSongPackageDownloaderError> {
        let dir = TempDir::new()?;

        let parsed_url = url
            .parse::<reqwest::Url>()
            .map_err(|e| SilkSongPackageDownloaderError::InvalidUrl(e.to_string()))?;

        let client = Client::new();

        let mut response = client.get(parsed_url.clone()).send()?;

        if !response.status().is_success() {
            return Err(SilkSongPackageDownloaderError::Http(
                response.error_for_status().unwrap_err(),
            ));
        }

        let file_path = dir.path().join("package.zip");
        let mut file = std::fs::File::create(file_path)?;
        copy(&mut response, &mut file)?;

        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_download_real_file() {
        // This is a small test file hosted publicly
        let url = "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf";

        let temp_dir = SilkSongPackageDownloader::download(url).expect("Failed to download file");

        // The TempDir exists
        assert!(temp_dir.path().exists());

        // Check that something was written (should not be empty)
        let entries: Vec<_> = fs::read_dir(temp_dir.path())
            .expect("Failed to read temp dir")
            .collect();
        assert!(!entries.is_empty(), "No file was written to temp dir");
    }

    #[test]
    fn test_download_invalid_url() {
        let url = "not a url";

        let result = SilkSongPackageDownloader::download(url);
        assert!(result.is_err());

        if let Err(SilkSongPackageDownloaderError::InvalidUrl(_)) = result {
            // expected
        } else {
            panic!("Expected InvalidUrl error");
        }
    }

    #[test]
    fn test_download_404() {
        // Known 404 URL
        let url = "https://httpstat.us/404";

        let result = SilkSongPackageDownloader::download(url);
        assert!(result.is_err());

        match result.unwrap_err() {
            SilkSongPackageDownloaderError::Http(_) => {} // expected
            e => panic!("Expected Http error, got {:?}", e),
        }
    }
}
