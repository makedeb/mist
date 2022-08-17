use crate::{cache::{Cache, CachePackage, MprCache}, style};
use rust_apt::cache::{Cache as AptCache, PackageSort};

pub fn list(args: &clap::ArgMatches) {
    let pkglist: Vec<&String> = match args.get_many("pkg") {
        Some(pkglist) => pkglist.collect(),
        None => Vec::new(),
    };
    let apt_only = args.is_present("apt-only");
    let mpr_only = args.is_present("mpr-only");
    let installed_only = args.is_present("installed-only");
    let name_only = args.is_present("name-only");
    let mpr_url: &String = args.get_one("mpr-url").unwrap();

    let cache = Cache::new(AptCache::new(), MprCache::new(mpr_url));
    let mut candidates: Vec<&Vec<CachePackage>> = Vec::new();

    if !pkglist.is_empty() {
        for pkg in pkglist {
            if let Some(pkg_group) = cache.pkgmap().get(pkg) {
                candidates.push(pkg_group);
            }
        }
    } else {
        for pkg_group in cache.pkgmap().values() {
            candidates.push(pkg_group);
        }
    }

    let mut matches: Vec<&Vec<CachePackage>> = Vec::new();

    for pkg_group in candidates {
        let pkgname = &pkg_group.get(0).unwrap().pkgname;

        // APT only.
        if apt_only {
            if let None = cache.get_apt_pkg(pkgname) {
                continue;
            }
        }

        // MPR only.
        if mpr_only {
            if let None = cache.get_mpr_pkg(pkgname) {
                continue;
            }
        }

        // Installed only.
        if installed_only {
            match cache.get_apt_pkg(pkgname) {
                Some(pkg) => {
                    if !cache.apt_cache().get(pkgname).unwrap().is_installed() {
                        continue;
                    }
                }
                None => continue,
            }
        }

        // Package be passed all the tests bro. We's be adding it to the vector now.
        matches.push(pkg_group);
    }

    let matches_len = matches.len();

    for (index, pkg_group) in matches.iter().enumerate() {
        if name_only || index == matches_len - 1 {
            println!("{}", style::generate_pkginfo_entry(pkg_group, &cache, name_only));
        } else {
            println!("{}\n", style::generate_pkginfo_entry(pkg_group, &cache, name_only));
        }
    }
}
