pub use colored::Colorize;
use colored::CustomColor;
use lazy_static::lazy_static;

use chrono::{TimeZone, Utc};

use crate::{
    apt_util,
    cache::{Cache, CachePackage},
};
use std::{cmp::Ordering, fmt::Write};

lazy_static! {
    pub static ref UBUNTU_ORANGE: CustomColor = CustomColor::new(255, 175, 0);
    pub static ref UBUNTU_PURPLE: CustomColor = CustomColor::new(95, 95, 255);
}

/// Generate a colored package information entry.
/// If `name_only` is [`true`], the package name will be returned by itself.
pub fn generate_pkginfo_entry(
    pkg_group: &[CachePackage],
    cache: &Cache,
    name_only: bool,
) -> String {
    let pkgname = pkg_group.get(0).unwrap().pkgname.clone();

    if name_only {
        return pkgname;
    }

    // Set up the string we'll return at the end of the function.
    let mut return_string = String::new();

    // Fancy colored pkgname to the max! :OOOOOOOOOOOOOOOOOO
    write!(return_string, "{}", pkgname.custom_color(*UBUNTU_ORANGE)).unwrap();

    // Get the APT and MPR packages.
    let apt_pkg = cache.get_apt_pkg(&pkgname);
    let mpr_pkg = cache.get_mpr_pkg(&pkgname);

    // Get the package sources.
    let mut src_str = String::new();

    if apt_pkg.is_some() && mpr_pkg.is_some() {
        write!(
            src_str,
            "[{}, {}]",
            "APT".custom_color(*UBUNTU_PURPLE),
            "MPR".custom_color(*UBUNTU_PURPLE)
        )
        .unwrap();
    } else if apt_pkg.is_some() {
        write!(src_str, "[{}]", "APT".custom_color(*UBUNTU_PURPLE)).unwrap();
    } else if mpr_pkg.is_some() {
        write!(src_str, "[{}]", "MPR".custom_color(*UBUNTU_PURPLE)).unwrap();
    } else {
        unreachable!();
    }

    // Figure out what version and description to use, in this order:
    // 1. APT if installed
    // 2. MPR if present
    // 3. APT
    let pkgver: String;
    let pkgdesc: Option<String>;

    if let Some(apt_pkg_unwrapped) = apt_pkg && let Some(mpr_pkg_unwrapped) = mpr_pkg {
        if cache.apt_cache().get(&apt_pkg_unwrapped.pkgname).unwrap().is_installed() {
            pkgver = apt_pkg.unwrap().version.clone();
            pkgdesc = apt_pkg.unwrap().pkgdesc.clone();
        } else {
            let apt_pkgver = &apt_pkg_unwrapped.version;
            let mpr_pkgver = &mpr_pkg_unwrapped.version;

            match apt_util::cmp_versions(apt_pkgver, mpr_pkgver) {
                Ordering::Greater | Ordering::Equal => {
                    pkgver = apt_pkgver.clone();
                    pkgdesc = apt_pkg_unwrapped.pkgdesc.clone();
                }
                Ordering::Less => {
                    pkgver = mpr_pkgver.clone();
                    pkgdesc = mpr_pkg_unwrapped.pkgdesc.clone();
                }
            }
        }
    } else if let Some(mpr_pkg_unwrapped) = mpr_pkg {
        pkgver = mpr_pkg_unwrapped.version.clone();
        pkgdesc = mpr_pkg_unwrapped.pkgdesc.clone();
    } else {
        let apt_pkg_unwrapped = apt_pkg.unwrap();
        pkgver = apt_pkg_unwrapped.version.clone();
        pkgdesc = apt_pkg_unwrapped.pkgdesc.clone();
    }

    // Add the first line and description, at long last. This string is the one we'll return at the end of the function.
    write!(return_string, "/{} {}", pkgver, src_str).unwrap();

    if let Some(unwrapped_pkgdesc) = pkgdesc {
        write!(
            return_string,
            "\n{} {}",
            "Description:".bold(),
            unwrapped_pkgdesc
        )
        .unwrap();
    } else {
        write!(return_string, "\n{} N/A", "Description:".bold()).unwrap();
    }

    // If the MPR package exists, add some extra information about that.
    if let Some(pkg) = mpr_pkg {
        // Maintainer.
        if let Some(maintainer) = &pkg.maintainer {
            write!(return_string, "\n{} {}", "Maintainer:".bold(), maintainer).unwrap();
        }

        // Votes.
        write!(
            return_string,
            "\n{} {}",
            "Votes:".bold(),
            &pkg.num_votes.unwrap()
        )
        .unwrap();

        // Popularity.
        write!(
            return_string,
            "\n{} {}",
            "Popularity:".bold(),
            &pkg.popularity.unwrap()
        )
        .unwrap();

        // Out of Date.
        let ood_date: String;

        if let Some(ood_epoch) = pkg.ood {
            ood_date = Utc
                .timestamp(ood_epoch as i64, 0)
                .format("%Y-%m-%d")
                .to_string();
        } else {
            ood_date = "N/A".to_owned();
        }

        write!(return_string, "\n{} {}", "Popularity:".bold(), ood_date).unwrap();
    }

    return_string
}

pub fn generate_pkginfo_entries(
    pkgs: &Vec<&Vec<CachePackage>>,
    cache: &Cache,
    apt_only: bool,
    mpr_only: bool,
    installed_only: bool,
    name_only: bool,
) -> String {
    let mut matches = Vec::new();
    let mut result_string = String::new();

    for pkg_group in pkgs {
        let pkgname = &pkg_group.get(0).unwrap().pkgname;

        // APT only.
        if apt_only && cache.get_apt_pkg(pkgname).is_none() {
            continue;
        }

        // MPR only.
        if mpr_only && cache.get_mpr_pkg(pkgname).is_none() {
            continue;
        }

        // Installed only.
        if installed_only
            && cache.get_apt_pkg(pkgname).is_some()
            && !cache.apt_cache().get(pkgname).unwrap().is_installed()
        {
            continue;
        }

        // Package be passed all the tests bro. We's be adding it to the vector now.
        matches.push(pkg_group);
    }

    let matches_len = matches.len();

    for (index, pkg_group) in matches.iter().enumerate() {
        if name_only {
            result_string.push_str(&pkg_group.get(0).unwrap().pkgname);
            result_string.push('\n');
        } else if index == matches_len - 1 {
            result_string.push_str(&generate_pkginfo_entry(pkg_group, cache, name_only));
            result_string.push('\n');
        } else {
            result_string.push_str(&generate_pkginfo_entry(pkg_group, cache, name_only));
            result_string.push_str("\n\n");
        }
    }

    result_string
}
