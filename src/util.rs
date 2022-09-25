use crate::{apt_util, message, style::Colorize};
use core::fmt::Display;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs as std_fs,
    io::{self, Write},
    path,
    process::{Command as ProcCommand, ExitStatus},
    str,
};

#[derive(Deserialize, Serialize)]
struct AuthenticationError {
    #[serde(rename = "type")]
    pub resp_type: String,
    pub code: String,
}

// Struct to handle API-authenticated requests to the MPR.
pub struct AuthenticatedRequest<'a> {
    api_token: &'a str,
    mpr_url: &'a str,
}

impl<'a> AuthenticatedRequest<'a> {
    pub fn new(api_token: &'a str, mpr_url: &'a str) -> Self {
        Self { api_token, mpr_url }
    }

    fn handle_response(&self, resp: reqwest::Result<reqwest::blocking::Response>) -> String {
        let resp = match resp {
            Ok(resp) => resp,
            Err(err) => {
                message::error(&format!("Failed to make request [{}]\n", err));
                quit::with_code(exitcode::UNAVAILABLE);
            }
        };

        // Check the response and see if we got a bad API token error. If we did, go
        // ahead and abort the program.
        let resp_text = resp.text().unwrap();

        if let Ok(json) = serde_json::from_str::<AuthenticationError>(&resp_text) {
            // TODO: We need to define a more suitable way for machine parsing of errors in
            // the MPR. Maybe something like '{"err_type": "invalid_api_key"}'.
            if json.resp_type == "error" && json.code == "err_invalid_api_key" {
                message::error("Invalid API key was passed in.\n");
                quit::with_code(exitcode::USAGE);
            }
        }

        resp_text
    }

    pub fn get(&self, path: &str) -> String {
        // Make the request.
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(format!("{}/api/{}", self.mpr_url, path))
            .header("Authorization", self.api_token)
            .send();

        self.handle_response(resp)
    }

    pub fn post(&self, path: &str, body: String) -> String {
        // Make the request.
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(format!("{}/api/{}", self.mpr_url, path))
            .body(body)
            .header("Authorization", self.api_token)
            .send();

        self.handle_response(resp)
    }
}

/// Handle errors from APT.
pub fn handle_errors(err_str: &apt_util::Exception) {
    for msg in err_str.what().split(';') {
        if msg.starts_with("E:") {
            message::error(&format!("{}\n", msg.strip_prefix("E:").unwrap()));
        } else if msg.starts_with("W:") {
            message::warning(&format!("{}\n", msg.strip_prefix("W:").unwrap()));
        };
    }
}

/// Run a command, and error out if it fails.
pub fn check_exit_status(cmd: &ProcCommand, status: &ExitStatus) {
    if !status.success() {
        let mut args = vec![cmd.get_program().to_str().unwrap().to_string()];
        for arg in cmd.get_args() {
            args.push(arg.to_str().unwrap().to_string());
        }

        message::error(&format!("Failed to run command: {:?}\n", args));
        quit::with_code(exitcode::UNAVAILABLE);
    }
}

/// Format a list of package names in the way APT would.
pub fn format_apt_pkglist<T: AsRef<str> + Display>(pkgnames: &Vec<T>) {
    // All package lines always start with two spaces, so pretend like we have two
    // less characters.
    let term_width = apt_util::terminal_width() - 2;
    let mut output = String::from("  ");
    let mut current_width = 0;

    for _pkgname in pkgnames {
        let pkgname = _pkgname.as_ref();
        output.push_str(pkgname);
        current_width += pkgname.len();

        if current_width > term_width {
            output.push_str("\n  ");
            current_width = 0;
        } else {
            output.push(' ');
        }
    }

    println!("{}", output);
}

/// Check if a response was a "yes" response. 'default' is what to return if
/// 'resp' is empty.
pub fn is_yes(resp: &str, default: bool) -> bool {
    resp.to_lowercase() == "y" || (resp.is_empty() && default)
}

/// Print out a question with options and get the result.
/// `multi_allowed` specifies if only a single option can be chosen.
pub fn ask_question(question: &str, options: &Vec<&str>, multi_allowed: bool) -> Vec<String> {
    let num_re = Regex::new("^[0-9]*-[0-9]*$|^[0-9]*$").unwrap();
    let options_len = options.len();
    message::question(question);

    // Panic if no options were passed in, there's nothing to work with there. This
    // function should only be used internally anyway, so this just gives a heads up
    // that it's being used incorrectly.
    if options.is_empty() {
        panic!("No values passed in for `options` parameter");
    }

    // Print the options.
    let mut str_options: Vec<String> = Vec::new();

    for (index, item) in options.iter().enumerate() {
        str_options.push(format!("[{}] {}", index, item))
    }

    format_apt_pkglist(&str_options);

    let print_question = || -> Option<Vec<String>> {
        let mut returned_items: Vec<String> = Vec::new();

        if multi_allowed {
            print!(
                "{}",
                "Please enter a selection (i.e. `1-3 5`, defaults to `0`): ".bold()
            );
        } else {
            print!(
                "{}",
                "Please enter a selection (i.e. `1` or `6`, defaults to `0`): ".bold()
            );
        }
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Pop off the leading newline.
        input.pop();

        // If no response was given, return the first item in the options.
        if input.is_empty() {
            returned_items.push(options.first().unwrap().to_string());
            return Some(returned_items);
        }

        let matched_items: Vec<&str> = input.split(' ').collect();

        if !multi_allowed
            && (matched_items.len() > 1 || matched_items.first().unwrap().contains('-'))
        {
            message::error("Only one value is allowed to be specified.\n");
            return None;
        }

        for item in &matched_items {
            if !num_re.is_match(item) {
                message::error(&format!(
                    "Error parsing item `{}`. Please make sure it is valid.\n",
                    item
                ));
                return None;
            }

            if item.contains('-') {
                let (num1_str, num2_str) = item.split_once('-').unwrap();
                let num1: usize = num1_str.parse().unwrap();
                let num2: usize = num2_str.parse().unwrap();

                if num1 > options_len - 1 {
                    message::error(&format!("Number is too big: {}\n", num1));
                    return None;
                } else if num2 > options_len - 1 {
                    message::error(&format!("Number is too big: {}\n", num2));
                    return None;
                }

                for num in num1..num2 {
                    returned_items.push(options.get(num).unwrap().to_string())
                }
            } else {
                let num: usize = item.parse().unwrap();

                if num > options_len - 1 {
                    message::error(&format!("Number is too big: {}\n", num));
                    return None;
                }
                returned_items.push(options.get(num).unwrap().to_string());
            }
        }

        Some(returned_items)
    };

    let mut result = print_question();
    while result.is_none() {
        result = print_question();
    }

    result.unwrap()
}

/// Get the system's distro and architecture. The first value returned is the
/// distribution, and the second is the architecture.
pub fn get_distro_arch_info() -> (String, String) {
    let mut distro_cmd = ProcCommand::new("lsb_release");
    distro_cmd.arg("-cs");
    let mut arch_cmd = ProcCommand::new("dpkg");
    arch_cmd.arg("--print-architecture");

    let distro = std::str::from_utf8(&distro_cmd.output().unwrap().stdout)
        .unwrap()
        .to_owned();
    let arch = std::str::from_utf8(&arch_cmd.output().unwrap().stdout)
        .unwrap()
        .to_owned();

    (distro, arch)
}

/// XDG directory wrapper thingermabobers.
pub mod xdg {
    /// Return the cache directory. Also creates it if it doesn't exist.
    pub fn get_cache_dir() -> super::path::PathBuf {
        let mut cache_dir = dirs::cache_dir().unwrap();
        cache_dir.push("mist");
        super::fs::create_dir(&cache_dir.clone().into_os_string().into_string().unwrap());
        cache_dir
    }

    /// Return the global cache directory that's for use by all users. Also
    /// creates it if it doesn't exist.
    pub fn get_global_cache_dir() -> super::path::PathBuf {
        let path: super::path::PathBuf = ["/var", "cache", "mist"].iter().collect();
        super::fs::create_dir(&path.clone().into_os_string().into_string().unwrap());
        path
    }
}

/// File/Folder wrappers for my joy.
pub mod fs {
    use crate::style::Colorize;

    /// Create a folder, aborting if unable to or the specified path already
    /// exists and isn't a folder.
    pub fn create_dir(directory: &str) {
        let path = super::path::Path::new(directory);
        if !path.exists() {
            if super::std_fs::create_dir_all(path).is_err() {
                super::message::error(&format!(
                    "Failed to create directory ({}).\n",
                    directory.green().bold()
                ));
                quit::with_code(exitcode::UNAVAILABLE);
            }
        } else if !path.is_dir() {
            super::message::error(&format!(
                "Path '{}' needs to be a directory, but it isn't.\n",
                directory.green().bold()
            ));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    }

    /// Create a file, aborting if unable to do so.
    pub fn create_file(path: &str) -> super::std_fs::File {
        match super::std_fs::File::create(path) {
            Ok(file) => file,
            Err(err) => {
                super::message::error(&format!(
                    "Failed to create file '{}' [{}]\n",
                    path.bold().green(),
                    err.to_string().bold()
                ));
                quit::with_code(exitcode::UNAVAILABLE);
            }
        }
    }
}

/// Sudo user management stuff.
pub mod sudo {
    super::lazy_static! {
        static ref NORMAL_UID: u32 = users::get_current_uid();
    }

    /// Change the user to root.
    pub fn to_root() {
        // Make sure the deref is ran on `normal` uid so that it's properly registered.
        let _ = *self::NORMAL_UID;

        users::switch::set_effective_uid(0).unwrap();
        users::switch::set_current_uid(0).unwrap();
    }

    pub fn check_perms() {
        super::message::info("Obtaining root permissions...\n");

        let mut cmd = self::run_as_normal_user("sudo");
        cmd.arg("true");

        if !cmd.spawn().unwrap().wait().unwrap().success() {
            super::message::error("Couldn't obtain root permissions.\n");
            quit::with_code(exitcode::USAGE);
        }
    }

    /// Change the user to the non-root user.
    // pub fn to_normal() {
    //     users::switch::set_effective_uid(*self::NORMAL_UID).unwrap();
    // }

    // Run a command as the normal user declared by [`NORMAL_UID`].
    pub fn run_as_normal_user<P: AsRef<super::OsStr>>(program: P) -> super::ProcCommand {
        let mut cmd = super::ProcCommand::new("sudo");
        cmd.args(["-E", "-n"]);
        cmd.arg(format!("-u#{}", *self::NORMAL_UID));
        cmd.arg("--");
        cmd.arg(program);
        cmd
    }
}
