use crate::{message, mpr_cache};
use ansi_term::{Colour, Style};
use chrono::{TimeZone, Utc};
use clap;

pub fn search(args: &clap::ArgMatches) -> () {
    let pkglist: Vec<&str> = args.values_of("pkg").unwrap().collect();
    let cache = mpr_cache::new();
    let mut matches: Vec<&mpr_cache::MprCache> = Vec::new();

    // Get matches.
    for pkg in &cache {
        for arg in &pkglist {
            if pkg.pkgname.contains(arg) && !matches.contains(&pkg) {
                matches.push(pkg);
            }

            match &pkg.pkgdesc {
                Some(pkgdesc) => {
                    if pkgdesc.to_lowercase().contains(arg) && !matches.contains(&pkg) {
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

    for (index, pkg) in matches.iter().enumerate() {
        println!(
            "{}/{}",
            Colour::Fixed(214).paint(pkg.pkgname.as_str()),
            pkg.version
        );

        match &pkg.pkgdesc {
            Some(pkgdesc) => println!("{} {}", Style::new().bold().paint("Description:"), pkgdesc),
            None => (),
        }

        match &pkg.maintainer {
            Some(maintainer) => println!(
                "{} {}",
                Style::new().bold().paint("Maintainer:"),
                maintainer
            ),
            None => (),
        }

        println!("{} {}", Style::new().bold().paint("Votes:"), &pkg.num_votes);
        println!(
            "{} {}",
            Style::new().bold().paint("Popularity:"),
            &pkg.popularity
        );

        match &pkg.ood {
            Some(ood) => {
                let dt = Utc.timestamp(*ood as i64, 0);
                println!("{} {}", Style::new().bold().paint("Out of Date:"), dt);
            }
            None => println!("{} N/A", Style::new().bold().paint("Out of Date:")),
        }

        if index < matches_length {
            println!();
        }
    }
}
