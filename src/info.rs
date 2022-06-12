use crate::{message, mpr_cache, search, util};

pub fn info(args: &clap::ArgMatches) {
    let pkg = args.value_of("pkg").unwrap();
    let mpr_url = args.value_of("mpr-url").unwrap();
    let cache = mpr_cache::new(mpr_url);

    let mut pkgnames: Vec<&str> = Vec::new();

    // Get a list of packages.
    for pkg in &cache {
        pkgnames.push(&pkg.pkgname);
    }

    // Abort if the package base doesn't exist.
    if !pkgnames.contains(&pkg) {
        message::error(&format!("Package '{}' doesn't exist on the MPR.", pkg));
        quit::with_code(exitcode::USAGE);
    }

    // If the user wants to open the web browser page, go to that.
    if args.is_present("web") {
        let pkg_url = format!("{}/packages/{}", mpr_url, pkg);

        util::run_command(&vec!["xdg-open", &pkg_url]);
        quit::with_code(exitcode::OK);
    };

    // Find our pkgbase and print the result.
    for pkg_item in cache {
        if pkg_item.pkgname == pkg {
            let result = search::pkg_info(&pkg_item);
            println!("{}", result);
        }
    }
}
