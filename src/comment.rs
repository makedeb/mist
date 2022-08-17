use crate::{cache::MprCache, message, util};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;

#[derive(Deserialize)]
struct CommentResult {
    link: String,
}

pub fn comment(args: &clap::ArgMatches) {
    let pkg: &String = args.get_one("pkg").unwrap();
    let mpr_url: &String = args.get_one("mpr-url").unwrap();
    let api_token: &String = match args.get_one("token") {
        Some(token) => token,
        None => {
            message::error("No API token was provided.");
            quit::with_code(exitcode::USAGE);
        }
    };

    // Get a list of packages.
    let mpr_cache = MprCache::new(mpr_url);
    let mut pkgnames: Vec<&String> = Vec::new();

    for pkg in mpr_cache.packages() {
        pkgnames.push(&pkg.pkgname);
    }

    // Abort if the package base doesn't exist.
    if !pkgnames.contains(&pkg) {
        message::error(&format!("Package '{}' doesn't exist on the MPR.", pkg));
        quit::with_code(exitcode::USAGE);
    }

    // Get the message.
    // If no message was supplied, get one from the user.
    let msg: String = match args.get_one::<String>("msg") {
        Some(msg) => (msg).to_owned(),
        None => {
            // Get the editor.
            let editor = match edit::get_editor() {
                Ok(editor) => editor.into_os_string().into_string().unwrap(),
                Err(err) => {
                    message::error(&format!(
                        "Couldn't find an editor to write a comment with. [{}]",
                        err
                    ));

                    quit::with_code(exitcode::UNAVAILABLE);
                }
            };

            // Generate a temporary file to write the message in.
            let file = match tempfile::Builder::new().suffix(".md").tempfile_in("/tmp") {
                Ok(file) => file.path().to_str().unwrap().to_owned(),
                Err(err) => {
                    message::error(&format!(
                        "Failed to create temporary file to write comment in. [{}]",
                        err
                    ));
                    quit::with_code(exitcode::UNAVAILABLE);
                }
            };

            // Open the file in the editor.
            message::info(&format!("Opening '{}' in '{}'...", &file, editor));

            let cmd = util::CommandInfo {
                args: &vec![&editor, &file],
                capture: false,
                stdin: None,
            };
            util::run_command(&cmd);

            // Read the changed file.
            let mut file_content = String::new();
            let mut _file = File::open(file).unwrap();
            _file.read_to_string(&mut file_content).unwrap();

            file_content
        }
    };

    // Upload the message!
    let body = json!({ "msg": msg }).to_string();

    let request = util::AuthenticatedRequest::new(api_token, mpr_url);
    let resp_text = request.post(&format!("comment/{}", pkg), body);

    // Parse the message.
    let json = serde_json::from_str::<CommentResult>(&resp_text).unwrap();
    message::info(&format!("Succesfully posted comment. [{}]", json.link));
}
