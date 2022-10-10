use crate::args::SearchMode;

pub fn search(
    query_list: &Vec<String>,
    mpr_url: &String,
    mode: &SearchMode,
    name_only: bool,
    installed_only: bool,
) -> String {
    crate::list::list(query_list, mpr_url, mode, name_only, installed_only)
}
