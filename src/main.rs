mod cache;
mod clone;
mod comment;
mod install;
mod install_util;
mod list;
mod list_comments;
mod message;
mod progress;
mod quick_list;
mod remove;
mod search;
mod style;
mod update;
mod util;
mod whoami;

pub use rust_apt::util as apt_util;

use clap::{self, Arg, Command, PossibleValue};

#[rustfmt::skip]
fn get_cli() -> Command<'static> {
    // Common arguments used in multiple commands.
    let token_arg = Arg::new("token")
        .help("The API token to authenticate to the MPR with")
        .long("token")
        .env("MPR_TOKEN")
        .hide_env_values(true)
        .takes_value(true)
        .required(true);

    let mpr_url_arg = Arg::new("mpr-url")
        .help("URL to access the MPR from")
        .long("mpr-url")
        .env("MPR_URL")
        .hide_env_values(true)
        .takes_value(true)
        .default_value("https://mpr.makedeb.org");

    let mpr_only_arg = Arg::new("mpr-only")
        .help("Filter results to packages available on the MPR")
        .long("mpr-only");
    
    let apt_only_arg = Arg::new("apt-only")
        .help("Filter results to packages available via APT")
        .long("apt-only");

    let installed_only_arg = Arg::new("installed-only")
        .help("Filter results to installed packages")
        .short('i')
        .long("installed");
    
    let name_only_arg = Arg::new("name-only")
        .help("Output the package's name without any extra details")
        .long("name-only");

    // The CLI.
    Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg_required_else_help(true)
        .subcommand(
            Command::new("clone")
                .about("Clone a package base from the MPR")
                .arg(
                    Arg::new("pkg")
                        .help("The package to clone")
                        .required(true)
                )
                .arg(mpr_url_arg.clone())
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
                .arg(token_arg.clone())
                .arg(mpr_url_arg.clone())
        )
        .subcommand(
            Command::new("install")
            .about("Install packages from APT and the MPR")
            .arg(
                Arg::new("pkg")
                .help("The package(s) to install")
                .multiple_values(true)
                .required(true)
            )
            .arg(mpr_url_arg.clone())
        )
        .subcommand(
            Command::new("list")
            .about("List packages available via APT and the MPR")
            .arg(
                Arg::new("pkg")
                .help("The package(s) to get information for")
                .multiple_values(true)
            )
            .arg(mpr_only_arg.clone())
            .arg(apt_only_arg.clone())
            .arg(installed_only_arg.clone())
            .arg(name_only_arg.clone())
            .arg(mpr_url_arg.clone())
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
                .arg(mpr_url_arg.clone())
        )
        .subcommand(
            Command::new("quick-list")
                .about("List available packages quickly for shell completions")
                .hide(true)
                .arg(
                    Arg::new("prefix")
                        .help("The prefix to limit output to")
                        .required(true)
                )
                .arg(mpr_only_arg.clone().conflicts_with("mpr-only"))
                .arg(apt_only_arg.clone().conflicts_with("apt-only"))
                .arg(mpr_url_arg.clone())
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
                .arg(mpr_url_arg.clone().hide(true))
        )
        .subcommand(
            Command::new("search")
                .about("Search for an APT/MPR package")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("query")
                        .required(true)
                        .help("The query to search for")
                        .multiple_values(true)
                )
                .arg(mpr_only_arg.clone())
                .arg(apt_only_arg.clone())
                .arg(installed_only_arg.clone())
                .arg(name_only_arg.clone())
                .arg(mpr_url_arg.clone())
        )
        .subcommand(
            Command::new("update")
                .about("Update the APT cache on the system")
        )
        .subcommand(
            Command::new("whoami")
                .about("Show the currently authenticated user")
                .arg(token_arg.clone())
                .arg(mpr_url_arg.clone())
        )
}

#[quit::main]
fn main() {
    let cmd_results = get_cli().get_matches();

    match cmd_results.subcommand() {
        Some(("clone", args)) => clone::clone(args),
        Some(("comment", args)) => comment::comment(args),
        Some(("install", args)) => install::install(args),
        Some(("list", args)) => list::list(args),
        Some(("list-comments", args)) => list_comments::list_comments(args),
        Some(("quick-list", args)) => quick_list::quick_list(args),
        Some(("remove", args)) => remove::remove(args),
        Some(("search", args)) => search::search(args),
        Some(("update", args)) => update::update(args),
        Some(("whoami", args)) => whoami::whoami(args),
        _ => unreachable!(),
    };
}
