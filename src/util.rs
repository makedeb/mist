use crate::message;
use serde::{Deserialize, Serialize};
use std::{
    process::{Command, ExitStatus},
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

    pub fn get(&self, path: &str) -> String {
        // Make the request.
        let client = reqwest::blocking::Client::new();
        let resp = match client
            .get(format!("{}/api/{}", self.mpr_url, path))
            .header("Authorization", self.api_token)
            .send()
        {
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
}

// Function to run a command, and abort if it fails.
pub fn run_command(args: &Vec<&str>) -> ExitStatus {
    let cmd = args[0];
    let cmd_args = &args[1..];

    let result = Command::new(cmd).args(cmd_args).spawn();

    match result {
        Ok(mut child) => child.wait().unwrap(),
        Err(err) => {
            message::error(&format!("Failed to run command {:?} [{}]", args, err));
            quit::with_code(exitcode::UNAVAILABLE);
        }
    }
}
