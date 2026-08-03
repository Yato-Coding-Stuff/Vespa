mod package;
pub mod package_fetcher;

use crate::base::packages::{IndexError, PackageRecord};

pub(crate) fn fetch_package_records(
    blacklist: &[&str],
) -> Result<Vec<PackageRecord>, IndexError> {
    let packages = package_fetcher::PackageFetcher::fetch()
        .map_err(|error| IndexError::FetcherError(error.to_string()))?;

    Ok(packages
        .into_iter()
        .filter(|package| !blacklist.contains(&package.name.as_str()))
        .map(PackageRecord::from)
        .collect())
}
