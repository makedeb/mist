use crate::{message, mpr_cache};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    process::{Command, ExitStatus, Stdio},
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
                message::error(&format!("Failed to make request [{}]", err));
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
                message::error("Invalid API key was passed in.");
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
pub struct CommandInfo<'a> {
    pub args: &'a Vec<&'a str>,
    pub capture: bool,
    pub stdin: Option<&'a str>,
}

pub struct CommandResult {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_status: ExitStatus,
}

pub fn run_command(cmd: &CommandInfo) -> CommandResult {
    let cmd_name = cmd.args[0];
    let cmd_args = &cmd.args[1..];
    // Functions like 'Command::stdin()' return references to the object created by
    // 'Command::new()', which returns the object itself.
    // We want to only interact with references to the object from hereon out.
    let mut _result = Command::new(cmd_name);
    let mut result = &mut _result;
    result = result.args(cmd_args);

    // If we passed in stdin, set up the command to accept it.
    if cmd.stdin.is_some() {
        result = result.stdin(Stdio::piped());
    }

    // Take in stdout and stderr if needed.
    if cmd.capture {
        result = result.stdout(Stdio::piped());
        result = result.stderr(Stdio::piped());
    }

    // Start the subprocess.
    let mut result = match result.spawn() {
        Ok(child) => child,
        Err(err) => {
            message::error(&format!(
                "Failed to run command. [{:?}] [{}]",
                cmd.args, err
            ));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    };

    // If we passed in stdin previously, pass in our stdin.
    if let Some(stdin) = cmd.stdin {
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

// Function that finds the matching package base of a given package.
pub fn find_pkgbase<'a>(
    pkgname: &'a str,
    package_cache: &'a Vec<mpr_cache::MprCache>,
) -> Option<&'a str> {
    for pkg in package_cache {
        if pkg.pkgname == pkgname {
            return Some(pkg.pkgbase.as_str());
        }
    }

    None
}
