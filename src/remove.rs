use crate::{
    apt_util,
    cache::{Cache, MprCache},
    message, util,
};
use rust_apt::cache::{Cache as AptCache, PackageSort};

pub fn remove(args: &clap::ArgMatches) {
    let pkglist: Vec<&String> = {
        if let Some(pkglist) = args.get_many("pkg") {
            pkglist.collect()
        } else {
            Vec::new()
        }
    };
    let purge = args.is_present("purge");
    let autoremove = args.is_present("autoremove");
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let cache = Cache::new(AptCache::new(), MprCache::new(mpr_url));

    // Lock the cache.
    if let Err(err) = apt_util::apt_lock() {
        util::handle_errors(&err);
        quit::with_code(exitcode::UNAVAILABLE);
    }

    // Remove the user requested packages.
    for pkgname in pkglist {
        if let Some(pkg) = cache.apt_cache().get(pkgname) {
            if !pkg.is_installed() {
                message::warning(&format!(
                    "Package '{}' isn't installed, so not removing.\n",
                    pkg.name(),
                ));
                continue;
            }

            pkg.mark_delete(purge).then_some(()).unwrap();
            pkg.protect();
        }
    }

    // Remove any packages that are no longer needed.
    if autoremove {
        for pkg in cache.apt_cache().packages(&PackageSort::default()) {
            if pkg.is_auto_removable() {
                pkg.mark_delete(purge).then_some(()).unwrap();
                pkg.protect();
            }
        }
    }

    if let Err(err) = cache.apt_cache().resolve(true) {
        util::handle_errors(&err);
        quit::with_code(exitcode::UNAVAILABLE);
    }

    // Unlock the cache so our transaction can complete.
    apt_util::apt_unlock();

    // Commit our changes.
    cache.commit(&Vec::new());
}
