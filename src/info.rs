use crate::{message, mpr_cache, search, util};

pub fn info(args: &clap::ArgMatches) {
    let pkg: &String = args.get_one("pkg").unwrap();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let cache = mpr_cache::new(mpr_url);
    let mut pkgnames: Vec<&String> = Vec::new();

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
    if args.contains_id("web") {
        let pkg_url = format!("{}/packages/{}", mpr_url, pkg);
        let cmd = util::CommandInfo {
            args: &vec!["xdg-open", &pkg_url],
            capture: false,
            stdin: None,
        };
        util::run_command(&cmd);
        quit::with_code(exitcode::OK);
    };

    // Find our pkgbase and print the result.
    for pkg_item in cache {
        if &pkg_item.pkgname == pkg {
            let result = search::pkg_info(&pkg_item);
            println!("{}", result);
        }
    }
}
