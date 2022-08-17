use crate::{cache::{Cache, MprCache}, util, message};
use rust_apt::cache::Cache as AptCache;

pub fn install(args: &clap::ArgMatches) {
    let pkglist: Vec<&String> = args.get_many("pkg").unwrap().collect();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let cache = Cache::new(AptCache::new(), MprCache::new(mpr_url));

    // Package sources.
    let mut apt_pkgs: Vec<&str> = Vec::new();
    let mut mpr_pkgs: Vec<&str> = Vec::new();

    for pkg in pkglist {
        let apt_pkg = cache.get_apt_pkg(pkg);
        let mpr_pkg = cache.get_mpr_pkg(pkg);

        if apt_pkg.is_some() && mpr_pkg.is_some() {
            let resp = util::ask_question(
                &format!("Package '{}' is available from multiple sources. Please select one to install:\n", pkg),
                &vec!["APT", "MPR"],
                false
            ).remove(0);
            println!();

            if resp == "APT" {
                apt_pkgs.push(pkg);
            } else {
                mpr_pkgs.push(pkg);
            }
        }
    }

    // Clone MPR packages.
    for pkg in &mpr_pkgs {
        message::info(&format!("Cloning '{}' from the MPR...\n", pkg));
    }
}