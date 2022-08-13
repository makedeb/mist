use crate::{
    color::Colorize,
    message,
    progress::{MistAcquireProgress, MistInstallProgress},
    util,
};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use regex::Regex;
use rust_apt::{
    cache::{Cache as AptCache, PackageSort},
    pkgmanager::OrderResult,
    progress::{InstallProgress, UpdateProgress},
};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{collections::HashMap, fs, io, time::SystemTime};

///////////////////////////
// Stuff for MPR caches. //
///////////////////////////
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
}

pub struct MprCache {
    pub packages: Vec<MprPackage>,
}

impl MprCache {
    pub fn new(mpr_url: &str) -> MprCache {
        // Get the XDG cache directory.
        let cache_dir = match dirs::cache_dir() {
            Some(dir) => dir,
            None => {
                message::error("Unable to find the xdg cache directory.");
                quit::with_code(exitcode::UNAVAILABLE);
            }
        };

        // Make sure the directory exists.
        let mut mpr_cache_dir = cache_dir;
        mpr_cache_dir.push("mpr-cli");

        if !mpr_cache_dir.exists() {
            match fs::create_dir_all(mpr_cache_dir.clone()) {
                Ok(()) => (),
                Err(err) => {
                    message::error(&format!(
                        "Encountered an unknown error while creating the cache directory. [{}]",
                        err
                    ));
                    quit::with_code(exitcode::UNAVAILABLE);
                }
            }
        } else if !mpr_cache_dir.is_dir() {
            message::error(&format!(
                "Cache path '{}' isn't a directory.",
                mpr_cache_dir.display()
            ));
            quit::with_code(exitcode::OSERR);
        }

        // Try reading the cache file. If it doesn't exist or it's older than five minutes, we have to
        // update the cache file.
        let mut mpr_cache_file = mpr_cache_dir;
        mpr_cache_file.push("cache.gz");

        let mut update_cache = false;

        match fs::metadata(mpr_cache_file.clone()) {
            // The file exists. Make sure it's been updated in the last five minutes.
            Ok(metadata) => {
                let five_minutes = 60 * 5; // The MPR updates package archives every five minutes.
                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let file_last_modified = metadata
                    .modified()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if (current_time - file_last_modified) > five_minutes {
                    update_cache = true;
                };
            }
            // The file doesn't exist. We need to create it.
            Err(err) => {
                if err.raw_os_error().unwrap() != 2 {
                    message::error(&format!(
                        "Encountered an unknown error while reading cache. [{}]",
                        err
                    ));
                    quit::with_code(exitcode::OSFILE);
                } else {
                    update_cache = true;

                    match fs::File::create(mpr_cache_file.clone()) {
                        Ok(_) => (),
                        Err(err) => {
                            message::error(&format!(
                                "Encountered an unknown error while reading cache. [{}]",
                                err
                            ));
                            quit::with_code(exitcode::OSFILE);
                        }
                    }
                }
            }
        };

        // If we need to, update the cache file.
        if update_cache {
            // Download the archive.
            let resp =
                match reqwest::blocking::get(format!("{}/packages-meta-ext-v2.json.gz", mpr_url)) {
                    Ok(resp) => resp,
                    Err(err) => {
                        message::error(&format!("Unable to make request. [{}]", err));
                        quit::with_code(exitcode::UNAVAILABLE);
                    }
                };

            if !resp.status().is_success() {
                message::error(&format!(
                    "Failed to download package archive from the MPR. [{}]",
                    resp.status()
                ));
                quit::with_code(exitcode::TEMPFAIL);
            }

            // Decompress the archive.
            let cache = match valid_archive(resp) {
                Ok(cache) => cache,
                Err(num) => {
                    if num == 1 {
                        message::error("Failed to decompress package archive from the MPR.");
                        quit::with_code(exitcode::TEMPFAIL);
                    } else {
                        message::error(
                            "Failed to verify integrity of package archive from the MPR.",
                        );
                        quit::with_code(exitcode::TEMPFAIL);
                    }
                }
            };

            // Now that the JSON has been verified, let's write out the archive to the cache file.
            let mut config_compressor = GzEncoder::new(Vec::new(), Compression::default());
            config_compressor
                .write_all(serde_json::to_string(&cache).unwrap().as_bytes())
                .unwrap();
            let config_gz = config_compressor.finish().unwrap();

            match fs::write(mpr_cache_file, config_gz) {
                Ok(()) => (),
                Err(err) => {
                    message::error(&format!(
                        "Failed to write updated package archive. [{}]",
                        err
                    ));
                    quit::with_code(exitcode::IOERR);
                }
            }

            // Return the new cache object.
            MprCache { packages: cache }
        } else {
            // The cache is less than 5 minutes old. We still need to validate that the cache is valid
            // though.
            let cache_file = match fs::File::open(mpr_cache_file.clone()) {
                Ok(file) => file,
                Err(err) => {
                    message::error(&format!(
                        "Failed to write updated package archive. [{}]",
                        err
                    ));
                    quit::with_code(exitcode::IOERR);
                }
            };

            match valid_archive(cache_file) {
                Ok(file) => MprCache { packages: file },
                Err(_) => {
                    // On an error, let's just remove the cache file and regenerate it by recalling
                    // this function.
                    fs::remove_file(mpr_cache_file).unwrap();
                    self::MprCache::new(mpr_url)
                }
            }
        }
    }
}

fn valid_archive(file: impl Read) -> Result<Vec<MprPackage>, u32> {
    let mut resp_gz = GzDecoder::new(file);
    let mut resp_json = String::new();

    match resp_gz.read_to_string(&mut resp_json) {
        Ok(_) => (),
        Err(_) => return Err(1),
    }

    // Feed the JSON into our struct.
    let cache = match serde_json::from_str::<Vec<MprPackage>>(&resp_json) {
        Ok(json) => json,
        Err(_) => return Err(2),
    };

    Ok(cache)
}

/////////////////////////////////////////////
// Stuff to handled shared APT/MPR caches. //
/////////////////////////////////////////////
//
// Some of these fields only make sense to one type of package, but this kind of cache allows us to
// combine both types when needed, such as when providing search results.

#[derive(PartialEq)]
pub enum CachePackageSource {
    Apt,
    Mpr,
}

#[derive(PartialEq)]
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
    pub current_state: Option<u8>,
    pub source: CachePackageSource,
    pub is_installed: Option<bool>,
}

pub struct Cache {
    pub packages: Vec<CachePackage>,
    _initialized: bool,
}

// Create a new cache.
impl Cache {
    pub fn new(apt_cache: &AptCache, mpr_cache: &MprCache) -> Self {
        let mut packages: Vec<CachePackage> = Vec::new();

        // Add APT packages.
        let re = Regex::new(r":.*$").unwrap();

        for pkg in apt_cache.packages(&PackageSort::default().names()) {
            // Foreign architecture have ':{arch}' appended to the package name, but we don't want
            // that since pkg.arch() contains that needed information anyway.
            let pkgname = re.replace(&pkg.name(), "").to_string();
            let version = pkg.candidate().unwrap();

            packages.push(CachePackage {
                pkgname,
                pkgbase: None,
                version: version.version(),
                pkgdesc: Some(version.summary()),
                arch: Some(version.arch()),
                maintainer: None,
                num_votes: None,
                popularity: None,
                ood: None,
                current_state: Some(pkg.current_state()),
                is_installed: Some(pkg.is_installed()),
                source: CachePackageSource::Apt,
            });
        }

        // Add MPR packages.
        for pkg in &mpr_cache.packages {
            packages.push(CachePackage {
                pkgname: pkg.pkgname.clone(),
                pkgbase: Some(pkg.pkgbase.clone()),
                version: pkg.version.clone(),
                pkgdesc: pkg.pkgdesc.clone(),
                arch: None,
                maintainer: pkg.maintainer.clone(),
                num_votes: Some(pkg.num_votes),
                popularity: Some(pkg.popularity),
                ood: pkg.ood,
                current_state: None,
                is_installed: None,
                source: CachePackageSource::Mpr,
            });
        }

        Cache {
            packages,
            _initialized: true,
        }
    }

    // Get a list of unique pkgnames - if a package exists in both APT repos and the MPR, they'll
    // be duplicated in the 'Cache.packages' list otherwise.
    pub fn get_unique_pkgnames(&self) -> Vec<&String> {
        let mut packages: Vec<&String> = Vec::new();

        for pkg in &self.packages {
            packages.push(&pkg.pkgname);
        }

        packages.sort_unstable();
        packages.dedup();
        packages
    }

    // Get a list of CachePackage objects that matche a certain pkgname.
    pub fn package_map(&self) -> HashMap<&String, Vec<&CachePackage>> {
        let mut packages: HashMap<&String, Vec<&CachePackage>> = HashMap::new();

        for pkg in &self.packages {
            match packages.get_mut(&&pkg.pkgname) {
                Some(vec) => vec.push(pkg),
                None => {
                    packages.insert(&pkg.pkgname, vec![pkg]);
                }
            }
        }

        packages
    }

    // See if a package is available via APT.
    // package_map is available from the package_map() function above.
    pub fn available_apt(
        &self,
        package_map: &HashMap<&String, Vec<&CachePackage>>,
        pkgname: &String,
    ) -> bool {
        match package_map.get(pkgname) {
            Some(packages) => {
                for pkg in packages {
                    match pkg.source {
                        CachePackageSource::Apt => return true,
                        _ => continue,
                    }
                }

                false
            }
            None => false,
        }
    }

    // See if a package is available on the MPR.
    // package_map is available from the package_map() function above.
    pub fn available_mpr(
        &self,
        package_map: &HashMap<&String, Vec<&CachePackage>>,
        pkgname: &String,
    ) -> bool {
        match package_map.get(pkgname) {
            Some(packages) => {
                for pkg in packages {
                    match pkg.source {
                        CachePackageSource::Mpr => return true,
                        _ => continue,
                    }
                }

                false
            }
            None => false,
        }
    }
}

// Run a transaction using our progress reporting.
pub fn run_transaction(cache: &AptCache, purge: bool) {
    let mut to_install: Vec<String> = Vec::new();
    let mut to_remove: Vec<String> = Vec::new();
    let mut to_upgrade: Vec<String> = Vec::new();
    let mut to_downgrade: Vec<String> = Vec::new();

    for pkg in cache.packages(&PackageSort::default()) {
        let pkgname = pkg.name();

        if pkg.marked_install() {
            to_install.push(pkgname);
        } else if pkg.marked_delete() {
            to_remove.push(pkgname);
        } else if pkg.marked_downgrade() {
            to_downgrade.push(pkgname);
        } else if pkg.marked_upgrade() {
            to_upgrade.push(pkgname);
        }
    }

    // Print out the transaction to the user.
    let is_to_install = !to_install.is_empty();
    let is_to_remove = !to_remove.is_empty();
    let is_to_upgrade = !to_upgrade.is_empty();
    let is_to_downgrade = !to_downgrade.is_empty();

    if is_to_install {
        println!("{}", "The following packages will be installed:".bold());
        util::format_apt_pkglist(&to_install);
        println!();
    }

    if is_to_remove {
        if purge {
            println!(
                "{}",
                "The following packages (and their configuration files) will be removed:".bold()
            );
        } else {
            println!("{}", "The following packages will be removed:".bold());
        }

        util::format_apt_pkglist(&to_remove);
        println!();
    }

    if is_to_upgrade {
        println!("{}", "The following packages will be upgraded:".bold());
        util::format_apt_pkglist(&to_upgrade);
        println!();
    }

    if is_to_downgrade {
        println!("{}", "The following packages will be DOWNGRADED:".bold());
        util::format_apt_pkglist(&to_downgrade);
        println!();
    }

    if vec![is_to_install, is_to_remove, is_to_upgrade, is_to_downgrade].contains(&true) {
        let to_install_string = match is_to_install {
            true => "install".purple(),
            false => "install".green(),
        };
        let to_remove_string = match is_to_remove {
            true => "remove".purple(),
            false => "remove".green(),
        };
        let to_upgrade_string = match is_to_upgrade {
            true => "upgrade".purple(),
            false => "upgrade".green(),
        };
        let to_downgrade_string = match is_to_downgrade {
            true => "downgrade".purple(),
            false => "downgrade".green(),
        };

        println!("{}", "Review:".bold());

        println!(
            "{}",
            format!(
                "- {} to {}",
                to_install.len().to_string().blue(),
                to_install_string,
            )
            .bold()
        );
        println!(
            "{}",
            format!(
                "- {} to {}",
                to_remove.len().to_string().blue(),
                to_remove_string
            )
            .bold()
        );
        println!(
            "{}",
            format!(
                "- {} to {}",
                to_upgrade.len().to_string().blue(),
                to_upgrade_string
            )
            .bold()
        );
        println!(
            "{}",
            format!(
                "- {} to {}",
                to_downgrade.len().to_string().blue(),
                to_downgrade_string
            )
            .bold()
        );
    } else {
        println!("{}", "Nothing found to do, quitting.".bold());
        quit::with_code(exitcode::OK);
    }

    print!("{}", "\nWould you like to continue? [Y/n] ".bold());
    io::stdout().flush().unwrap();

    let mut resp = String::new();
    io::stdin().read_line(&mut resp).unwrap();
    resp.pop();

    if !util::is_yes(&resp, true) {
        println!("{}", "Aborting...".bold());
    }

    let mut updater: Box<dyn UpdateProgress> = Box::new(MistAcquireProgress {});
    if let Err(_) = cache.get_archives(&mut cache.records.borrow_mut(), &mut updater) {
        message::error("Failed to fetch needed archives.");
        quit::with_code(exitcode::UNAVAILABLE);
    }

    let mut installer: Box<dyn InstallProgress> = Box::new(MistInstallProgress {});
    match cache.do_install(&mut installer) {
        OrderResult::Completed => (),
        OrderResult::Incomplete => {
            unimplemented!("`cache.do_install()` returned `OrderResult::Incomplete`. Please report this as an issue.");
        }
        OrderResult::Failed => {
            message::error("There was an issue running the transaction.");
            quit::with_code(exitcode::UNAVAILABLE);
        }
    }
}
