use crate::{
    cache::{Cache, MprCache},
    cli::{Cli, CliInstall},
    install_util, message,
    style::Colorize,
    util,
};
use rust_apt::cache::Cache as AptCache;

pub fn install(args: &Cli, cmd_args: &CliInstall) {
    let pkglist = &cmd_args.pkg;
    let mpr_url = &args.mpr_url;
    let cache = Cache::new(AptCache::new(), MprCache::new());

    // Package sources.
    let mut apt_pkgs: Vec<&str> = Vec::new();
    let mut mpr_pkgs: Vec<&str> = Vec::new();

    // Check real quick for any packages that cannot be found. We don't want to ask
    // the user anything else if there's packages that cannot be found, instead we
    // should just show those packages and abort.
    let mut unfindable = false;

    for pkg in pkglist {
        if cache.apt_cache().get(pkg).is_none() && !cache.mpr_cache().packages().contains_key(pkg) {
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
        let apt_pkg = cache.apt_cache().get(pkg);
        let mpr_pkg = cache.mpr_cache().packages().get(pkg);

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

    // Get the ordering for MPR package installation.
    let mpr_install_order = install_util::order_mpr_packages(&cache, &mpr_pkgs);

    // Make sure any new marked APT packages are resolved properly.
    if let Err(err) = cache.apt_cache().resolve(true) {
        util::handle_errors(&err);
        quit::with_code(exitcode::UNAVAILABLE);
    }

    cache.commit(&mpr_install_order, mpr_url);
}
