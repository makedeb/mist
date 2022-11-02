#![feature(let_chains)]
mod cache;
mod cli;
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

pub use rust_apt::util as apt_util;
use std::{
    env,
    fs::File,
    os::{linux::fs::MetadataExt, unix::fs::PermissionsExt},
};
use style::Colorize;
use which::which;

#[quit::main]
fn main() {
    use clap::Parser;
    let cmd_results = cli::Cli::parse();

    // Make sure that this executable has the `setuid` flag set and is owned by
    // root. Parts of this program (intentionally) expect such behavior.
    let cmd_name = {
        let cmd = env::args().collect::<Vec<String>>().remove(0);
        if cmd.contains('/') {
            cmd
        } else {
            which(&cmd).unwrap().into_os_string().into_string().unwrap()
        }
    };

    let cmd_metadata = File::open(&cmd_name).unwrap().metadata().unwrap();

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

    use cli::CliSubcommand::*;

    // If we're running a command that should be permission-checked, then do so.
    if matches!(
        cmd_results.subcommand,
        Install(_) | Remove(_) | Update(_) | Upgrade(_)
    ) {
        // If we're running a command that invokes 'makedeb', ensure that we're not
        // running as root.
        if matches!(cmd_results.subcommand, Install(_) | Upgrade(_)) && *util::sudo::NORMAL_UID == 0
        {
            message::error(&format!(
            "This command cannot be ran as root, as it needs to call '{}', which is required to run under a non-root user.\n",
            "makedeb".bold().green()
        ));
            quit::with_code(exitcode::USAGE);
        }

        util::sudo::check_perms();
    }

    match &cmd_results.subcommand {
        Clone(clone_args) => clone::clone(&cmd_results, clone_args),
        Comment(comment_args) => comment::comment(&cmd_results, comment_args),
        Install(install_args) => install::install(&cmd_results, install_args),
        List(list_args) => list::list(&cmd_results, list_args),
        ListComments(list_comments_args) => {
            list_comments::list_comments(&cmd_results, list_comments_args)
        }
        Remove(remove_args) => remove::remove(&cmd_results, remove_args),
        Search(search_args) => search::search(&cmd_results, search_args),
        Update(update_args) => update::update(&cmd_results, update_args),
        Upgrade(upgrade_args) => upgrade::upgrade(&cmd_results, upgrade_args),
        Whoami(whoami_args) => whoami::whoami(&cmd_results, whoami_args),
    };
}
