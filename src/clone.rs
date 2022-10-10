use crate::{
    cache::{Cache, MprCache},
    message, util,
};
use rust_apt::cache::Cache as AptCache;

pub fn clone(pkg: &String, mpr_url: &String) {
    let cache = Cache::new(AptCache::new(), MprCache::new());
    let mut pkgbases: Vec<&String> = Vec::new();

    // Get a list of package bases.
    for pkg in cache.mpr_cache().packages().values() {
        pkgbases.push(&pkg.pkgbase);
    }

    // Abort if the package base doesn't exist.
    if !pkgbases.contains(&pkg) {
        message::error(&format!(
            "Package base '{}' doesn't exist on the MPR.\n",
            pkg
        ));

        // If there's a pkgbase that builds this package, guide the user to clone that
        // package instead.
        if let Some(pkgbase) = cache.find_pkgbase(pkg) {
            message::error(&format!(
                "Package base '{}' exists on the MPR though, which builds '{}'. You probably want to clone that instead:\n",
                pkgbase,
                &pkg
            ));
            message::error(&format!(
                "    {} clone '{}'\n",
                clap::crate_name!(),
                pkgbase
            ));
        }

        quit::with_code(exitcode::USAGE);
    }

    // Clone the package.
    let pkg_url = format!("{}/{}", mpr_url, pkg);
    let mut cmd = util::sudo::run_as_normal_user("git");
    cmd.args(["clone", &pkg_url]);
    let exit_code = cmd.output().unwrap().status;

    if !exit_code.success() {
        message::error("Failed to clone package.\n");
        quit::with_code(exitcode::UNAVAILABLE);
    };
}
