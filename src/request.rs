use crate::message;
use exitcode;
use quit;
use reqwest;
use std::{thread, time::Duration};

pub fn get(url: &str) -> reqwest::blocking::Response {
    let mut resp_count = 0;

    loop {
        match reqwest::blocking::get(url) {
            Ok(resp) => {
                break resp
            },
            Err(err) => {
                if resp_count > 2 {
                    message::error(
                        &format!("Unable to make request to '{}'.", url)
                    );
                    quit::with_code(exitcode::UNAVAILABLE);
                } else {
                    resp_count += 1;
                    thread::sleep(Duration::from_millis(100));
                };
            }
        }
    }
}
