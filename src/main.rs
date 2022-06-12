mod clone;
mod info;
mod message;
mod mpr_cache;
mod search;
mod util;
mod whoami;

use clap::{self, Arg, Command};

#[quit::main]
#[rustfmt::skip]
fn main() {
    let cmd = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg_required_else_help(true)
        .arg(
            Arg::new("token")
                .help("The API token to authenticate to the MPR with")
                .long("token")
                .env("MPR_TOKEN")
                .hide_env_values(true)
                .global(true)
                .takes_value(true)
        )
        .arg(
            Arg::new("mpr-url")
            .help("URL to access the MPR from")
            .long("mpr-url")
            .env("MPR_URL")
            .hide_env_values(true)
            .global(true)
            .takes_value(true)
            .default_value("https://mpr.makedeb.org")
            )
        .subcommand(
            Command::new("clone")
                .about("Clone a package base from the MPR")
                .arg(
                    Arg::new("pkg")
                        .help("The package to clone")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("info")
                .arg_required_else_help(true)
                .about("View information about a package")
                .arg(
                    Arg::new("pkg")
                    .help("The package to view")
                    .required(true)
                )
                .arg(
                    Arg::new("web")
                    .help("Open the page for the package in a web browser")
                    .short('w')
                    .long("web")
                )
        )
        .subcommand(
            Command::new("search")
                .about("Search the MPR for a package")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("pkg")
                        .required(true)
                        .help("The query to search for")
                        .multiple_values(true)
                ),
        )
        .subcommand(
            Command::new("whoami")
                .about("Show the currently authenticated user")
        )
        .get_matches();

    match cmd.subcommand() {
        Some(("clone", args)) => clone::clone(args),
        Some(("info", args)) => info::info(args),
        Some(("search", args)) => search::search(args),
        Some(("whoami", args)) => whoami::whoami(args),
        _                      => {},
    };
}
