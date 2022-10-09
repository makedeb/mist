use crate::{
    args::SearchMode,
    cache::{Cache, CachePackage, MprCache},
    style,
};
use rust_apt::cache::Cache as AptCache;

pub fn list(pkglist: &Vec<String>, _: &String, mode: &SearchMode, name_only: &bool) {
    let cache = Cache::new(AptCache::new(), MprCache::new());
    let mut candidates: Vec<&Vec<CachePackage>> = Vec::new();

    if !pkglist.is_empty() {
        for pkg in pkglist {
            if let Some(pkg_group) = cache.pkgmap().get(pkg) {
                candidates.push(pkg_group);
            }
        }
    } else {
        for pkg_group in cache.pkgmap().values() {
            candidates.push(pkg_group);
        }
    }

    print!(
        "{}",
        style::generate_pkginfo_entries(&candidates, &cache, mode, *name_only)
    );
}
