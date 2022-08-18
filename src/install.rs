use crate::{
    cache::{Cache, MprCache},
    install_util, message, util, style::Colorize
};
use rust_apt::cache::Cache as AptCache;
use std::path::Path;

pub fn install(args: &clap::ArgMatches) {
    let pkglist: Vec<&String> = args.get_many("pkg").unwrap().collect();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let cache = Cache::new(AptCache::new(), MprCache::new(mpr_url));

    // Package sources.
    let mut apt_pkgs: Vec<&str> = Vec::new();
    let mut mpr_pkgs: Vec<&str> = Vec::new();

    // Check real quick for any packages that cannot be found. We don't want to ask the user anything else if there's packages that cannot be found, instead we should just show those packages and abort.
    let mut unfindable = false;

    for pkg in &pkglist {
        if cache.get_apt_pkg(pkg).is_none() && cache.get_mpr_pkg(pkg).is_none() {
            message::error(&format!(
                "Unable to find package '{}'.\n",
                pkg.green().bold()
            ));
            unfindable = true;
        }
    }

    if unfindable {
        quit::with_code(exitcode::USAGE);
    }

    for pkg in pkglist {
        let apt_pkg = cache.get_apt_pkg(pkg);
        let mpr_pkg = cache.get_mpr_pkg(pkg);

        if apt_pkg.is_some() && mpr_pkg.is_some() {
            let resp = util::ask_question(
                &format!("Package '{}' is available from multiple sources. Please select one to install:\n", pkg.green().bold()),
                &vec!["APT", "MPR"],
                false
            ).remove(0);
            println!();

            if resp == "APT" {
                apt_pkgs.push(pkg);
            } else {
                mpr_pkgs.push(pkg);
            }
        } else if apt_pkg.is_some() {
            apt_pkgs.push(pkg);
        } else if mpr_pkg.is_some() {
            mpr_pkgs.push(pkg);
        }
    }

    // Clone MPR packages.
    install_util::clone_mpr_pkgs(&mpr_pkgs, mpr_url);

    // Mark any APT packages for installation.
    for pkg in apt_pkgs {
        let apt_pkg = cache.apt_cache().get(pkg).unwrap();

        if !apt_pkg.mark_install(false, true) {
            message::error(&format!(
                "There was an issue marking '{}' for installation.\n",
                pkg.green().bold()
            ));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    }

    todo!("Need to get package ordering for MPR packages, and I think we'll be good for installation functionality.");
}