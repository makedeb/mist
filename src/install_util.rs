use crate::{
    cache::{Cache, MprPackage},
    message,
    style::Colorize,
    util,
};
use makedeb_srcinfo::SplitPackage;
use rust_apt::config::Config;
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
                cache_dir
                    .into_os_string()
                    .into_string()
                    .unwrap()
                    .green()
                    .bold()
            ));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    } else if !cache_dir.is_dir() {
        message::error(&format!(
            "Config directory path '{}' needs to be a directory, but it isn't.\n",
            cache_dir
                .into_os_string()
                .into_string()
                .unwrap()
                .green()
                .bold()
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
                    &git_dir.clone().into_os_string().into_string().unwrap(),
                ],
                true,
                None,
            )
            .run();

            if !res.exit_status.success() {
                exit_with_git_error(pkg, &res);
            }

            env::set_current_dir(git_dir).unwrap();
        // Error out if it isn't a directory.
        } else if !git_dir.is_dir() {
            message::error(&format!(
                "Path '{}' should be a folder, but is isn't.\n",
                &git_dir
                    .into_os_string()
                    .into_string()
                    .unwrap()
                    .green()
                    .bold()
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
            let checkout_res =
                util::Command::new(vec!["git", "checkout", "master"], true, None).run();

            if !checkout_res.exit_status.success() {
                exit_with_git_error(pkg, &checkout_res);
            }

            // Pull from the remote.
            let pull_res = util::Command::new(vec!["git", "pull"], true, None).run();

            if !pull_res.exit_status.success() {
                exit_with_git_error(pkg, &checkout_res);
            }
        }
    }
}

/// Mark an MPR package for installation, as well as its dependencies.
/// This function gets recursively called until a resolution is met.
/// This returns a list of the package's dependencies that were marked to be installed. Note that ordering of said dependencies must be handled outside of this function though.
fn resolve_mpr_package(
    cache: &Cache,
    pkg: &str,
    current_recursion: i32,
    recursion_limit: i32,
) -> Vec<String> {
    // If we've gone over the recursion limit, error out.
    if current_recursion > recursion_limit {
        message::error(&format!(
            "Went over the recursion limit ({}) while resolving MPR dependencies. Try increasing it via the 'APT::pkgPackageManager::MaxLoopCount' config option.\n",
            recursion_limit
        ));
        quit::with_code(exitcode::SOFTWARE);
    }

    // The list of MPR packages that need to be installed.
    let mut mpr_pkglist = Vec::new();

    let mpr_pkg = cache.mpr_cache().packages().get(&pkg.to_owned()).unwrap();
    let (system_distro, system_arch) = util::get_distro_arch_info();

    // Get the keys to get dependency and conflicts variables from.
    // Each dep group is a string such as 'pkg1|pkg2>=3'.
    // Conflicting packages can't have the `|` operator, so reflect that in the variable name here.
    let mut dep_groups: Vec<String> = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();

    // See if we can find dependencies in the order makedeb resolves distro-arch variables.
    // We're calling the struct's methods directly so we don't have to repeat the same code three times in a row. Who'd want that?
    for dep_func in [
        MprPackage::get_depends,
        MprPackage::get_makedepends,
        MprPackage::get_checkdepends,
    ] {
        if let Some(mut deps) = dep_func(mpr_pkg, Some(&system_distro), Some(&system_arch)) {
            dep_groups.append(&mut deps);
        } else if let Some(mut deps) = dep_func(mpr_pkg, Some(&system_distro), None) {
            dep_groups.append(&mut deps);
        } else if let Some(mut deps) = dep_func(mpr_pkg, None, Some(&system_arch)) {
            dep_groups.append(&mut deps);
        } else if let Some(mut deps) = dep_func(mpr_pkg, None, None) {
            dep_groups.append(&mut deps);
        }
    }

    // Sadly we must duplicate the above code for our 'conflicts' vector, at least from what I've currently tried.
    // TODO: Simplify so we don't duplicate, thanks! :D
    if let Some(mut deps) = mpr_pkg.get_conflicts(Some(&system_distro), Some(&system_arch)) {
        conflicts.append(&mut deps);
    } else if let Some(mut deps) = mpr_pkg.get_conflicts(Some(&system_distro), None) {
        conflicts.append(&mut deps);
    } else if let Some(mut deps) = mpr_pkg.get_conflicts(None, Some(&system_arch)) {
        conflicts.append(&mut deps);
    } else if let Some(mut deps) = mpr_pkg.get_conflicts(None, None) {
        conflicts.append(&mut deps);
    }

    // Mark packages for installation and removal.
    for dep_group in dep_groups {
        let mut good_dep_found = false;
        let deps: Vec<&str> = dep_group.split('|').collect();

        for dep_str in deps {
            let dep = SplitPackage::new(dep_str);

            // Find a version of a package that satisfies our requirements.
            let cache_apt_pkg = cache.get_apt_pkg(&dep.pkgname);
            let mpr_pkg = cache.mpr_cache().packages().get(&dep.pkgname);

            // We want to prefer the APT package for any dependencies, so test that first.
            if let Some(pkg) = cache_apt_pkg {
                let version_satisfied: bool;
                let apt_pkg = cache.apt_cache().get(&pkg.pkgname).unwrap();

                if let Some(dep_version) = dep.version {
                    let apt_version = apt_pkg.candidate().unwrap().version();
                    version_satisfied = util::check_version_requirement(
                        &apt_version,
                        &dep.operator.unwrap(),
                        &dep_version,
                    );
                } else {
                    version_satisfied = true;
                }

                if version_satisfied {
                    if apt_pkg.marked_install() {
                        good_dep_found = true;
                        break;
                    } else if apt_pkg.marked_keep() {
                        apt_pkg.mark_install(true, false).then_some(()).unwrap();
                        apt_pkg.protect();
                        good_dep_found = true;
                        break;
                    }
                }
            // The APT package didn't meet our requirements, so check the MPR package.
            } else if let Some(pkg) = mpr_pkg {
                let version_satisfied: bool;

                if let Some(dep_version) = dep.version {
                    version_satisfied = util::check_version_requirement(
                        &pkg.version,
                        &dep.operator.unwrap(),
                        &dep_version,
                    );
                } else {
                    version_satisfied = true;
                }

                if version_satisfied {
                    mpr_pkglist.append(&mut resolve_mpr_package(
                        cache,
                        &pkg.pkgname,
                        current_recursion + 1,
                        recursion_limit,
                    ));
                }
            }
        }

        if !good_dep_found {
            message::error(&format!(
                "Couldn't find a package to satisfy '{}' for '{}'.\n",
                dep_group.magenta(),
                pkg.green()
            ));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    }

    mpr_pkglist
}

/// Order marked MPR packages for installation.
/// This function assumes all packages in `pkglist` actually exist.
pub fn order_mpr_packages(cache: &Cache, pkglist: &Vec<&str>) -> Vec<String> {
    // The list of MPR packages that we need to install.
    let mut mpr_pkglist = Vec::new();

    // The maximum recursion limit for calls to `resolve_mpr_package`.
    let recursion_limit = Config::new().int("APT::pkgPackageManager::MaxLoopCount", 50);

    for pkg in pkglist {
        // Append the package itself.
        mpr_pkglist.push(pkg.to_string());

        // Append its dependencies.
        mpr_pkglist.append(&mut resolve_mpr_package(cache, pkg, 1, recursion_limit));
    }

    // TODO: This list needs ordered before returning it so that MPR packages are installed in the correct way.
    message::warning("PLEASE ORDER BEFORE MERGE {:?} THX!\n");
    mpr_pkglist
}
