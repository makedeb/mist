use crate::{
    cache::{Cache, MprCache},
    message, search, util,
};
use rust_apt::cache::Cache as AptCache;

pub fn info(args: &clap::ArgMatches) {
    let pkg: &String = args.get_one("pkg").unwrap();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let apt_cache = AptCache::new();
    let mpr_cache = MprCache::new(mpr_url);
    let cache = Cache::new(&apt_cache, &mpr_cache);
    let package_map = cache.package_map();

    // Abort if the package base doesn't exist.
    match package_map.get(&pkg) {
        Some(_) => (),
        None => {
            message::error(&format!("Package '{}' doesn't exist on the MPR.", pkg));
            quit::with_code(exitcode::USAGE);
        }
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

    // Print the info for our package.
    println!("{}", search::pkg_info(&package_map, pkg));
}
