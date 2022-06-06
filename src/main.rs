mod message;
mod mpr_cache;
mod search;
mod util;
mod whoami;

use clap::{self, Arg, Command};

#[quit::main]
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
        .subcommand(
            Command::new("search")
                .about("Search the MPR for a package")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("pkg")
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
        Some(("search", args)) => search::search(args),
        Some(("whoami", args)) => whoami::whoami(args),
        _                      => {},
    };
}
