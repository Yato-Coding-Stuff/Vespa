use serde::Deserialize;

/*
 * Thunderstore's API sucks, so we're forced to do it like this.
 * */

#[derive(Debug, Deserialize)]
pub struct SilkSongPackage {
    name: String,
    owner: String,
    package_url: String,
    versions: Vec<SilkSongVersion>,
}

#[derive(Debug, Deserialize)]
pub struct SilkSongVersion {
    full_name: String,
    description: String,
    version_number: String,
    dependencies: Vec<String>,
}
