use crate::{message, mpr_cache};
use ansi_term::{Colour, Style};
use chrono::{TimeZone, Utc};
use rust_apt::cache::{Cache, PackageSort};
use std::fmt::Write;

pub fn search(args: &clap::ArgMatches) {
    let pkglist: Vec<&String> = args.get_many("pkg").unwrap().collect();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let cache = mpr_cache::new(mpr_url);
    let mut matches: Vec<&mpr_cache::MprCache> = Vec::new();

    // Get matches.
    for pkg in &cache {
        for arg in &pkglist {
            if pkg.pkgname.contains(arg.as_str()) && !matches.contains(&pkg) {
                matches.push(pkg);
            }

            match &pkg.pkgdesc {
                Some(pkgdesc) => {
                    if pkgdesc.to_lowercase().contains(arg.as_str()) && !matches.contains(&pkg) {
                        matches.push(pkg);
                    }
                }
                None => (),
            }
        }
    }

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

    matches.sort_by(|a, b| a.pkgname.to_lowercase().cmp(&b.pkgname.to_lowercase()));

    let matches_length = matches_length - 1;
    let mut result = String::new();

    for (index, pkg) in matches.iter().enumerate() {
        result.push_str(&pkg_info(pkg));

        if index < matches_length {
            result.push_str("\n\n");
        }
    }

    println!("{}", result);
}

pub fn pkg_info(pkg: &mpr_cache::MprCache) -> String {
    let mut result = String::new();
    writeln!(
        result,
        "{}/{}",
        Colour::Fixed(214).paint(pkg.pkgname.as_str()),
        pkg.version
    )
    .unwrap();
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

    writeln!(
        result,
        "{} {}",
        Style::new().bold().paint("Votes:"),
        &pkg.num_votes
    )
    .unwrap();

    writeln!(
        result,
        "{} {}",
        Style::new().bold().paint("Popularity:"),
        &pkg.popularity
    )
    .unwrap();

    match &pkg.ood {
        Some(ood) => {
            let dt = Utc.timestamp(*ood as i64, 0).format("%Y-%m-%d").to_string();
            write!(
                result,
                "{} {}",
                Style::new().bold().paint("Out of Date:"),
                dt
            )
            .unwrap();
        }

        None => {
            write!(result, "{} N/A", Style::new().bold().paint("Out of Date:")).unwrap();
        }
    }

    result
}
