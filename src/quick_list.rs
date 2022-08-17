use crate::cache::MprCache;
use rust_apt::cache::{Cache as AptCache, PackageSort};

pub fn quick_list(args: &clap::ArgMatches) {
    let prefix: &String = args.get_one("prefix").unwrap();
    let apt_only = args.is_present("apt-only");
    let mpr_only = args.is_present("mpr-only");
    let mpr_url: &String = args.get_one("mpr-url").unwrap();

    if !mpr_only {
        for pkg in AptCache::new().packages(&PackageSort::default()) {
            let pkgname = pkg.name();

            if pkgname.starts_with(prefix) {
                println!("{}", pkg.name());
            }
        }
    }

    if !apt_only {
        for pkg in MprCache::new(mpr_url).packages() {
            let pkgname = &pkg.pkgname;

            if pkgname.starts_with(prefix) {
                println!("{}", pkgname);
            }
        }
    }
}
