use crate::{
    cache::{Cache, MprCache},
    install_util, util,
};
use rust_apt::{
    cache::{Cache as AptCache, PackageSort},
    tagfile,
};
use std::{collections::HashMap, fs};

pub fn upgrade(args: &clap::ArgMatches) {
    let apt_only = args.is_present("apt-only");
    let mpr_only = args.is_present("mpr-only");
    let mpr_url: &String = args.get_one("mpr-url").unwrap();

    let cache = Cache::new(AptCache::new(), MprCache::new());

    // Get the list of packages on this system.
    let dpkg_pkgs =
        tagfile::parse_tagfile(&fs::read_to_string("/var/lib/dpkg/status").unwrap()).unwrap();

    // Convert it into a [`HashMap`] for easier access.
    let mut dpkg_map = HashMap::new();

    for pkg in dpkg_pkgs {
        dpkg_map.insert(pkg.get("Package").unwrap().to_owned(), pkg);
    }

    // The list of MPR packages we're going to update.
    let mut mpr_pkgs = vec![];

    // Check which APT packages need upgrading, and mark any for such if needed.
    for pkg in Cache::get_nonvirtual_packages(cache.apt_cache(), &PackageSort::default()) {
        let pkgname = pkg.name();

        if !mpr_only && pkg.is_upgradable(false) && let Some(pkg_control) = dpkg_map.get(&pkgname) && pkg_control.get("MPR-Package").is_none() {
            pkg.mark_install(false, !pkg.is_auto_installed());
            pkg.protect();
        } else if !apt_only && let Some(pkg_control) = dpkg_map.get(&pkgname) && pkg_control.get("MPR-Package").is_some() {
            // See if the MPR version is more recent. If so, add the package for installation.
            if crate::apt_util::cmp_versions(
                dpkg_map.get(&pkgname).unwrap().get("Version").unwrap(),
                &cache.mpr_cache().packages().get(&pkgname).unwrap().version,
            )
            .is_lt()
            {
                mpr_pkgs.push(pkgname);
            }
        }
    }

    // Get the ordering for MPR package installation.
    let mpr_install_order = install_util::order_mpr_packages(
        &cache,
        &mpr_pkgs.iter().map(|pkg| pkg.as_str()).collect(),
    );

    // Make sure any new marked APT packages are resolved properly.
    if let Err(err) = cache.apt_cache().resolve(true) {
        util::handle_errors(&err);
        quit::with_code(exitcode::UNAVAILABLE);
    }

    crate::message::warning(&format!("{}\n", mpr_install_order.len()));
    cache.commit(&mpr_install_order, mpr_url);
}
