mod package;
pub mod package_fetcher;
pub mod package_scanner;

pub use crate::base::packages::{IndexError, PackageRecord};

pub(crate) fn fetch_package_records(blacklist: &[&str]) -> Result<Vec<PackageRecord>, IndexError> {
    let packages = package_fetcher::PackageFetcher::fetch()
        .map_err(|error| IndexError::FetcherError(error.to_string()))?;
    Ok(package::flatten_packages(packages, blacklist))
}
