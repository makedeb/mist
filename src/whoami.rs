use crate::{util};
use serde::Deserialize;

#[derive(Deserialize)]
struct Authenticated {
    user: String,
}

pub fn whoami(api_token: String, mpr_url: String) {
    let request = util::AuthenticatedRequest::new(api_token, mpr_url);
    let resp_text = request.get("test");
    let json = serde_json::from_str::<Authenticated>(&resp_text).unwrap();

    println!("Authenticated to the MPR as {}.", json.user);
}
