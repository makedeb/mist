use crate::message;
use serde::{Deserialize, Serialize};
use std::str;

#[derive(Deserialize, Serialize)]
pub struct Authenticated {
    #[serde(rename = "type")]
    pub resp_type: String,
    pub msg: String,
}

pub static MPR_URL: &str = "mpr.makedeb.org";

// Struct to handle API-authenticated requests to the MPR.
pub struct AuthenticatedRequest<'a> {
    api_token: &'a str,
}

impl<'a> AuthenticatedRequest<'a> {
    pub fn new(api_token: &'a str) -> Self {
        Self { api_token }
    }

    pub fn get(&self, path: &str) -> String {
        // Make the request.
        let client = reqwest::blocking::Client::new();
        let resp = match client
            .get(format!("https://{}/api/{}", self::MPR_URL, path))
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
        
        if let Ok(json) = serde_json::from_str::<Authenticated>(&resp_text) {
                // TODO: We need to define a more suitable way for machine parsing of errors in the
                // MPR. Maybe something like '{"err_type": "invalid_api_key"}'.
                if json.resp_type == "error" && json.msg == "Invalid API key." {
                    message::error("Invalid API key was passed in.");
                    quit::with_code(exitcode::USAGE);
                }
        }

        resp_text
    }
}
