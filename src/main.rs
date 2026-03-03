mod fetching;
mod installer;
mod packages;

use fetching::sk_fetcher::SilkSongFetcher;

use crate::packages::sk_package::SilkSongIndex;

fn main() {
    let packages = SilkSongFetcher::new().fetch().unwrap();
    let index = SilkSongIndex::new(packages);
    println!("{:#?}", index);
}
