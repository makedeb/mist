use crate::{message, util};

pub fn whoami(args: &clap::ArgMatches) {
    let api_token = match args.value_of("token") {
        Some(token) => token,
        None => {
            message::error("No API key was provided.");
            quit::with_code(exitcode::USAGE);
        }
    };

    let request = util::AuthenticatedRequest::new(api_token);
    let resp_text = request.get("test");
    let json = serde_json::from_str::<util::Authenticated>(&resp_text).unwrap();

    println!("{}", json.msg);
}
