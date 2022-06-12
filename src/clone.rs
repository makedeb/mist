use crate::{message, mpr_cache, util};
use std::collections::HashMap;

pub fn clone(args: &clap::ArgMatches) {
    let pkg = args.value_of("pkg").unwrap();
    let mpr_url = args.value_of("mpr-url").unwrap();
    let cache = mpr_cache::new(mpr_url);

    let mut pkgbases: Vec<&str> = Vec::new();
    let mut pkgbase_mappings: HashMap<&str, &str> = HashMap::new();

    // Get a list of package bases.
    for pkg in &cache {
        pkgbases.push(&pkg.pkgbase);
        pkgbase_mappings.insert(&pkg.pkgname, &pkg.pkgbase);
    }

    // Abort if the package base doesn't exist.
    if !pkgbases.contains(&pkg) {
        message::error(&format!("Package base '{}' doesn't exist on the MPR.", pkg));

        // If the specified package doesn't exist, but another package base builds the requested
        // package, the user probably wants to clone that instead (i.e. 'rustc' for `cargo` on the MPR).
        if pkgbase_mappings.contains_key(&pkg) {
            message::error(
                &format!(
                    "Package base '{}' exists on the MPR though, which builds '{}'. You probably want to clone that instead:",
                    pkgbase_mappings[&pkg],
                    &pkg
                )
            );

            message::error_bold(&format!(
                "    {} clone '{}'",
                clap::crate_name!(),
                pkgbase_mappings[&pkg]
            ));
        }

        quit::with_code(exitcode::USAGE);
    }

    // Clone the package.
    let pkg_url = format!("{}/{}", mpr_url, pkg);
    let cmd = vec!["git", "clone", "--h", &pkg_url];
    let exit_code = util::run_command(&cmd);

    if !exit_code.success() {
        message::error("Failed to clone package.");
        quit::with_code(exitcode::UNAVAILABLE);
    };
}
