use crate::{cache::MprCache, message, util};

pub fn clone(args: &clap::ArgMatches) {
    let pkg: &String = args.get_one("pkg").unwrap();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let mpr_cache = MprCache::new(mpr_url);

    let mut pkgbases: Vec<&String> = Vec::new();

    // Get a list of package bases.
    for pkg in mpr_cache.packages() {
        pkgbases.push(&pkg.pkgbase);
    }

    // Abort if the package base doesn't exist.
    if !pkgbases.contains(&pkg) {
        message::error(&format!("Package base '{}' doesn't exist on the MPR.", pkg));

        // If there's a pkgbase that builds this package, guide the user to clone that package
        // instead.
        let pkgbase = util::find_pkgbase(pkg, &mpr_cache);

        match pkgbase {
            Some(pkgbase) => {
                message::error(
                    &format!(
                        "Package base '{}' exists on the MPR though, which builds '{}'. You probably want to clone that instead:",
                        pkgbase,
                        &pkg
                    )
                );

                message::error(&format!("    {} clone '{}'", clap::crate_name!(), pkgbase));
            }

            None => (),
        }

        quit::with_code(exitcode::USAGE);
    }

    // Clone the package.
    let pkg_url = format!("{}/{}", mpr_url, pkg);
    let cmd = util::CommandInfo {
        args: &vec!["git", "clone", &pkg_url],
        capture: false,
        stdin: None,
    };
    let exit_code = util::run_command(&cmd).exit_status;

    if !exit_code.success() {
        message::error("Failed to clone package.");
        quit::with_code(exitcode::UNAVAILABLE);
    };
}
