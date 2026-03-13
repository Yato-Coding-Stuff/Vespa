use std::io::{Read, Write};

use reqwest::blocking::Client;
use tempfile::TempDir;
use thiserror::Error;

use crate::cli::presenter::events::install_event::InstallEvent;

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

    pub fn download<F: FnMut(InstallEvent)>(
        &self,
        url: &str,
        progress: &mut F,
    ) -> Result<TempDir, SilkSongPackageDownloaderError> {
        let dir = TempDir::new()?;

        let parsed_url = url
            .parse::<reqwest::Url>()
            .map_err(|e| SilkSongPackageDownloaderError::InvalidUrl(e.to_string()))?;

        let client = Client::new();
        let mut response = client.get(parsed_url).send()?;

        if !response.status().is_success() {
            return Err(SilkSongPackageDownloaderError::Http(
                response.error_for_status().unwrap_err(),
            ));
        }

        let total = response.content_length().unwrap_or(0);

        progress(InstallEvent::StartingDownload { total });

        let file_path = dir.path().join("package.zip");
        let mut file = std::fs::File::create(file_path)?;

        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 8192];

        loop {
            let n = response.read(&mut buffer)?;
            if n == 0 {
                break;
            }

            file.write_all(&buffer[..n])?;
            downloaded += n as u64;

            progress(InstallEvent::DownloadProgress { downloaded });
        }

        Ok(dir)
    }
}
