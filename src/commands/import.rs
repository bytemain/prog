use super::add;
use crate::context::Context;
use crate::helpers::git::get_remote_url;
use crate::helpers::path::expand_path;
use log::info;
use std::path::PathBuf;
use std::process::exit;

pub fn run(c: &mut Context, path: PathBuf) {
    info!("Import: {:?}", path);
    let path = expand_path(path.to_str().unwrap());
    if path.exists() {
        info!("path exists, trying to clone");
        let remote_url = get_remote_url(path.to_str().unwrap());
        add::run(c, &remote_url, &vec![]);
        return;
    }

    eprintln!("Path does not exist");
    exit(1);
}
