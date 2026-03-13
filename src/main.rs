mod fetching;
mod installer;
mod packages;
mod downloading;
mod util;

use fetching::sk_package_fetcher::SilkSongPackageFetcher;

use crate::{packages::sk_package::SilkSongIndex, util::config::Config};

fn main() {
    let config = Config::load().expect("Failed to load config");
    println!("Config: {:#?}", config);
}
