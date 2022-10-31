pub use colored::Colorize;
use colored::CustomColor;
use lazy_static::lazy_static;

use chrono::{TimeZone, Utc};

use crate::cache::Cache;
use std::fmt::Write;

lazy_static! {
    pub static ref UBUNTU_ORANGE: CustomColor = CustomColor::new(255, 175, 0);
    pub static ref UBUNTU_PURPLE: CustomColor = CustomColor::new(95, 95, 255);
}

/// Generate a colored package information entry.
/// If `name_only` is [`true`], the package name will be returned by itself.
pub fn generate_pkginfo_entry(pkgname: &str, cache: &Cache, name_only: bool) -> String {
    if name_only {
        return pkgname.to_string();
    }

    // Set up the string we'll return at the end of the function.
    let mut return_string = String::new();

    // Fancy colored pkgname to the max! :OOOOOOOOOOOOOOOOOO
    write!(return_string, "{}", pkgname.custom_color(*UBUNTU_ORANGE)).unwrap();

    // Get the APT and MPR packages.
    let apt_pkg = cache.apt_cache().get(pkgname);
    let mpr_pkg = cache.mpr_cache().packages().get(pkgname);

    // Get the package sources.
    let mut src_str = String::new();
    {
        let mut sources = vec![];

        if apt_pkg.is_some() {
            sources.push("APT".custom_color(*UBUNTU_PURPLE));
        }
        if mpr_pkg.is_some() {
            sources.push("MPR".custom_color(*UBUNTU_PURPLE));
        }

        let mut sources_str = String::new();

        for src in sources {
            sources_str.push_str(&format!("{}, ", src));
        }

        sources_str.pop().unwrap();
        sources_str.pop().unwrap();

        write!(src_str, "[{}]", sources_str).unwrap();
    }

    // Figure out what version and description to use, in this order:
    // 1. APT if installed
    // 2. MPR if present
    // 3. APT
    let pkgver: String;
    let pkgdesc: Option<String>;

    if let Some(apt_pkg_unwrapped) = &apt_pkg && apt_pkg_unwrapped.is_installed() {
        let version = apt_pkg_unwrapped.candidate().unwrap();
        pkgver = version.version();
        pkgdesc = version.description();
    } else if let Some(mpr_pkg_unwrapped) = mpr_pkg {
        pkgver = mpr_pkg_unwrapped.version.to_string();
        pkgdesc = mpr_pkg_unwrapped.pkgdesc.clone();
    } else if let Some(apt_pkg_unwrapped) = &apt_pkg {
        let version = apt_pkg_unwrapped.candidate().unwrap();
        pkgver = version.version();
        pkgdesc = version.description();
    } else {
        unreachable!();
    }

    // Add the first line and description, at long last. This string is the one
    // we'll return at the end of the function.
    write!(return_string, "/{} {}", pkgver, src_str).unwrap();
    write!(
        return_string,
        "\n{} {}",
        "Description:".bold(),
        pkgdesc.unwrap_or_else(|| "N/A".to_string())
    )
    .unwrap();

    // If the MPR package exists, add some extra information about that.
    if let Some(mpr_pkg_unwrapped) = mpr_pkg {
        // Maintainer.
        if let Some(maintainer) = &mpr_pkg_unwrapped.maintainer {
            write!(return_string, "\n{} {}", "Maintainer:".bold(), maintainer).unwrap();
        }

        // Votes.
        write!(
            return_string,
            "\n{} {}",
            "Votes:".bold(),
            &mpr_pkg_unwrapped.num_votes
        )
        .unwrap();

        // Popularity.
        write!(
            return_string,
            "\n{} {}",
            "Popularity:".bold(),
            &mpr_pkg_unwrapped.popularity
        )
        .unwrap();

        // Out of Date.
        let ood_date: String;

        if let Some(ood_epoch) = mpr_pkg_unwrapped.ood {
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

pub fn generate_pkginfo_entries<T: AsRef<str>>(
    pkgs: &[T],
    cache: &Cache,
    apt_only: bool,
    mpr_only: bool,
    installed_only: bool,
    name_only: bool,
) -> String {
    let mut matches = Vec::new();
    let mut result_string = String::new();

    for pkg in pkgs {
        let pkgname = pkg.as_ref();

        // APT only.
        if apt_only && cache.apt_cache().get(pkgname).is_none() {
            continue;
        }

        // MPR only.
        if mpr_only && cache.mpr_cache().packages().get(pkgname).is_none() {
            continue;
        }

        // Installed only.
        if installed_only
            && let Some(pkg) = cache.apt_cache().get(pkgname)
            && !pkg.is_installed()
        {
            continue;
        } else if cache.apt_cache().get(pkgname).is_none() {
            continue;
        }

        // Package be passed all the tests bro. We's be adding it to the vector now.
        matches.push(pkgname);
    }

    let matches_len = matches.len();

    for (index, pkgname) in matches.iter().enumerate() {
        if name_only {
            result_string.push_str(pkgname);
            result_string.push('\n');
        } else if index == matches_len - 1 {
            result_string.push_str(&generate_pkginfo_entry(pkgname, cache, name_only));
            result_string.push('\n');
        } else {
            result_string.push_str(&generate_pkginfo_entry(pkgname, cache, name_only));
            result_string.push_str("\n\n");
        }
    }

    result_string
}
