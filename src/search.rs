use crate::{
    cache::{Cache, CachePackage, CachePackageSource, MprCache},
    message,
};
use ansi_term::{Colour, Style};
use chrono::{TimeZone, Utc};
use rust_apt::cache::Cache as AptCache;
use std::{collections::HashMap, fmt::Write};

pub fn search(args: &clap::ArgMatches) {
    let pkglist: Vec<&String> = args.get_many("pkg").unwrap().collect();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let apt_only = args.is_present("apt-only");
    let mpr_only = args.is_present("mpr-only");
    let apt_cache = AptCache::new();
    let mpr_cache = MprCache::new(mpr_url);
    let cache = Cache::new(&apt_cache, &mpr_cache);

    let mut matches: Vec<&String> = Vec::new();
    let package_map = cache.package_map();

    // Get matches.
    for pkgname in cache.get_unique_pkgnames() {
        // See if the package can be found in APT repos/the MPR if '--apt-only' or '--mpr-only'
        // were passed in.
        if ((apt_only && mpr_only)
            && !(cache.available_apt(&package_map, pkgname)
                && cache.available_mpr(&package_map, pkgname)))
            || (apt_only && !cache.available_apt(&package_map, pkgname))
            || (mpr_only && !cache.available_mpr(&package_map, pkgname))
        {
            continue;
        }

        // We don't care what source the package is from, we just want the pkgname.
        let pkg = package_map.get(pkgname).unwrap()[0];

        for arg in &pkglist {
            if pkg.pkgname.contains(arg.as_str()) {
                matches.push(&pkg.pkgname);
            }

            match &pkg.pkgdesc {
                Some(pkgdesc) => {
                    if pkgdesc.to_lowercase().contains(arg.as_str()) {
                        matches.push(&pkg.pkgname);
                    }
                }
                None => (),
            }
        }
    }

    matches.sort_unstable();
    matches.dedup();

    // Print matches.
    //
    // We'll be comparing this length against indexes, so subtract 1 so that it functions as if we
    // started at 0.
    // Also make sure to return if we got no matches, as we'll get an underflow then.
    let matches_length = matches.len();

    if matches_length == 0 {
        message::info("No results.");
        return;
    }

    matches.sort_by_key(|a| a.to_lowercase());

    let matches_length = matches_length - 1;
    let mut result = String::new();

    for (index, pkg) in matches.iter().enumerate() {
        result.push_str(&pkg_info(&package_map, pkg));
        if index < matches_length {
            result.push('\n');
        }
    }

    print!("{}", result);
}

pub fn pkg_info(package_map: &HashMap<&String, Vec<&CachePackage>>, pkg_str: &String) -> String {
    let mut result = String::new();

    // Get a list of sources for this package.
    let packages = package_map.get(pkg_str).unwrap();
    // Create a sources vector to concatenate into a colored array-like string later (i.e. '[APT, MPR]').
    let mut sources: Vec<&str> = Vec::new();
    let pkg;

    if packages.len() == 2 {
        sources.push("APT");
        sources.push("MPR");

        // If the APT version of the package is installed and matches the MPR version, the APT
        // version probably originated from the MPR, and we should use the MPR package version for
        // search results. If the APT version is installed and its version doesn't match, it was
        // probably installed from an APT repository and we should use the APT version. Otherwise,
        // it's just not installed and we can show the MPR version.
        let apt_pkg;
        let mpr_pkg;

        match packages[0].source {
            CachePackageSource::Apt => {
                apt_pkg = packages[0];
                mpr_pkg = packages[1];
            }
            _ => {
                apt_pkg = packages[1];
                mpr_pkg = packages[2];
            }
        };

        if apt_pkg.is_installed.unwrap() && apt_pkg.version == mpr_pkg.version {
            pkg = mpr_pkg;
        } else if apt_pkg.is_installed.unwrap() {
            pkg = apt_pkg;
        } else {
            pkg = mpr_pkg;
        };
    } else {
        pkg = packages[0];
        match pkg.source {
            CachePackageSource::Apt => sources.push("APT"),
            _ => sources.push("MPR"),
        }
    };

    let mut sources_str = String::from("[");

    for source in sources {
        write!(sources_str, "{}, ", Colour::Fixed(63).paint(source)).unwrap();
    }

    // Remove the trailing  ', ' at the end of the string. Then add the closing ']'.
    sources_str.pop();
    sources_str.pop();
    sources_str.push(']');

    // pkgname + version.
    writeln!(
        result,
        "{}/{} {}",
        Colour::Fixed(214).paint(pkg.pkgname.as_str()),
        pkg.version,
        sources_str
    )
    .unwrap();

    // pkgdesc.
    match &pkg.pkgdesc {
        Some(pkgdesc) => {
            writeln!(
                result,
                "{} {}",
                Style::new().bold().paint("Description:"),
                pkgdesc
            )
            .unwrap();
        }

        None => (),
    }

    // Maintainer.
    match &pkg.maintainer {
        Some(maintainer) => {
            writeln!(
                result,
                "{} {}",
                Style::new().bold().paint("Maintainer:"),
                maintainer
            )
            .unwrap();
        }

        None => (),
    }

    // Votes.
    match &pkg.num_votes {
        Some(num_votes) => writeln!(
            result,
            "{} {}",
            Style::new().bold().paint("Votes:"),
            num_votes
        )
        .unwrap(),

        None => (),
    }

    // Popularity.
    match &pkg.popularity {
        Some(popularity) => writeln!(
            result,
            "{} {}",
            Style::new().bold().paint("Popularity:"),
            popularity
        )
        .unwrap(),

        None => (),
    }

    // Out of date.
    if let CachePackageSource::Mpr = pkg.source {
        match &pkg.ood {
            Some(ood) => {
                let dt = Utc.timestamp(*ood as i64, 0).format("%Y-%m-%d").to_string();
                writeln!(
                    result,
                    "{} {}",
                    Style::new().bold().paint("Out of Date:"),
                    dt
                )
                .unwrap();
            }

            None => {
                writeln!(result, "{} N/A", Style::new().bold().paint("Out of Date:")).unwrap();
            }
        }
    }

    result
}
