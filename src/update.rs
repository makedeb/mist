use crate::{message, progress::MistAcquireProgress};
use rust_apt::{cache::Cache as AptCache, progress::AcquireProgress};

pub fn update(_args: &clap::ArgMatches) {
    let cache = AptCache::new();
    let mut progress: Box<dyn AcquireProgress> = Box::new(MistAcquireProgress {});

    if let Err(error) = cache.update(&mut progress) {
        for msg in error.what().split(';') {
            if msg.starts_with("E:") {
                message::error(&format!("{}\n", msg.strip_prefix("E:").unwrap()));
            } else if msg.starts_with("W:") {
                message::warning(&format!("{}\n", msg.strip_prefix("W:").unwrap()));
            };
        }
    };
}
