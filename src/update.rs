use crate::{cache::MprCache, message, progress::MistAcquireProgress, style::Colorize, util};
use makedeb_srcinfo::SplitDependency;
use rust_apt::{cache::Cache as AptCache, progress::AcquireProgress, tagfile::TagSection};
use std::{
    env, fs,
    io::{self, Write},
    path,
    process::Command,
};

pub fn update(args: &clap::ArgMatches) {
    let mpr_url: &String = args.get_one("mpr-url").unwrap();

    // For some reason we have to set our current UID to 0 instead of just the EUID
    // when using setuid functionality. TODO: No clue why, but this fixes the
    // issue for now.
    users::switch::set_current_uid(0).unwrap();

    // Update APT packages.
    let cache = AptCache::new();
    let mut progress: Box<dyn AcquireProgress> = Box::new(MistAcquireProgress {});

    if let Err(error) = cache.update(&mut progress) {
        for msg in error.what().split(';') {
            if msg.starts_with("E:") {
                message::error(&format!("{}\n", msg.strip_prefix("E:").unwrap()));
            } else if msg.starts_with("W:") {
                message::warning(&format!("{}\n", msg.strip_prefix("W:").unwrap()));
            };
        }
    };

    // Get the new MPR cache.
    let client = reqwest::blocking::Client::new();
    let resp = match client
        .get(format!("{}/packages-meta-ext-v2.json.gz", mpr_url))
        .send()
    {
        Ok(resp) => resp.bytes().unwrap(),
        Err(err) => {
            message::error(&format!("Failed to make request [{}]\n", err));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    };

    let mpr_cache = match MprCache::validate_data(&resp) {
        Ok(mpr_cache) => mpr_cache,
        Err(_) => {
            message::error("There was an issue validating the downloaded MPR cache archive.");
            quit::with_code(exitcode::UNAVAILABLE);
        }
    };

    // Create the '.deb' files for the packages in the MPR cache.
    let mut cache_dir = util::xdg::get_global_cache_dir();
    cache_dir.push("deb-pkgs");

    {
        let dir_string = cache_dir.clone().into_os_string().into_string().unwrap();

        util::fs::create_dir(&dir_string);
        env::set_current_dir(&dir_string).unwrap();
    }

    let (system_distro, system_arch) = util::get_distro_arch_info();

    // Get the list of packages we need to build.
    let mut to_build: Vec<String> = vec![];

    for pkg in mpr_cache.packages().values() {
        // If the deb doesn't exist, we have to build.
        if !path::Path::new(&format!("{}.deb", pkg.pkgname)).exists() {
            to_build.push(pkg.pkgname.clone());
            continue;
        }

        let mut control_file_path = cache_dir.clone();
        control_file_path.push(&pkg.pkgname);
        control_file_path.push("DEBIAN");
        control_file_path.push("control");

        if path::Path::new(&control_file_path).exists() {
            let control_file =
                TagSection::new(&fs::read_to_string(&control_file_path).unwrap()).unwrap();

            // If the version in the control file matches the current MPR package's version,
            // then we don't need to update it.
            if control_file.get("Version").unwrap() == &pkg.version {
                continue;
            } else {
                to_build.push(pkg.pkgname.clone());
            }
        }
    }

    let num_of_packages = to_build.len();

    for (iter, pkg_string) in to_build.iter().enumerate() {
        let pkg = mpr_cache.packages().get(pkg_string).unwrap();

        // Generate the control file.
        let mut control_file_str = String::new();
        control_file_str.push_str(&format!("Package: {}\n", pkg.pkgname));
        control_file_str.push_str(&format!("Version: {}\n", pkg.version));
        control_file_str.push_str("Architecture: all\n");
        control_file_str
            .push_str("Description: Dummy description so 'dpkg-deb' doesn't complain.\n");

        let mut depends = vec![];
        let mut predepends = vec![];

        for dep_group in [
            pkg.get_system_depends(&system_distro, &system_arch),
            pkg.get_system_makedepends(&system_distro, &system_arch),
            pkg.get_system_checkdepends(&system_distro, &system_arch),
        ] {
            if let Some(mut deps) = dep_group {
                for dep in deps {
                    if let Some(no_prefix_string) = dep.strip_prefix("p!") {
                        predepends.push(no_prefix_string.to_string());
                    } else {
                        depends.push(dep);
                    }
                }
            }
        }

        if !depends.is_empty() {
            let mut depends_items = String::new();
            for dep in depends {
                depends_items.push_str(&SplitDependency::new(&dep).as_control());
                depends_items.push_str(", ");
            }
            depends_items.pop().unwrap();
            depends_items.pop().unwrap();

            control_file_str.push_str(&format!("Depends: {}\n", &depends_items));
        }

        if !predepends.is_empty() {
            let mut predepends_items = String::new();
            for predep in predepends {
                predepends_items.push_str(&SplitDependency::new(&predep).as_control());
                predepends_items.push_str(", ");
            }
            predepends_items.pop().unwrap();
            predepends_items.pop().unwrap();

            control_file_str.push_str(&format!("Pre-Depends: {}\n", &predepends_items));
        }

        if let Some(conflicts) = pkg.get_system_conflicts(&system_distro, &system_arch) {
            let mut conflicts_items = String::new();

            for conflict in conflicts {
                conflicts_items.push_str(&SplitDependency::new(&conflict).as_control());
                conflicts_items.push_str(", ");
            }
            conflicts_items.pop().unwrap();
            conflicts_items.pop().unwrap();

            control_file_str.push_str(&format!("Conflicts: {}\n", &conflicts_items));
        }

        // Write the control file.
        let control_file_dir = pkg.pkgname.clone() + "/DEBIAN";
        util::fs::create_dir(&control_file_dir);
        let mut control_file = util::fs::create_file(&(control_file_dir.clone() + "/control"));
        control_file
            .write(&control_file_str.as_bytes().to_vec())
            .unwrap();

        // Build the package.
        let clear_line = || {
            print!("\x1b[2K");
            io::stdout().flush().unwrap();
            print!("\x1b[0G");
            io::stdout().flush().unwrap();
        };
        clear_line();
        message::info(&format!(
            "[{}/{}] Processing MPR package '{}'...",
            iter + 1,
            num_of_packages,
            pkg.pkgname.bold().green(),
        ));
        io::stdout().flush().unwrap();

        let cmd = Command::new("dpkg-deb")
            .args(["-b", &pkg.pkgname])
            .output()
            .unwrap();
        if !cmd.status.success() {
            clear_line();
            message::error(&format!(
                "Failed to process MPR package '{}'. The package won't be available to install from the MPR.\n",
                pkg.pkgname.bold().green()
            ));
        }
    }

    println!();

    // Write the archive file.
    cache_dir.pop();
    cache_dir.push("cache.gz");
    fs::write(&cache_dir, resp).unwrap();
}
