mod fetching;
mod packages;

use fetching::sk_fetcher::SilkSongFetcher;

fn main() {
    let packages = SilkSongFetcher::new().fetch().unwrap();
    println!("{:#?}", packages);
}
