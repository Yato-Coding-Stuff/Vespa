mod fetching;
mod installer;
mod packages;
mod downloading;

use fetching::sk_package_fetcher::SilkSongPackageFetcher;

use crate::packages::sk_package::SilkSongIndex;

fn main() {
    let packages = SilkSongPackageFetcher::new().fetch().unwrap();
    let index = SilkSongIndex::new(packages);
    println!("{:#?}", index);
}
