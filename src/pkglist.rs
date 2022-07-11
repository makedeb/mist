use crate::cache::{Cache, MprCache};
use rust_apt::cache::Cache as AptCache;

pub fn pkglist(args: &clap::ArgMatches) {
    let mpr_url = args.get_one::<String>("mpr-url").unwrap();
    let apt_cache = AptCache::new();
    let mpr_cache = MprCache::new(mpr_url);
    let cache = Cache::new(&apt_cache, &mpr_cache);

    // Print a list of packages.
    for pkgname in cache.package_map().keys() {
        println!("{}", pkgname);
    }
}
