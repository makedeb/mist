use crate::{
    cache::{Cache, MprCache},
    cli::{Cli, CliSearch},
    style,
};
use rust_apt::cache::{Cache as AptCache, PackageSort};

pub fn search(args: &Cli, cmd_args: &CliSearch) {
    let query_list = &cmd_args.query;
    let apt_only = args.apt_only;
    let mpr_only = args.mpr_only;
    let installed_only = args.installed_only;
    let name_only = args.name_only;

    let cache = Cache::new(AptCache::new(), MprCache::new());
    let mut candidates = Vec::new();

    for query in query_list {
        for pkg in Cache::get_nonvirtual_packages(cache.apt_cache(), &PackageSort::default()) {
            let pkgname = pkg.name();
            if (pkgname.contains(query)
                || pkg
                    .candidate()
                    .unwrap()
                    .description()
                    .unwrap_or_default()
                    .contains(query))
                && !candidates.contains(&pkgname)
            {
                candidates.push(pkgname);
            }
        }

        for pkg in cache.mpr_cache().packages().values() {
            if (pkg.pkgname.contains(query)
                || pkg.pkgdesc.clone().unwrap_or_default().contains(query))
                && !candidates.contains(&pkg.pkgname)
            {
                candidates.push(pkg.pkgname.to_string());
            }
        }
    }

    if candidates.is_empty() {
        quit::with_code(exitcode::UNAVAILABLE);
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
