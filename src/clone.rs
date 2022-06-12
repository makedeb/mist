use crate::{message, mpr_cache, util};

pub fn clone(args: &clap::ArgMatches) {
    let pkg = args.value_of("pkg").unwrap();
    let mpr_url = args.value_of("mpr-url").unwrap();
    let cache = mpr_cache::new(mpr_url);

    let mut pkgbases: Vec<&str> = Vec::new();

    // Get a list of package bases.
    for pkg in &cache {
        pkgbases.push(&pkg.pkgbase);
    }

    // Abort if the package base doesn't exist.
    if !pkgbases.contains(&pkg) {
        message::error(&format!("Package base '{}' doesn't exist on the MPR.", pkg));

        // If there's a pkgbase that builds this package, guide the user to clone that package
        // instead.
        let pkgbase = util::find_pkgbase(pkg, &cache);

        match pkgbase {
            Some(pkgbase) => {
                message::error(
                    &format!(
                        "Package base '{}' exists on the MPR though, which builds '{}'. You probably want to clone that instead:",
                        pkgbase,
                        &pkg
                    )
                );

                message::error_bold(&format!("    {} clone '{}'", clap::crate_name!(), pkgbase));
            }

            None => (),
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
