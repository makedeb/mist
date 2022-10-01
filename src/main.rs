#![feature(let_chains)]
mod cache;
mod clone;
mod comment;
mod install;
mod install_util;
mod list;
mod list_comments;
mod message;
mod progress;
mod remove;
mod search;
mod style;
mod update;
mod upgrade;
mod util;
mod whoami;

use clap::{self, Arg, Command, PossibleValue};
pub use rust_apt::util as apt_util;
use std::{
    env,
    fs::File,
    os::{linux::fs::MetadataExt, unix::fs::PermissionsExt},
};
use style::Colorize;
use which::which;

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
        )
        .subcommand(
            Command::new("update")
                .about("Update the APT cache on the system")
                .arg(mpr_url_arg.clone())
        )
        .subcommand(
            Command::new("upgrade")
                .about("Upgrade the packages on the system")
                .arg(Arg::new("apt-only").help("Only upgrade APT packages").long("apt-only").conflicts_with("mpr-only"))
                .arg(Arg::new("mpr-only").help("Only upgrade MPR packages").long("mpr-only").conflicts_with("apt-only"))
                .arg(mpr_url_arg.clone())
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

    // Make sure that this executable has the `setuid` flag set and is owned by
    // root. Parts of this program (intentionally) expect such behavior.
    let cmd_name = {
        let cmd = env::args().collect::<Vec<String>>().remove(0);
        if cmd.contains('/') {
            cmd
        } else {
            which(cmd).unwrap().into_os_string().into_string().unwrap()
        }
    };

    let cmd_metadata = File::open(cmd_name).unwrap().metadata().unwrap();

    // Make sure `root` owns the executable.
    if cmd_metadata.st_uid() != 0 {
        message::error("This executable needs to be owned by `root` in order to run.\n");
        quit::with_code(exitcode::USAGE);
    // Make sure the `setuid` bit flag is set. This appears to be third
    // digit in the six-digit long mode returned.
    } else if format!("{:o}", cmd_metadata.permissions().mode())
        .chars()
        .nth(2)
        .unwrap()
        .to_string()
        .parse::<u8>()
        .unwrap()
        < 4
    {
        message::error(
            "This executable needs to have the `setuid` bit flag set in order to run command.\n",
        );
        quit::with_code(exitcode::USAGE);
    }

    util::sudo::to_root();

    // If we're running a command that should be permission-checked, then do so.
    if vec!["install", "remove", "update", "upgrade"].contains(&cmd_results.subcommand().unwrap().0)
    {
        // If we're running a command that invokes 'makedeb', ensure that we're not
        // running as root.
        if vec!["install", "upgrade"].contains(&cmd_results.subcommand().unwrap().0)
            && *util::sudo::NORMAL_UID == 0
        {
            message::error(&format!(
            "This command cannot be ran as root, as it needs to call '{}', which is required to run under a non-root user.\n",
            "makedeb".bold().green()
        ));
            quit::with_code(exitcode::USAGE);
        }

        util::sudo::check_perms();
    }

    match cmd_results.subcommand() {
        Some(("clone", args)) => clone::clone(args),
        Some(("comment", args)) => comment::comment(args),
        Some(("install", args)) => install::install(args),
        Some(("list", args)) => list::list(args),
        Some(("list-comments", args)) => list_comments::list_comments(args),
        Some(("remove", args)) => remove::remove(args),
        Some(("search", args)) => search::search(args),
        Some(("update", args)) => update::update(args),
        Some(("upgrade", args)) => upgrade::upgrade(args),
        Some(("whoami", args)) => whoami::whoami(args),
        _ => unreachable!(),
    };
}
