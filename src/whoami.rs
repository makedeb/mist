use crate::{message, util};
use serde::Deserialize;

#[derive(Deserialize)]
struct Authenticated {
    user: String,
}

pub fn whoami(args: &clap::ArgMatches) {
    let api_token: &String = match args.get_one("token") {
        Some(token) => token,
        None => {
            message::error("No API key was provided.");
            quit::with_code(exitcode::USAGE);
        }
    };
    let mpr_url: &String = args.get_one("mpr-url").unwrap();

    let request = util::AuthenticatedRequest::new(api_token, mpr_url);
    let resp_text = request.get("test");
    let json = serde_json::from_str::<Authenticated>(&resp_text).unwrap();

    println!("Authenticated to the MPR as {}.", json.user);
}
