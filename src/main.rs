mod message;
mod search;
mod request;
mod mpr_cache;
mod util;

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
                .global(true)
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
        .get_matches();

    match cmd.subcommand() {
        Some(("search", args)) => search::search(args),
        _                      => {},
    };
}
