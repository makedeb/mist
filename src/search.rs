use crate::{
    cache::{Cache, CachePackage, MprCache},
    style,
};
use rust_apt::cache::Cache as AptCache;

pub fn search(args: &clap::ArgMatches) {
    let query_list: Vec<&String> = args.get_many("query").unwrap().collect();
    let apt_only = args.is_present("apt-only");
    let mpr_only = args.is_present("mpr-only");
    let installed_only = args.is_present("installed-only");
    let name_only = args.is_present("name-only");
    let mpr_url: &String = args.get_one("mpr-url").unwrap();

    let cache = Cache::new(AptCache::new(), MprCache::new(mpr_url));
    let mut candidates: Vec<&Vec<CachePackage>> = Vec::new();

    for query in query_list {
        for (pkgname, pkg_group) in cache.pkgmap().iter() {
            let mut pkgs = Vec::new();
            let apt_pkg = cache.get_apt_pkg(pkgname);
            let mpr_pkg = cache.get_mpr_pkg(pkgname);

            if let Some(pkg) = apt_pkg {
                pkgs.push(pkg);
            }
            if let Some(pkg) = mpr_pkg {
                pkgs.push(pkg);
            }

            for pkg in pkgs {
                if (pkg.pkgname.contains(query)
                    || pkg
                        .pkgdesc
                        .as_ref()
                        .unwrap_or(&"".to_owned())
                        .contains(query))
                    && !candidates.contains(&pkg_group)
                {
                    candidates.push(pkg_group);
                }
            }
        }
    }

    print!(
        "{}",
        style::generate_pkginfo_entries(
            &candidates,
            &cache,
            apt_only,
            mpr_only,
            installed_only,
            name_only
        )
    );
}
