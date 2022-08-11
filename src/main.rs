mod cache;
mod clone;
mod color;
mod comment;
mod info;
mod list_comments;
mod message;
mod pkglist;
mod remove;
mod search;
mod update;
mod util;
mod whoami;

use clap::{self, Arg, Command, PossibleValue};

#[rustfmt::skip]
fn get_cli() -> Command<'static> {
    Command::new(clap::crate_name!())
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
            Command::new("comment")
                .arg_required_else_help(true)
                .about("Comment on a package page")
                .arg(
                    Arg::new("pkg")
                        .help("The package to comment on")
                        .required(true)
                        .takes_value(true)
                )
                .arg(
                    Arg::new("msg")
                        .help("The comment to post")
                        .short('m')
                        .long("msg")
                )
        )
        .subcommand(
            Command::new("info")
                .arg_required_else_help(true)
                .about("View information about an APT/MPR package")
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
            Command::new("list-comments")
                .arg_required_else_help(true)
                .about("List the comments on a package")
                .arg(
                    Arg::new("pkg")
                        .help("The package to view comments for")
                        .required(true)
                )
                .arg(
                    Arg::new("paging")
                        .help("When to send output to a pager")
                        .long("paging")
                        .takes_value(true)
                        .default_value("auto")
                        .value_parser([
                            PossibleValue::new("auto"),
                            PossibleValue::new("always"),
                            PossibleValue::new("never")
                        ])
                )
        )
        .subcommand(
            Command::new("pkglist")
                .hide(true)
        )
        .subcommand(
            Command::new("remove")
                .about("Remove packages from the system")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("pkg")
                        .help("The package(s) to remove")
                        .multiple_values(true)
                )
                .arg(
                    Arg::new("purge")
                        .help("Remove configuration files along with the package(s)")
                        .long("purge")
                )
                .arg(
                    Arg::new("autoremove")
                        .help("Automatically remove any unneeded packages")
                        .long("autoremove")
                )
        )
        .subcommand(
            Command::new("search")
                .about("Search for an APT/MPR package")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("pkg")
                        .required(true)
                        .help("The query to search for")
                        .multiple_values(true)
                )
                .arg(
                    Arg::new("apt-only")
                        .help("Filter results to packages available via APT")
                        .long("apt-only")
                )
                .arg(
                    Arg::new("mpr-only")
                        .help("Filter results to packages available on the MPR")
                        .long("mpr-only")
                )
        )
        .subcommand(
            Command::new("update")
                .about("Update the APT cache on the system")
        )
        .subcommand(
            Command::new("whoami")
                .about("Show the currently authenticated user")
        )
}

#[quit::main]
fn main() {
    let cmd_results = get_cli().get_matches();

    match cmd_results.subcommand() {
        Some(("clone", args)) => clone::clone(args),
        Some(("comment", args)) => comment::comment(args),
        Some(("info", args)) => info::info(args),
        Some(("list-comments", args)) => list_comments::list_comments(args),
        Some(("pkglist", args)) => pkglist::pkglist(args),
        Some(("remove", args)) => remove::remove(args),
        Some(("search", args)) => search::search(args),
        Some(("update", args)) => update::update(args),
        Some(("whoami", args)) => whoami::whoami(args),
        _ => {}
    };
}
