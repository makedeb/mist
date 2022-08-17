use crate::{
    cache::{Cache, CachePackage, CachePackageSource, MprCache},
    message,
    style::{self, Colorize},
};
use chrono::{TimeZone, Utc};
use rust_apt::cache::{Cache as AptCache, PackageSort};
use std::{collections::HashMap, fmt::Write};

pub fn search(args: &clap::ArgMatches) {
    let pkglist: Vec<&String> = args.get_many("pkg").unwrap().collect();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let apt_only = args.is_present("apt-only");
    let mpr_only = args.is_present("mpr-only");
    let cache = Cache::new(AptCache::new(), MprCache::new(mpr_url));

    let mut matches: Vec<String> = Vec::new();

    // Get matches.
    for pkgname in cache.pkgmap().keys() {
        // See if the package can be found in APT repos/the MPR if '--apt-only' or '--mpr-only'
        // were passed in.
        if ((apt_only && mpr_only)
            && !(cache.get_apt_pkg(pkgname).is_some() && cache.get_mpr_pkg(pkgname).is_some()))
            || (apt_only && cache.get_apt_pkg(pkgname).is_none())
            || (mpr_only && cache.get_mpr_pkg(pkgname).is_none())
        {
            continue;
        }

        // We don't care what source the package is from, we just want the pkgname.
        let pkg = cache.pkgmap().get(pkgname).unwrap().get(0).unwrap();

        for arg in &pkglist {
            if pkg.pkgname.contains(arg.as_str()) {
                matches.push(pkg.pkgname.clone());
            }

            match &pkg.pkgdesc {
                Some(pkgdesc) => {
                    if pkgdesc.to_lowercase().contains(arg.as_str()) {
                        matches.push(pkg.pkgname.clone());
                    }
                }
                None => (),
            }
        }
    }

    matches.sort_unstable();
    matches.dedup();

    // Print matches.
    //
    // We'll be comparing this length against indexes, so subtract 1 so that it functions as if we
    // started at 0.
    // Also make sure to return if we got no matches, as we'll get an underflow then.
    let matches_length = matches.len();

    if matches_length == 0 {
        message::info("No results.");
        return;
    }

    matches.sort_by_key(|a| a.to_lowercase());

    let matches_length = matches_length - 1;
    let mut result = String::new();

    for (index, pkg) in matches.iter().enumerate() {
        result.push_str(&pkg_info(cache.pkgmap(), pkg));
        if index < matches_length {
            result.push('\n');
        }
    }

    print!("{}", result);
}

pub fn pkg_info(package_map: &HashMap<String, Vec<CachePackage>>, pkg_str: &String) -> String {
    todo!("PLEASE REDO!");
}
