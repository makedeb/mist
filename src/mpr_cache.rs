use crate::{message, util};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::{fs, time::SystemTime};

// REMOVE LATER!
use std::io::prelude::*;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct MprCache {
    #[serde(rename = "Name")]
    pub pkgname: String,
    #[serde(rename = "PackageBase")]
    pub pkgbase: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Description")]
    pub pkgdesc: Option<String>,
    #[serde(rename = "Maintainer")]
    pub maintainer: Option<String>,
    #[serde(rename = "NumVotes")]
    pub num_votes: u32,
    #[serde(rename = "Popularity")]
    pub popularity: f32,
    #[serde(rename = "OutOfDate")]
    pub ood: Option<u32>,
}

pub fn new() -> Vec<MprCache> {
    // Get the XDG cache directory.
    let cache_dir = match dirs::cache_dir() {
        Some(dir) => dir,
        None => {
            message::error("Unable to find the xdg cache directory.");
            quit::with_code(exitcode::UNAVAILABLE);
        }
    };

    // Make sure the directory exists.
    let mut mpr_cache_dir = cache_dir;
    mpr_cache_dir.push("mpr-cli");

    if !mpr_cache_dir.exists() {
        match fs::create_dir(mpr_cache_dir.clone()) {
            Ok(()) => (),
            Err(err) => {
                message::error(&format!(
                    "Encountered an unknown error while creating the cache directory. [{}]",
                    err
                ));
                quit::with_code(exitcode::UNAVAILABLE);
            }
        }
    } else if !mpr_cache_dir.is_dir() {
        message::error(&format!(
            "Cache path '{}' isn't a directory.",
            mpr_cache_dir.display()
        ));
        quit::with_code(exitcode::OSERR);
    }

    // Try reading the cache file. If it doesn't exist or it's older than five minutes, we have to
    // update the cache file.
    let mut mpr_cache_file = mpr_cache_dir;
    mpr_cache_file.push("cache.gz");

    let mut update_cache = false;

    match fs::metadata(mpr_cache_file.clone()) {
        // The file exists. Make sure it's been updated in the last five minutes.
        Ok(metadata) => {
            let five_minutes = 60 * 5; // The MPR updates package archives every five minutes.
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let file_last_modified = metadata
                .modified()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if (current_time - file_last_modified) > five_minutes {
                update_cache = true;
            };
        }
        // The file doesn't exist. We need to create it.
        Err(err) => {
            if err.raw_os_error().unwrap() != 2 {
                message::error(&format!(
                    "Encountered an unknown error while reading cache. [{}]",
                    err
                ));
                quit::with_code(exitcode::OSFILE);
            } else {
                update_cache = true;

                match fs::File::create(mpr_cache_file.clone()) {
                    Ok(_) => (),
                    Err(err) => {
                        message::error(&format!(
                            "Encountered an unknown error while reading cache. [{}]",
                            err
                        ));
                        quit::with_code(exitcode::OSFILE);
                    }
                }
            }
        }
    };

    // If we need to, update the cache file.
    if update_cache {
        // Download the archive.
        let resp = match reqwest::blocking::get(format!(
            "https://{}/packages-meta-ext-v2.json.gz",
            util::MPR_URL
        )) {
            Ok(resp) => resp,
            Err(err) => {
                message::error(&format!("Unable to make request. [{}]", err));
                quit::with_code(exitcode::UNAVAILABLE);
            }
        };

        if !resp.status().is_success() {
            message::error(&format!(
                "Failed to download package archive from the MPR. [{}]",
                resp.status()
            ));
            quit::with_code(exitcode::TEMPFAIL);
        }

        // Decompress the archive.
        let cache = match valid_archive(resp) {
            Ok(cache) => cache,
            Err(num) => {
                if num == 1 {
                    message::error("Failed to decompress package archive from the MPR.");
                    quit::with_code(exitcode::TEMPFAIL);
                } else {
                    message::error("Failed to verify integrity of package archive from the MPR.");
                    quit::with_code(exitcode::TEMPFAIL);
                }
            }
        };

        // Now that the JSON has been verified, let's write out the archive to the cache file.
        let mut config_compressor = GzEncoder::new(Vec::new(), Compression::default());
        config_compressor
            .write_all(serde_json::to_string(&cache).unwrap().as_bytes())
            .unwrap();
        let config_gz = config_compressor.finish().unwrap();

        match fs::write(mpr_cache_file, config_gz) {
            Ok(()) => (),
            Err(err) => {
                message::error(&format!(
                    "Failed to write updated package archive. [{}]",
                    err
                ));
                quit::with_code(exitcode::IOERR);
            }
        }

        // Return the new cache object.
        cache
    } else {
        // The cache is less than 5 minutes old. We still need to validate that the cache is valid
        // though.
        let cache_file = match fs::File::open(mpr_cache_file.clone()) {
            Ok(file) => file,
            Err(err) => {
                message::error(&format!(
                    "Failed to write updated package archive. [{}]",
                    err
                ));
                quit::with_code(exitcode::IOERR);
            }
        };

        match valid_archive(cache_file) {
            Ok(file) => file,
            Err(_) => {
                // On an error, let's just remove the cache file and regenerate it by recalling
                // this function.
                fs::remove_file(mpr_cache_file).unwrap();
                self::new()
            }
        }
    }
}

fn valid_archive(file: impl Read) -> Result<Vec<MprCache>, u32> {
    let mut resp_gz = GzDecoder::new(file);
    let mut resp_json = String::new();

    match resp_gz.read_to_string(&mut resp_json) {
        Ok(_) => (),
        Err(_) => return Err(1),
    }

    // Feed the JSON into our struct.
    let cache = match serde_json::from_str::<Vec<MprCache>>(&resp_json) {
        Ok(json) => json,
        Err(_) => return Err(2),
    };

    Ok(cache)
}
