use crate::{apt_util, cache::MprCache, message, style::Colorize};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Write},
    process::{Command as ProcCommand, ExitStatus, Stdio},
    str,
};

use core::fmt::Display;
use regex::Regex;

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

        // Check the response and see if we got a bad API token error. If we did, go ahead and
        // abort the program.
        let resp_text = resp.text().unwrap();

        if let Ok(json) = serde_json::from_str::<AuthenticationError>(&resp_text) {
            // TODO: We need to define a more suitable way for machine parsing of errors in the
            // MPR. Maybe something like '{"err_type": "invalid_api_key"}'.
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

// Structs and functions to run a command, and abort if it fails.
pub struct CommandResult {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_status: ExitStatus,
}

pub struct Command {
    args: Vec<String>,
    capture: bool,
    stdin: Option<String>
}

impl Command {
    pub fn new(args: Vec<String>, capture: bool, stdin: Option<String>) -> Self {
        Self {
            args,
            capture,
            stdin,
        }
    }

    pub fn run(&self) -> CommandResult {
        let cmd_name = self.args.get(0).unwrap().clone();
        let cmd_args = &self.args[1..];
        // Functions like 'ProcCommand::stdin()' return references to the object created by
        // 'ProcCommand::new()', which returns the object itself.
        // We want to only interact with references to the object from hereon out.
        let mut _result = ProcCommand::new(cmd_name);
        let mut result = &mut _result;
        result = result.args(cmd_args);

        // If we passed in stdin, set up the command to accept it.
        if self.stdin.is_some() {
            result = result.stdin(Stdio::piped());
        }

        // Take in stdout and stderr if needed.
        if self.capture {
            result = result.stdout(Stdio::piped());
            result = result.stderr(Stdio::piped());
        }

        // Start the subprocess.
        let mut result = match result.spawn() {
            Ok(child) => child,
            Err(err) => {
                message::error(&format!(
                    "Failed to run command. [{:?}] [{}]\n",
                    self.args, err
                ));
                quit::with_code(exitcode::UNAVAILABLE);
            }
        };

        // If we passed in stdin previously, pass in our stdin.
        if let Some(stdin) = self.stdin.clone() {
            result
                .stdin
                .take()
                .unwrap()
                .write_all(stdin.as_bytes())
                .unwrap();
        }

        // Wait for the command to exit.
        let prog_exit = result.wait_with_output().unwrap();

        // Return the result.
        CommandResult {
            stdout: prog_exit.stdout,
            stderr: prog_exit.stderr,
            exit_status: prog_exit.status,
        }
    }
}

/// Function that finds the matching package base of a given package.
pub fn find_pkgbase<'a>(pkgname: &'a str, package_cache: &'a MprCache) -> Option<&'a str> {
    for pkg in package_cache.packages() {
        if pkg.pkgname == pkgname {
            return Some(pkg.pkgbase.as_str());
        }
    }

    None
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

/// Format a list of package names in the way APT would.
pub fn format_apt_pkglist<T: AsRef<str> + Display>(pkgnames: &Vec<T>) {
    // All package lines always start with two spaces, so pretend like we have two less characters.
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

/// Check if a response was a "yes" response. 'default' is what to return if 'resp' is empty.
pub fn is_yes(resp: &str, default: bool) -> bool {
    resp.to_lowercase() == "y" || (resp.is_empty() && default)
}

/// Run a function with the lockfile locked, and abort if there's an error.
pub fn with_lock<F: Fn()>(func: F) {
    if let Err(err) = apt_util::apt_lock() {
        handle_errors(&err);
        quit::with_code(exitcode::UNAVAILABLE);
    }

    func();

    if let Err(err) = apt_util::apt_unlock() {
        handle_errors(&err);
        quit::with_code(exitcode::UNAVAILABLE);
    }
}

/// Print out a question with options and get the result.
/// `multi_allowed` specifies if only a single option can be chosen.
pub fn ask_question(question: &str, options: &Vec<&str>, multi_allowed: bool) -> Vec<String> {
    let num_re = Regex::new("^[0-9]*-[0-9]*$|^[0-9]*$").unwrap();
    let options_len = options.len();
    message::question(question);

    // Panic if no options were passed in, there's nothing to work with there. This function should only be used internally anyway, so this just gives a heads up that it's being used incorrectly.
    if options.is_empty() {
        panic!("No values passed in for `options` parameter");
    }

    // Print the options.
    let mut str_options: Vec<String> = Vec::new();

    for (index, item) in options.iter().enumerate() {
        str_options.push(format!(
            "[{}] {}",
            index,
            item
        ))
    };

    format_apt_pkglist(&str_options);

    let print_question = || -> Option<Vec<String>> {
        let mut returned_items: Vec<String> = Vec::new();

        if multi_allowed {
            // Make sure there's an empty line by adding an extra newline at the beginning of this string.
            print!("\n{}", "Please enter a selection (i.e. `1-3 5`, defaults to `0`): ".bold());
        } else {
            print!("\n{}", "Please enter a selection (i.e. `1` or `6`, defaults to `0`): ".bold());
        }
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Pop off the leading newline.
        input.pop();

        // If no response was given, return the first item in the options.
        if input.is_empty() {
            returned_items.push(options.get(0).unwrap().to_string());
            return Some(returned_items);
        }
        
        let matched_items: Vec<&str> = input.split(' ').collect();

        if !multi_allowed && (matched_items.len() > 1 || matched_items.get(0).unwrap().contains('-')) {
            message::error("Only one value is allowed to be specified.\n");
            return None;
        }

        for item in &matched_items {
            if !num_re.is_match(item) {
                message::error(&format!("Error parsing item `{}`. Please make sure it is valid.\n", item));
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

                if num > options_len -1 {
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