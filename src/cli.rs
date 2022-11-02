use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum CliPaging {
    /// auto
    Auto,
    /// always
    Always,
    /// never
    Never,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// The API token to authenticate to the MPR with
    #[clap(long, env = "MPR_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// URL to access the MPR from
    #[clap(
        long,
        env = "MPR_URL",
        hide_env_values = true,
        default_value = "https://mpr.makedeb.org"
    )]
    pub mpr_url: String,

    /// Filter results to packages available on the MPR
    #[arg(long, default_value_t = false, conflicts_with = "apt_only")]
    pub mpr_only: bool,

    /// Filter results to packages available via APT
    #[arg(long, default_value_t = false, conflicts_with = "mpr_only")]
    pub apt_only: bool,

    /// Filter results to installed packages
    #[clap(long, short = 'i')]
    #[arg(default_value_t = false)]
    pub installed_only: bool,

    /// Output the package's name without any extra details
    #[clap(long, default_value = "false")]
    pub name_only: bool,

    #[clap(subcommand)]
    pub subcommand: CliSubcommand,
}

#[derive(Debug, Parser)]
pub enum CliSubcommand {
    /// Clone a package base from the MPR
    Clone(CliClone),

    /// Comment on a package page
    Comment(CliComment),

    /// Install packages from APT and the MPR
    Install(CliInstall),

    /// List packages available via APT and the MPR
    List(CliList),

    /// List the comments on a package
    ListComments(CliListComments),

    /// Remove packages from the system
    Remove(CliRemove),

    /// Search for an APT/MPR package
    Search(CliSearch),

    /// Update the APT cache on the system
    Update(CliUpdate),

    /// Upgrade the packages on the system
    Upgrade(CliUpgrade),

    /// Show the currently authenticated user
    Whoami(CliWhoami),
}

#[derive(Debug, Parser)]
pub struct CliClone {
    /// The package to clone
    #[clap(required = true)]
    pub pkg: String,
}

#[derive(Debug, Parser)]
pub struct CliComment {
    /// The package to comment on
    #[clap(required = true)]
    pub pkg: String,

    /// The comment to post
    #[clap(long, short)]
    pub msg: Option<String>,
}

#[derive(Debug, Parser)]
pub struct CliInstall {
    /// The package(s) to install
    #[clap(required = true)]
    pub pkg: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct CliList {
    /// The package(s) to get information for
    pub pkg: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct CliListComments {
    /// The package to view comments for
    #[clap(required = true)]
    pub pkg: String,

    /// When to send output to a pager
    #[clap(long)]
    #[arg(value_enum, default_value_t = CliPaging::Auto)]
    pub paging: CliPaging,
}

#[derive(Debug, Parser)]
pub struct CliRemove {
    /// The package(s) to remove
    #[clap(required = true)]
    pub pkg: Vec<String>,

    /// Remove configuration files along with the package(s)
    #[clap(long)]
    #[arg(default_value_t = false)]
    pub purge: bool,

    /// Automatically remove any unneeded packages
    #[clap(long)]
    #[arg(default_value_t = false)]
    pub autoremove: bool,
}

#[derive(Debug, Parser)]
pub struct CliSearch {
    /// The query to search for
    #[clap(required = true)]
    pub query: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct CliUpdate {}

#[derive(Debug, Parser)]
pub struct CliUpgrade {}

#[derive(Debug, Parser)]
pub struct CliWhoami {}
