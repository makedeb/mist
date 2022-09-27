use crate::{cache::Cache, message, style::Colorize, util};
use rust_apt::{cache::Cache as AptCache, tagfile::TagSection};
use std::{env, fs};

pub fn clone_mpr_pkgs(pkglist: &Vec<&str>, mpr_url: &str) {
    let mut cache_dir = util::xdg::get_cache_dir();
    cache_dir.push("git-pkg");
    util::sudo::to_normal();
    util::fs::create_dir(&cache_dir.clone().into_os_string().into_string().unwrap());
    util::sudo::to_root();

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

            {
                let mut cmd = util::sudo::run_as_normal_user("git");
                cmd.arg("clone");
                cmd.arg(format!("{}/{}", mpr_url, pkg));
                cmd.arg(git_dir.clone().into_os_string().into_string().unwrap());

                let status = cmd.output().unwrap().status;
                util::check_exit_status(&cmd, &status);
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
            {
                let mut cmd = util::sudo::run_as_normal_user("git");
                cmd.args(["checkout", "master"]);
                let status = cmd.output().unwrap().status;
                util::check_exit_status(&cmd, &status);
            }

            // Pull from the remote.
            {
                let mut cmd = util::sudo::run_as_normal_user("git");
                cmd.arg("pull");
                let status = cmd.output().unwrap().status;
                util::check_exit_status(&cmd, &status);
            }
        }
    }
}

/// Order marked MPR packages for installation.
/// This function assumes all packages in `pkglist` actually exist and that all
/// changes have already been marked in the `cache` object.
pub fn order_mpr_packages(cache: &Cache, pkglist: &Vec<&str>) -> Vec<Vec<String>> {
    let mut cache_dir = util::xdg::get_global_cache_dir();
    cache_dir.push("deb-pkgs");
    env::set_current_dir(&cache_dir).unwrap();

    // Get the list of MPR packages on this system. These are created in
    // [`crate::update::update`].
    let mut debs_owned = vec![];

    for path in fs::read_dir("./").unwrap() {
        let filename = path.unwrap().file_name().into_string().unwrap();

        if filename.ends_with(".deb") {
            debs_owned.push(filename);
        }
    }

    let debs: Vec<&str> = debs_owned.iter().map(|s| s.as_str()).collect();

    // Create a new cache object that we'll use to find what packages are to be
    // installed from the MPR.
    let new_cache = AptCache::debs(&debs).unwrap();

    // Mirror the changes from the passed in cache into this one.
    for pkg in cache.apt_cache().get_changes(false) {
        let version = pkg.candidate().unwrap().version();
        let new_cache_pkg = new_cache.get(&pkg.name()).unwrap();
        new_cache_pkg.get_version(&version).unwrap().set_candidate();

        if pkg.marked_install()
            || pkg.marked_downgrade()
            || pkg.marked_reinstall()
            || pkg.marked_upgrade()
        {
            assert!(new_cache_pkg.mark_install(false, !pkg.is_auto_installed()));
        } else if pkg.marked_delete() {
            new_cache_pkg.mark_delete(false);
        } else if pkg.marked_purge() {
            new_cache_pkg.mark_delete(true);
        } else if pkg.marked_keep() {
            new_cache_pkg.mark_keep();
        } else {
            unreachable!(
                "Package '{}' is in an unknown state.",
                pkg.name().bold().green()
            );
        }

        new_cache_pkg.protect();
    }

    // Mark any MPR packages for installation.
    for pkg_str in pkglist {
        let pkg = new_cache.get(pkg_str).unwrap();

        // Get the package's version in its control file.
        let tagsection =
            TagSection::new(&fs::read_to_string(pkg_str.to_string() + "/DEBIAN/control").unwrap())
                .unwrap();
        let version = tagsection.get("Version").unwrap();

        pkg.get_version(version).unwrap().set_candidate();
        assert!(pkg.mark_install(false, true));
        pkg.protect();
    }

    // Resolve the cache.
    if let Err(err) = new_cache.resolve(true) {
        message::error("Couldn't resolve MPR packages.\n");
        util::handle_errors(&err);
        quit::with_code(exitcode::UNAVAILABLE);
    }

    // Get the list of changes for MPR packages.
    let mut mpr_pkgs = vec![vec![]];
    let apt_cache = cache.apt_cache();

    for pkg in new_cache.get_changes(false) {
        let mut invalid_change: Option<&str> = None;
        let mpr_pkg_change = {
            if let Ok(string) = fs::read_to_string(pkg.name() + "/DEBIAN/control")
                && let Ok(tagsection) = TagSection::new(&string)
                && tagsection.get("Version").unwrap() == &pkg.candidate().unwrap().version() {
                    true
                } else {
                    false
                }
        };

        if apt_cache.get(&pkg.name()).is_some() {
            let normal_pkg = apt_cache.get(&pkg.name()).unwrap();
            let normal_pkg_keep = normal_pkg.marked_keep();

            // Mirror these changes back into the normal cache.
            if pkg.marked_install()
                || pkg.marked_downgrade()
                || pkg.marked_reinstall()
                || pkg.marked_upgrade()
            {
                if !normal_pkg_keep
                    && !normal_pkg.marked_install()
                    && pkg.marked_downgrade()
                    && pkg.marked_reinstall()
                    && pkg.marked_upgrade()
                {
                    invalid_change = Some("install");
                }
                (!mpr_pkg_change).then(|| assert!(normal_pkg.mark_install(false, true)));
            } else if pkg.marked_delete() {
                if !normal_pkg_keep && !normal_pkg.marked_delete() {
                    invalid_change = Some("delete");
                }
                (!mpr_pkg_change).then(|| normal_pkg.mark_delete(false));
            } else if pkg.marked_purge() {
                if !normal_pkg_keep && !normal_pkg.marked_purge() {
                    invalid_change = Some("purge");
                }
                (!mpr_pkg_change).then(|| normal_pkg.mark_delete(true));
            } else if pkg.marked_keep() {
                if !normal_pkg_keep {
                    invalid_change = Some("keep");
                }
                (!mpr_pkg_change).then(|| normal_pkg.mark_keep());
            }

            if let Some(change) = invalid_change {
                message::error(&format!(
                    "There was an issue marking '{}', as it was supposed to be marked for {} but wasn't.",
                    pkg.name().bold().green(),
                    change
                ));
                quit::with_code(exitcode::UNAVAILABLE);
            }

            normal_pkg.protect();
        }

        if mpr_pkg_change {
            mpr_pkgs.first_mut().unwrap().push(pkg);
        }
    }

    // Order the MPR packages.
    //
    // The changed index. A tuple containing an array containing the element to
    // remove (the vector's position, and the package position in that vector).
    let mut changed_index: Option<[usize; 2]> = Some([0, 0]);

    while changed_index.is_some() {
        let index_len = mpr_pkgs.len() - 1;
        changed_index = None;

        'main: for (vec_index, pkg_vec) in mpr_pkgs.iter().enumerate() {
            for (pkg_index, pkg) in pkg_vec.iter().enumerate() {
                // Get the dependencies of this package.
                let dependencies: Vec<String> = {
                    let mut deps = vec![];
                    let version = pkg.candidate().unwrap();
                    if let Some(dep_groups) = version.dependencies() {
                        for dep_grp in dep_groups {
                            for dep in &dep_grp.base_deps {
                                deps.push(dep.name().to_owned());
                            }
                        }
                    }

                    deps
                };

                // Loop over this vector and each one after this, and see if any of the
                // packages it contains is a package from `dependencies`. If it
                // is, this package needs to be moved to a vector after that
                // package's vector.
                if let Some(inner_pkg_vecs) = mpr_pkgs.get(vec_index..=index_len) {
                    for inner_pkg_vec in inner_pkg_vecs {
                        for inner_pkg in inner_pkg_vec {
                            // See if this package or any packages it provides are in the dependency
                            // list (i.e. 'lbrynet-bin' providing 'lbrynet' on the MPR).
                            let mut provides_list = inner_pkg.candidate().unwrap().provides_list();
                            provides_list.push((inner_pkg.name(), None));

                            for (pkgname, _) in provides_list {
                                if dependencies.contains(&pkgname) {
                                    changed_index = Some([vec_index, pkg_index]);
                                    break 'main;
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(change) = changed_index {
            let new_vec_position = change[0] + 1;

            // Remove the element from the specified position.
            let pkg = mpr_pkgs.get_mut(change[0]).unwrap().remove(change[1]);

            if let Some(vec) = mpr_pkgs.get_mut(new_vec_position) {
                vec.push(pkg);
            } else {
                mpr_pkgs.push(vec![pkg]);
            }
        }
    }

    let mut returned_vec = vec![];

    for vec in mpr_pkgs {
        returned_vec.push(vec.iter().map(|pkg| pkg.name()).collect());
    }

    returned_vec
}

#[allow(clippy::ptr_arg)]
// Convert a list of MPR packages (obtained from [`order_mpr_packages`]) into a
// list of MPR package bases.
pub fn pkgnames_to_pkgbases(cache: &Cache, pkglist: &Vec<Vec<String>>) -> Vec<Vec<String>> {
    let mut returned_vec = vec![];

    // Replace each entry in the list with its corresponding pkgbase.
    for vec in pkglist.iter() {
        let mut inner_vec = vec![];

        for pkgname in vec {
            let mpr_pkg = cache.mpr_cache().packages().get(pkgname).unwrap();
            inner_vec.push(mpr_pkg.pkgbase.clone());
        }

        returned_vec.push(inner_vec);
    }

    // Remove any entries that are duplicates, considering the last valid option.
    // I.e. a `Vec<Vec<"cargo">, Vec<"rustc", "toast">>` would turn into
    // `Vec<Vec<"rustc", Vec<"rustc", "toast">>`.
    // Keep the element closest to the beginning of the main vector and discord any
    // others in order to respect dependency installation order.
    let mut removal_index: Option<[usize; 2]> = Some([0, 0]);

    while removal_index.is_some() {
        removal_index = None;
        let vec_len = returned_vec.len();

        'main: for (vec_index, vec) in returned_vec.iter().enumerate() {
            for pkg in vec {
                // Loop over each index after this one, and see if there's a matching package.
                if let Some(inner_vecs) = returned_vec.get(vec_index..=vec_len) {
                    for (inner_vec_index, inner_vec) in inner_vecs.iter().enumerate() {
                        for (inner_pkg_index, inner_pkg) in inner_vec.iter().enumerate() {
                            if pkg == inner_pkg {
                                removal_index = Some([inner_vec_index, inner_pkg_index]);
                                break 'main;
                            }
                        }
                    }
                }
            }
        }

        // If we found a matching package later in the index, remove it.
        if let Some(index) = removal_index {
            returned_vec.get_mut(index[0]).unwrap().remove(index[1]);
        }
    }

    returned_vec
}
