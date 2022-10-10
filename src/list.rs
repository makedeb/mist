use crate::{
    args::SearchMode,
    cache::{Cache, CachePackage, CachePackageSource, MprCache},
    style,
};
use itertools::Itertools;
use rust_apt::cache::Cache as AptCache;

pub fn list(
    query_list: &Vec<String>,
    _: &String,
    mode: &SearchMode,
    name_only: bool,
    installed_only: bool,
) -> String {
    let cache = Cache::new(AptCache::new(), MprCache::new());

    let packages = if query_list.is_empty() {
        cache.pkglist()
    } else {
        query_list
            .iter()
            .flat_map(|query| cache.search(query))
            .collect()
    };

    let pkgs = if installed_only {
        packages
            .into_iter()
            .filter(|pkg| pkg.is_installed)
            .collect()
    } else {
        packages
    };

    let pkgs = match mode {
        SearchMode::None => pkgs,
        SearchMode::AptOnly => pkgs
            .into_iter()
            .filter(|pkg| pkg.source == CachePackageSource::Apt)
            .collect(),
        SearchMode::MprOnly => pkgs
            .into_iter()
            .filter(|pkg| pkg.source == CachePackageSource::Mpr)
            .collect(),
    };

    let pkgs: Vec<&CachePackage> = pkgs
        .into_iter()
        .unique_by(|pkg| pkg.pkgname.clone())
        .collect();

    match name_only {
        true => pkgs.iter().map(|pkg| &pkg.pkgname).join("\n"),
        false => pkgs
            .iter()
            .map(|pkg| style::generate_pkginfo_entry(&pkg.pkgname, &cache))
            .join("\n\n"),
    }
}
