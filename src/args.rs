use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Clone a package base from the MPR
    Clone {
        /// The package to clone
        #[arg(required = true)]
        package_name: String,

        #[command(flatten)]
        mpr_url: MprURL,
    },
    /// Comment on a package page
    Comment {
        /// The package to comment on
        #[arg(required = true)]
        package_name: String,

        /// The comment to post
        #[arg(short, long = "msg")]
        message: Option<String>,

        #[command(flatten)]
        mpr_url: MprURL,

        #[command(flatten)]
        mpr_token: MprToken,
    },
    /// Install packages from APT and the MPR
    Install {
        /// The package(s) to install
        #[arg(required = true)]
        package_names: Vec<String>,

        #[command(flatten)]
        mpr_url: MprURL,
    },
    /// List packages available via APT and the MPR
    List {
        #[command(flatten)]
        mpr_url: MprURL,

        /// Output the package's name without any extra details
        #[arg(long = "name-only", default_value_t = false)]
        name_only: bool,

        #[arg(long, value_enum, default_value_t=SearchMode::None)]
        mode: SearchMode,

        /// The package(s) to get information for
        #[arg(required = true)]
        package_names: Vec<String>,
    },
    /// List the comments on a package
    ListComments {
        #[command(flatten)]
        mpr_url: MprURL,

        /// When to send output to a pager
        #[arg(long, value_enum, default_value_t=Paging::Auto)]
        paging: Paging,

        /// The package(s) to get information for
        #[arg(required = true)]
        package_name: String,
    },
    /// Remove packages from the system
    Remove {
        #[command(flatten)]
        mpr_url: MprURL,

        /// Remove configuration files along with the package(s)
        #[arg(long)]
        purge: bool,

        /// Remove configuration files along with the package(s)
        #[arg(long)]
        autoremove: bool,

        /// The package(s) to get information for
        #[arg(required = true)]
        package_names: Vec<String>,
    },
    /// Search for an APT/MPR package
    Search {
        #[command(flatten)]
        mpr_url: MprURL,

        /// Output the package's name without any extra details
        #[arg(long = "name-only", default_value_t = false)]
        name_only: bool,

        #[arg(long, value_enum, default_value_t=SearchMode::None)]
        mode: SearchMode,

        /// The package(s) to get information for
        #[arg(required = true)]
        query: Vec<String>,
    },
    /// Update the APT cache on the system
    Update {
        #[command(flatten)]
        mpr_url: MprURL,
    },
    /// Upgrade the packages on the system
    Upgrade {
        #[command(flatten)]
        mpr_url: MprURL,

        #[arg(long, value_enum, default_value_t=UpgradeMode::Both)]
        mode: UpgradeMode,
    },
    /// Show the currently authenticated user
    Whoami {
        #[command(flatten)]
        mpr_url: MprURL,

        #[command(flatten)]
        mpr_token: MprToken,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum UpgradeMode {
    /// Upgrade both APT and MPR packages
    Both,
    /// Only upgrade APT packages
    AptOnly,
    /// Only upgrade MPR packages
    MprOnly,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Paging {
    Auto,
    Always,
    Never,
}

#[derive(Args)]
pub struct MprURL {
    /// URL to access the MPR from
    #[arg(
        env = "MPR_URL",
        default_value = "https://mpr.makedeb.org",
        long = "mpr-url"
    )]
    pub url: String,
}

#[derive(Args)]
pub struct MprToken {
    /// The API token to authenticate to the MPR with
    #[arg(
        env = "MPR_TOKEN",
        required = true,
        hide_env_values = true,
        long = "token"
    )]
    pub token: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SearchMode {
    /// No filter applied
    None,
    /// Only packages available on the MPR
    MprOnly,
    /// Only packages available via APT
    AptOnly,
    /// Only installed packages
    Installed,
}
