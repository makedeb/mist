use crate::{message, mpr_cache};
use bat::{self, PrettyPrinter};
use chrono::{TimeZone, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
struct Comment {
    date: i64,
    msg: String,
    user: String,
}

pub fn list_comments(args: &clap::ArgMatches) {
    let pkg: &String = args.get_one("pkg").unwrap();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let paging = args.get_one::<String>("paging").unwrap().as_str();
    let cache = mpr_cache::new(mpr_url);

    let mut pkgnames: Vec<&String> = Vec::new();

    // Get a list of packages.
    for pkg in &cache {
        pkgnames.push(&pkg.pkgname);
    }

    // Abort if the package base doesn't exist.
    if !pkgnames.contains(&pkg) {
        message::error(&format!("Package '{}' doesn't exist on the MPR.", pkg));
        quit::with_code(exitcode::USAGE);
    }

    // Get package comments.
    let resp = match reqwest::blocking::get(format!("{}/api/list-comments/{}", mpr_url, pkg)) {
        Ok(resp) => resp,
        Err(err) => {
            message::error(&format!("Failed to make request. [{}]", err));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    };

    let resp_text = resp.text().unwrap();
    let resp_json = match serde_json::from_str::<Vec<Comment>>(&resp_text) {
        Ok(json) => json,
        Err(err) => {
            message::error(&format!("Failed to unpack response. [{}]", err));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    };

    // Generate a markdown string to show the user.
    let comments_len = resp_json.len() - 1; // We'll be using indexes to compare against this, so subtract 1.
    let mut comments_str = String::new();

    for (index, comment) in resp_json.iter().enumerate() {
        let date = Utc
            .timestamp(comment.date, 0)
            .format("%Y-%m-%d")
            .to_string();

        comments_str.push_str(&format!(
            "# Date: {}\n# Author: {}\n\n{}",
            date,
            comment.user,
            comment.msg.trim()
        ));

        if index < comments_len {
            comments_str.push_str("\n\n  --------------------\n\n");
        }
    }

    // Get the paging mode from the user.
    let paging_mode = match paging {
        "always" => bat::PagingMode::Always,
        "never" => bat::PagingMode::Never,
        &_ => bat::PagingMode::QuitIfOneScreen,
    };

    PrettyPrinter::new()
        .input_from_bytes(comments_str.as_bytes())
        .language("md")
        .paging_mode(paging_mode)
        .print()
        .unwrap();
}
