use crate::{cache::{Cache, MprCache}, util, message};
use rust_apt::cache::Cache as AptCache;
use std::path::Path;

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
    let git_dir = dirs::cache_dir().unwrap().into_os_string().into_string().unwrap() + "/git-mpr";

    for pkg in &mpr_pkgs {
        let repo_path_str = git_dir.clone() + "/" + pkg;
        let repo_path = Path::new(&repo_path_str);

        if !repo_path.exists() {
            message::info(&format!(
                "Cloning '{}' from the MPR...\n",
                pkg
            ));

            let cmd = util::Command::new(
                vec![
                    "git".to_owned(),
                    "clone".to_owned(),
                    format!("{}/{}", mpr_url, pkg),
                    repo_path_str
                ],
                false,
                None
            );

            if !cmd.run().exit_status.success() {
                message::error("Failed to clone package from the MPR.\n");
                quit::with_code(exitcode::UNAVAILABLE);
            }
            println!();
        } else if !repo_path.is_dir() {
            message::error(&format!(
                "Repository path '{}' isn't a directory.\n",
                repo_path.display()
            ));
            quit::with_code(exitcode::UNAVAILABLE);
        } else {
            message::info(&format!(
                "Updating Git repository for '{}'...\n",
                pkg
            ));

            let cmd = util::Command::new(
                vec!["git".to_owned(), "pull".to_owned(), pkg.to_string()],
                false,
                None
            );

            if !cmd.run().exit_status.success() {
                message::error("Failed to update Git repository.\n");
                quit::with_code(exitcode::UNAVAILABLE);
            }
        }
    }
}