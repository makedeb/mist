use crate::{
    cli::{Cli, CliWhoami},
    message, util,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Authenticated {
    user: String,
}

pub fn whoami(args: &Cli, _cmd_args: &CliWhoami) {
    let api_token = match args.token {
        Some(ref token) => token,
        None => {
            message::error("No API key was provided.");
            quit::with_code(exitcode::USAGE);
        }
    };
    let mpr_url = &args.mpr_url;

    let request = util::AuthenticatedRequest::new(api_token, mpr_url);
    let resp_text = request.get("test");
    let json = serde_json::from_str::<Authenticated>(&resp_text).unwrap();

    println!("Authenticated to the MPR as {}.", json.user);
}
