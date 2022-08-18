use crate::{message, util, style::Colorize};
use std::{env, fs};

pub fn exit_with_git_error(pkg: &str, res: &util::CommandResult) {
    message::error(&format!(
        "Failed to clone '{}'.\n{}\n{}\n\n{}\n{}",
        pkg.green().bold(),
        "STDOUT\n------".bold(),
        std::str::from_utf8(&res.stdout).unwrap(),
        "STDERR\n------".bold(),
        std::str::from_utf8(&res.stderr).unwrap(),
    ));
    quit::with_code(exitcode::UNAVAILABLE);
}

pub fn clone_mpr_pkgs(pkglist: &Vec<&str>, mpr_url: &str) {
    let mut cache_dir = util::xdg::get_cache_dir();
    cache_dir.push("git-pkg");

    // Lint checks for the cache dir.
    if !cache_dir.exists() {
        if fs::create_dir_all(&cache_dir).is_err() {
            message::error(&format!(
                "Failed to create directory for cache directory ({}).\n",
                cache_dir.into_os_string().into_string().unwrap().green().bold()
            ));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    } else if !cache_dir.is_dir() {
        message::error(&format!(
            "Config directory path '{}' needs to be a directory, but it isn't.\n",
            cache_dir.into_os_string().into_string().unwrap().green().bold()
        ));
        quit::with_code(exitcode::UNAVAILABLE);
    }

    // Check each package.
    for pkg in pkglist {
        let mut git_dir = cache_dir.clone();
        git_dir.push(pkg);

        // Clone the repository.
        if !git_dir.exists() {
            message::info(&format!(
                "Cloning '{}' Git repository from the MPR...\n",
                pkg.green().bold()
            ));

            let res = util::Command::new(
                vec![
                    "git",
                    "clone",
                    &format!("{}/{}", mpr_url, pkg),
                    &git_dir.into_os_string().into_string().unwrap(),
                ],
                true,
                None,
            )
            .run();

            if !res.exit_status.success() {
                exit_with_git_error(pkg, &res);
            }
        // Error out if it isn't a directory.
        } else if !git_dir.is_dir() {
            message::error(&format!(
                "Path '{}' should be a folder, but is isn't.\n",
                &git_dir.into_os_string().into_string().unwrap().green().bold()
            ));
            quit::with_code(exitcode::UNAVAILABLE);
        // Otherwise, make sure the repository is up to date.
        } else {
            env::set_current_dir(git_dir).unwrap();

            message::info(&format!(
                "Making sure Git repository for '{}' is up to date...\n",
                pkg.green().bold()
            ));
            
            // Checkout to the right branch.
            let checkout_res = util::Command::new(
                vec!["git", "checkout", "master"],
                true,
                None
            ).run();

            if !checkout_res.exit_status.success() {
                exit_with_git_error(pkg, &checkout_res);
            }
            
            // Pull from the remote.
            let pull_res = util::Command::new(
                vec!["git", "pull"],
                true,
                None
            ).run();

            if !pull_res.exit_status.success() {
                exit_with_git_error(pkg, &checkout_res);
            }
        }
    }
}
