use crate::{
    install_util, message,
    progress::{MistAcquireProgress, MistInstallProgress},
    style::Colorize,
    util,
};
use flate2::read::GzDecoder;
use rust_apt::{
    cache::{Cache as AptCache, PackageSort},
    package::Package,
    progress::{AcquireProgress, InstallProgress},
    tagfile::TagSection,
};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{collections::HashMap, env, fs, io};

///////////////////////////
// Stuff for MPR caches. //
///////////////////////////
#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub struct MprDependencyGroup {
    #[serde(rename = "Distro")]
    distro: Option<String>,
    #[serde(rename = "Arch")]
    arch: Option<String>,
    #[serde(rename = "Packages")]
    packages: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq)]
pub struct MprPackage {
    #[serde(rename = "Name")]
    pub pkgname: String,
    #[serde(rename = "PackageBase")]
    pub pkgbase: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Description")]
    pub pkgdesc: Option<String>,
    #[serde(rename = "Maintainer")]
    pub maintainer: Option<String>,
    #[serde(rename = "NumVotes")]
    pub num_votes: u32,
    #[serde(rename = "Popularity")]
    pub popularity: f32,
    #[serde(rename = "OutOfDate")]
    pub ood: Option<u32>,
    #[serde(rename = "Depends")]
    pub depends: Vec<MprDependencyGroup>,
    #[serde(rename = "MakeDepends")]
    pub makedepends: Vec<MprDependencyGroup>,
    #[serde(rename = "CheckDepends")]
    pub checkdepends: Vec<MprDependencyGroup>,
    #[serde(rename = "Conflicts")]
    pub conflicts: Vec<MprDependencyGroup>,
    #[serde(rename = "Provides")]
    pub provides: Vec<MprDependencyGroup>,
}

impl MprPackage {
    fn get_pkg_group(
        &self,
        distro: Option<&str>,
        arch: Option<&str>,
        dep_groups: &Vec<MprDependencyGroup>,
    ) -> Option<Vec<String>> {
        let distro = distro.map(|distro| distro.to_owned());
        let arch = arch.map(|arch| arch.to_owned());

        for dep_group in dep_groups {
            if dep_group.distro == distro && dep_group.arch == arch {
                return Some(dep_group.packages.clone());
            }
        }

        None
    }

    /// Get a list of depends packages for a specific distro/arch pair.
    pub fn get_depends(&self, distro: Option<&str>, arch: Option<&str>) -> Option<Vec<String>> {
        self.get_pkg_group(distro, arch, &self.depends)
    }

    /// Get a list of makedepends packages for a specific distro/arch pair.
    pub fn get_makedepends(&self, distro: Option<&str>, arch: Option<&str>) -> Option<Vec<String>> {
        self.get_pkg_group(distro, arch, &self.makedepends)
    }

    /// Get a list of checkdepends packages for a specific distro/arch pair.
    pub fn get_checkdepends(
        &self,
        distro: Option<&str>,
        arch: Option<&str>,
    ) -> Option<Vec<String>> {
        self.get_pkg_group(distro, arch, &self.checkdepends)
    }

    /// Get a list of conflicts packages for a specific distro/arch pair.
    pub fn get_conflicts(&self, distro: Option<&str>, arch: Option<&str>) -> Option<Vec<String>> {
        self.get_pkg_group(distro, arch, &self.conflicts)
    }

    /// Get a list of provides packages for a specific distro/arch pair.
    pub fn get_provides(&self, distro: Option<&str>, arch: Option<&str>) -> Option<Vec<String>> {
        self.get_pkg_group(distro, arch, &self.provides)
    }

    /// Get one of the above dependency vectors, looping through the order of
    /// specificity for distro-architecture variables used by makedeb.
    fn get_system_pkgs<F: Fn(&Self, Option<&str>, Option<&str>) -> Option<Vec<String>>>(
        &self,
        f: F,
        distro: &str,
        arch: &str,
    ) -> Option<Vec<String>> {
        if let Some(deps) = f(self, Some(distro), Some(arch)) {
            Some(deps)
        } else if let Some(deps) = f(self, Some(distro), None) {
            Some(deps)
        } else if let Some(deps) = f(self, None, Some(arch)) {
            Some(deps)
        } else {
            f(self, None, None)
        }
    }

    /// Get the `depends` values of this package, looping through the order of
    /// specificity for distro-architecture variables used by makedeb.
    pub fn get_system_depends(&self, distro: &str, arch: &str) -> Option<Vec<String>> {
        self.get_system_pkgs(Self::get_depends, distro, arch)
    }

    /// Get the `makedepends` values of this package, looping through the order
    /// of specificity for distro-architecture variables used by makedeb.
    pub fn get_system_makedepends(&self, distro: &str, arch: &str) -> Option<Vec<String>> {
        self.get_system_pkgs(Self::get_makedepends, distro, arch)
    }
    /// Get the `checkdepends` values of this package, looping through the order
    /// of specificity for distro-architecture variables used by makedeb.
    pub fn get_system_checkdepends(&self, distro: &str, arch: &str) -> Option<Vec<String>> {
        self.get_system_pkgs(Self::get_checkdepends, distro, arch)
    }

    /// Get the `conflicts` values of this package, looping through the order of
    /// specificity for distro-architecture variables used by makedeb.
    pub fn get_system_conflicts(&self, distro: &str, arch: &str) -> Option<Vec<String>> {
        self.get_system_pkgs(Self::get_conflicts, distro, arch)
    }

    /// Get the `provides` values of this package, looping through the order of
    /// specificity for distro-architecture variables used by makedeb.
    pub fn get_system_provides(&self, distro: &str, arch: &str) -> Option<Vec<String>> {
        self.get_system_pkgs(Self::get_provides, distro, arch)
    }
}

pub struct MprCache {
    packages: HashMap<String, MprPackage>,
}

impl MprCache {
    // Convert a Vector of MPR packages (the way they're stored on the MPR itself)
    // into a HashMap that's accessible via key-value pairs.
    fn vec_to_map(packages: Vec<MprPackage>) -> HashMap<String, MprPackage> {
        let mut map = HashMap::new();

        for pkg in packages {
            let pkgname = pkg.pkgname.clone();
            map.insert(pkgname, pkg);
        }

        map
    }

    pub fn validate_data(data: &[u8]) -> Result<Self, ()> {
        let mut file_gz = GzDecoder::new(data);
        let mut file_json = String::new();

        match file_gz.read_to_string(&mut file_json) {
            Ok(_) => (),
            Err(_) => return Err(()),
        }

        let cache = match serde_json::from_str::<Vec<MprPackage>>(&file_json) {
            Ok(json) => json,
            Err(_) => return Err(()),
        };

        Ok(Self {
            packages: Self::vec_to_map(cache),
        })
    }

    pub fn new() -> Self {
        // Try reading the cache file. If it doesn't exist or it's older than five
        // minutes, we have to update the cache file.
        let mut cache_file_path = util::xdg::get_global_cache_dir();
        cache_file_path.push("cache.gz");

        match fs::read(cache_file_path.clone()) {
            Ok(file) => match Self::validate_data(&file) {
                Ok(cache) => cache,
                Err(_) => {
                    message::error(&format!(
                        "There was an issue parsing the cache archive. Try running '{}'.\n",
                        "mist update".bold().green()
                    ));
                    quit::with_code(exitcode::UNAVAILABLE);
                }
            },
            Err(err) => {
                message::error(&format!(
                    "There was an issue reading the cache archive. Try running '{}' [{}].\n",
                    "mist update".bold().green(),
                    err.to_string().bold()
                ));
                quit::with_code(exitcode::UNAVAILABLE);
            }
        }
    }

    pub fn packages(&self) -> &HashMap<String, MprPackage> {
        &self.packages
    }
}

/////////////////////////////////////////////
// Stuff to handled shared APT/MPR caches. //
/////////////////////////////////////////////
//
// Some of these fields only make sense to one type of package, but this kind of
// cache allows us to combine both types when needed, such as when providing
// search results.

#[derive(Clone, PartialEq, Eq)]
pub enum CachePackageSource {
    Apt,
    Mpr,
}

#[derive(Clone, PartialEq)]
pub struct CachePackage {
    pub pkgname: String,
    pub pkgbase: Option<String>,
    pub version: String,
    pub pkgdesc: Option<String>,
    pub arch: Option<String>,
    pub maintainer: Option<String>,
    pub num_votes: Option<u32>,
    pub popularity: Option<f32>,
    pub ood: Option<u32>,
    pub source: CachePackageSource,
}

pub struct Cache {
    /// The underlying APT cache struct.
    apt_cache: AptCache,
    /// The underlying MPR cache struct.
    mpr_cache: MprCache,
    /// A combined list of all packages in the cache.
    //pkglist: Vec<CachePackage>,
    /// A map for getting all packages with a certain pkgname. Can be quicker
    /// than looping over [`Self::pkglist`].
    pkgmap: HashMap<String, Vec<CachePackage>>,
}

impl Cache {
    /// [`PackageSort::default`] isn't supposed to include virtual packages, but
    /// it appears to be doing so on some systems. This checks for virtual
    /// packages manually and excludes them.
    pub fn get_nonvirtual_packages<'a>(
        apt_cache: &'a AptCache,
        sort: &'a PackageSort,
    ) -> Vec<Package<'a>> {
        let mut vec = vec![];

        for pkg in apt_cache.packages(sort) {
            if pkg.candidate().is_some() {
                vec.push(pkg);
            }
        }

        vec
    }

    /// Create a new cache.
    pub fn new(apt_cache: AptCache, mpr_cache: MprCache) -> Self {
        // Package list.
        let mut pkglist = Vec::new();

        for pkg in Self::get_nonvirtual_packages(&apt_cache, &PackageSort::default()) {
            let candidate = pkg.candidate().unwrap();

            pkglist.push(CachePackage {
                pkgname: pkg.name(),
                pkgbase: None,
                version: candidate.version(),
                pkgdesc: candidate.summary(),
                arch: Some(pkg.arch()),
                maintainer: None,
                num_votes: None,
                popularity: None,
                ood: None,
                source: CachePackageSource::Apt,
            });
        }

        for pkg in mpr_cache.packages().values() {
            pkglist.push(CachePackage {
                pkgname: pkg.pkgname.clone(),
                pkgbase: Some(pkg.pkgbase.clone()),
                version: pkg.version.clone(),
                pkgdesc: pkg.pkgdesc.clone(),
                arch: None,
                maintainer: pkg.maintainer.clone(),
                num_votes: Some(pkg.num_votes),
                popularity: Some(pkg.popularity),
                ood: pkg.ood,
                source: CachePackageSource::Mpr,
            });
        }

        // Package map.
        let mut pkgmap: HashMap<String, Vec<CachePackage>> = HashMap::new();

        for pkg in &pkglist {
            let pkgname = pkg.pkgname.clone();

            #[allow(clippy::map_entry)]
            if pkgmap.contains_key(&pkgname) {
                pkgmap.get_mut(&pkgname).unwrap().push(pkg.clone());
            } else {
                pkgmap.insert(pkgname, vec![pkg.clone()]);
            }
        }

        Self {
            apt_cache,
            mpr_cache,
            //pkglist,
            pkgmap,
        }
    }

    /// Get a reference to the APT cache passed into this function.
    pub fn apt_cache(&self) -> &AptCache {
        &self.apt_cache
    }

    /// Get a reference to the MPR cache passed into this function.
    pub fn mpr_cache(&self) -> &MprCache {
        &self.mpr_cache
    }

    /// Run a transaction.
    /// `mpr_pkgs` is the list of MPR packages to install.
    pub fn commit(&self, mpr_pkgs: &Vec<Vec<String>>, mpr_url: &str) {
        let mut to_install: Vec<String> = Vec::new();
        let mut to_remove: Vec<String> = Vec::new();
        let mut to_purge: Vec<String> = Vec::new();
        let mut to_upgrade: Vec<String> = Vec::new();
        let mut to_downgrade: Vec<String> = Vec::new();

        // Report APT packages.
        for pkg in Self::get_nonvirtual_packages(&self.apt_cache, &PackageSort::default()) {
            let pkgname = pkg.name();
            let apt_string = format!("{}{}", "apt/".to_string().green(), &pkgname);

            if pkg.marked_install() {
                to_install.push(apt_string);
            } else if pkg.marked_delete() {
                to_remove.push(apt_string);
            } else if pkg.marked_purge() {
                to_purge.push(apt_string);
            } else if pkg.marked_upgrade() {
                to_upgrade.push(apt_string);
            } else if pkg.marked_downgrade() {
                to_downgrade.push(apt_string);
            }
        }

        // Report MPR packages.
        for pkg in mpr_pkgs.iter().flatten() {
            let mpr_string = format!("{}{}", "mpr/".to_owned().green(), pkg);
            to_install.push(mpr_string);
        }

        // Print out the transaction.
        if to_install.is_empty()
            && to_remove.is_empty()
            && to_purge.is_empty()
            && to_upgrade.is_empty()
            && to_downgrade.is_empty()
        {
            println!("{}", "Nothing to do, quitting.".bold());
            quit::with_code(exitcode::OK);
        };

        if !to_install.is_empty() {
            println!("{}", "The following packages will be installed:".bold());
            util::format_apt_pkglist(&to_install);
            println!();
        }

        if !to_remove.is_empty() {
            println!(
                "{}",
                format!("The following packages will be {}:", "removed".red()).bold()
            );
            util::format_apt_pkglist(&to_remove);
            println!();
        }

        if !to_purge.is_empty() {
            println!(
                "{}",
                format!(
                    "The following packages (along with their configuration files) will be {}:",
                    "removed".red()
                )
                .bold()
            );
            util::format_apt_pkglist(&to_purge);
            println!();
        }

        if !to_upgrade.is_empty() {
            println!("{}", "The following packages will be upgraded:".bold());
            util::format_apt_pkglist(&to_upgrade);
            println!();
        }

        if !to_downgrade.is_empty() {
            println!("{}", "The following packages will be downgraded:".bold());
            util::format_apt_pkglist(&to_downgrade);
            println!();
        }

        let (to_install_string, to_install_count) = {
            let count = to_install.len();
            let string = match count {
                0 => "install".green(),
                _ => "install".magenta(),
            };
            (string, count)
        };
        let (to_remove_string, to_remove_count) = {
            let count = to_remove.len();
            let string = match count {
                0 => "remove".green(),
                _ => "remove".magenta(),
            };
            (string, count)
        };
        let (to_upgrade_string, to_upgrade_count) = {
            let count = to_upgrade.len();
            let string = match count {
                0 => "upgrade".green(),
                _ => "upgrade".magenta(),
            };
            (string, count)
        };
        let (to_downgrade_string, to_downgrade_count) = {
            let count = to_downgrade.len();
            let string = match count {
                0 => "downgrade".green(),
                _ => "downgrade".magenta(),
            };
            (string, count)
        };

        println!("{}", "Review:".bold());

        println!(
            "{}",
            format!("- {} to {}", to_install_count, to_install_string).bold()
        );
        println!(
            "{}",
            format!("- {} to {}", to_remove_count, to_remove_string).bold()
        );
        println!(
            "{}",
            format!("- {} to {}", to_upgrade_count, to_upgrade_string).bold()
        );
        println!(
            "{}",
            format!("- {} to {}", to_downgrade_count, to_downgrade_string).bold()
        );

        print!("{}", "\nWould you like to continue? [Y/n] ".bold());
        io::stdout().flush().unwrap();

        let mut resp = String::new();
        io::stdin().read_line(&mut resp).unwrap();
        resp.pop();

        if !util::is_yes(&resp, true) {
            println!("{}", "Aborting...".bold());
            quit::with_code(exitcode::OK);
        }

        println!();
        // Clone MPR packages.
        //
        // We should be able to flatten the `mpr_pkgs` list to get this variable, but I
        // haven't gotten it to work yet. TODO: Make it work, duh.
        let mut flattened_pkgnames = vec![];
        let mut flattened_pkgbases = vec![];
        let mpr_pkgbases = install_util::pkgnames_to_pkgbases(self, mpr_pkgs);

        for vec in mpr_pkgs {
            for pkg in vec {
                flattened_pkgnames.push(pkg.as_str());
            }
        }

        for vec in &mpr_pkgbases {
            for pkg in vec {
                flattened_pkgbases.push(pkg.as_str());
            }
        }

        install_util::clone_mpr_pkgs(&flattened_pkgbases, mpr_url);

        // Review MPR packages.

        // Get the editor to review package files with.
        let editor = match edit::get_editor() {
            Ok(editor) => editor.into_os_string().into_string().unwrap(),
            Err(err) => {
                message::error(&format!(
                    "Couldn't find an editor to review package files with. [{}]\n",
                    err
                ));

                quit::with_code(exitcode::UNAVAILABLE);
            }
        };

        for pkg in flattened_pkgbases {
            println!();

            loop {
                message::question(&format!(
                    "Review files for '{}'? [Y/n] ",
                    pkg.bold().green()
                ));
                io::stdout().flush().unwrap();

                let mut resp = String::new();
                io::stdin().read_line(&mut resp).unwrap();
                resp.pop();

                if !util::is_yes(&resp, true) {
                    break;
                }

                let mut cache_dir = util::xdg::get_cache_dir();
                cache_dir.push("git-pkg");
                cache_dir.push(pkg);

                let files = {
                    let mut files = vec![];

                    let mut cmd = util::sudo::run_as_normal_user("git");
                    cmd.args(["ls-tree", "master", "--name-only"]);
                    let output = cmd.output().unwrap();
                    util::check_exit_status(&cmd, &output.status);

                    let string = std::str::from_utf8(&output.stdout).unwrap();

                    for file in string.lines() {
                        // There's no point in having the user review the '.SRCINFO' file.
                        if file == ".SRCINFO" {
                            continue;
                        }

                        files.push(file.to_string());
                    }

                    files
                };

                let mut cmd = util::sudo::run_as_normal_user(&editor);
                cmd.args(files);

                let status = cmd.spawn().unwrap().wait().unwrap();
                util::check_exit_status(&cmd, &status)
            }
        }

        // Install APT packages.
        let mut updater: Box<dyn AcquireProgress> = Box::new(MistAcquireProgress {});
        if self.apt_cache().get_archives(&mut updater).is_err() {
            message::error("Failed to fetch needed archives\n");
            quit::with_code(exitcode::UNAVAILABLE);
        }

        let mut installer: Box<dyn InstallProgress> = Box::new(MistInstallProgress {});
        if let Err(err) = self.apt_cache().do_install(&mut installer) {
            util::handle_errors(&err);
            quit::with_code(exitcode::UNAVAILABLE);
        }

        // If we're not installing any MPR packages, go ahead and quit.
        if mpr_pkgs.is_empty() {
            quit::with_code(exitcode::OK);
        }

        // Build and install MPR packages.
        let current_dir = env::current_dir().unwrap();
        let mut cache_dir = util::xdg::get_cache_dir();
        cache_dir.push("git-pkg");

        for pkg_group in mpr_pkgbases {
            let mut debs = vec![];
            // The list of packages to install; A Vector containing pkgname/version pairs.
            let mut install_list: Vec<[String; 2]> = vec![];

            for pkg in pkg_group {
                let mut git_dir = cache_dir.clone();
                git_dir.push(pkg.clone());
                env::set_current_dir(&git_dir).unwrap();

                // See this package has a control field value of 'MPR-Package'. If it does,
                // don't add it to our arg list. TODO: We need to add this key
                // to makedeb's .SRCINFO files.
                let mpr_package_field = {
                    let mut cmd = util::sudo::run_as_normal_user("bash");
                    cmd.arg("-c");
                    cmd.arg("source PKGBUILD; printf '%s\n' \"${control_fields[@]}\" | grep -q '^MPR-Package:'");
                    cmd.output().unwrap().status.success()
                };

                let mut cmd = util::sudo::run_as_normal_user("makedeb");

                if !mpr_package_field {
                    cmd.arg("-H");
                    cmd.arg("MPR-Package: yes");
                }

                message::info(&format!("Running makedeb for '{}'...\n", pkg.green()));
                if !cmd.spawn().unwrap().wait().unwrap().success() {
                    message::error("Failed to run makedeb.\n");
                    quit::with_code(exitcode::UNAVAILABLE);
                }

                // Get the list of '.deb' files that were built.
                for dir in fs::read_dir("./pkg").unwrap() {
                    let mut path = dir.unwrap().path();
                    path.push("DEBIAN");
                    path.push("control");
                    let control_file =
                        TagSection::new(&fs::read_to_string(&path).unwrap()).unwrap();

                    // Only add this deb for installation if the user asked for it to be installed.
                    let pkgname = control_file.get("Package").unwrap();
                    let version = control_file.get("Version").unwrap();
                    let arch = control_file.get("Architecture").unwrap();

                    if flattened_pkgnames.contains(&pkgname.as_str()) {
                        debs.push(format!(
                            "{}/{}_{}_{}.deb",
                            git_dir.display(),
                            pkgname,
                            version,
                            arch
                        ));
                    }

                    install_list.push([
                        pkgname.to_string(),
                        control_file.get("Version").unwrap().to_string(),
                    ]);
                }

                env::set_current_dir(&current_dir).unwrap();
            }

            // Convert the debs into the format required by the
            // [`rust_apt::cache::Cache::debs`] initializer.
            let mut debs_as_str = vec![];
            for deb in &debs {
                debs_as_str.push(deb.as_str());
            }

            // Install the packages.
            let deb_cache = AptCache::debs(&debs_as_str).unwrap();

            for pkg in &install_list {
                let cache_pkg = deb_cache.get(&pkg[0]).unwrap();
                let version = cache_pkg.get_version(&pkg[1]).unwrap();
                version.set_candidate();
                assert!(cache_pkg.mark_install(false, true));
                cache_pkg.protect();
            }

            if let Err(err) = deb_cache.resolve(true) {
                util::handle_errors(&err);
                quit::with_code(exitcode::UNAVAILABLE);
            }

            if deb_cache.get_archives(&mut updater).is_err() {
                message::error("Failed to fetch needed archives\n");
                quit::with_code(exitcode::UNAVAILABLE);
            }

            if let Err(err) = deb_cache.do_install(&mut installer) {
                util::handle_errors(&err);
                quit::with_code(exitcode::UNAVAILABLE);
            }
        }
    }

    /// Get a reference to the generated pkglist (contains a combined APT+MPR
    /// cache).
    /*pub fn pkglist(&self) -> &Vec<CachePackage> {
        &self.pkglist
    }*/

    /// Get a reference to the generated pkgmap (a key-value pair with keys of
    /// pkgnames and values of lists of packages). Can be quicker than
    /// [`Cache::pkglist`] if you're trying to lookup a package.
    pub fn pkgmap(&self) -> &HashMap<String, Vec<CachePackage>> {
        &self.pkgmap
    }

    // Get the APT variant of a package.
    pub fn get_apt_pkg(&self, pkgname: &str) -> Option<&CachePackage> {
        if let Some(pkglist) = self.pkgmap().get(&pkgname.to_owned()) {
            for pkg in pkglist {
                if let CachePackageSource::Apt = pkg.source {
                    return Some(pkg);
                }
            }
        }
        None
    }

    // Get the MPR variant of a package.
    pub fn get_mpr_pkg(&self, pkgname: &str) -> Option<&CachePackage> {
        if let Some(pkglist) = self.pkgmap().get(&pkgname.to_owned()) {
            for pkg in pkglist {
                if let CachePackageSource::Mpr = pkg.source {
                    return Some(pkg);
                }
            }
        }
        None
    }

    // Find the pkgbase of a given MPR package's pkgname.
    pub fn find_pkgbase(&self, pkgname: &str) -> Option<String> {
        for pkg in self.mpr_cache().packages().values() {
            if pkg.pkgname == pkgname {
                return Some(pkg.pkgbase.clone());
            }
        }
        None
    }
}
