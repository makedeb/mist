use crate::{cache::run_transaction, message, util::handle_errors};
use rust_apt::cache::{Cache as AptCache, PackageSort};

pub fn remove(args: &clap::ArgMatches) {
    let cache = AptCache::new();
    let pkglist: Vec<&String> = {
        if let Some(pkglist) = args.get_many("pkg") {
            pkglist.collect()
        } else {
            Vec::new()
        }
    };
    let purge = args.is_present("purge");
    let autoremove = args.is_present("autoremove");

    // Remove the user requested packages.
    for pkgname in pkglist {
        if let Some(pkg) = cache.get(pkgname) {
            if !pkg.is_installed() {
                message::warning(&format!(
                    "Package '{}' isn't installed, so not removing.",
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
        for pkg in cache.packages(&PackageSort::default()) {
            if pkg.is_auto_removable() {
                pkg.mark_delete(purge).then_some(()).unwrap();
                pkg.protect();
            }
        }
    }

    if let Err(err) = cache.resolve(true) {
        handle_errors(&err);
        quit::with_code(exitcode::UNAVAILABLE);
    }

    run_transaction(&cache, purge);
}
