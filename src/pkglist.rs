use crate::mpr_cache;

pub fn pkglist(args: &clap::ArgMatches) {
    let mpr_url = args.get_one::<String>("mpr-url").unwrap();
    let cache = mpr_cache::new(mpr_url);

    // Print a list of packages.
    for pkg in &cache {
        println!("{}", pkg.pkgname);
    }
}
