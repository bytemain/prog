use std::path::Path;

use crate::helpers::colors::Colorize;
use crate::{context::Context, helpers::platform};
use git_url_parse::GitUrl;
use log::debug;

pub fn run(c: &mut Context, url: &str, rest: &Vec<String>) {
    let base_dir = c.config().get_base_dir().unwrap();
    let url = c.config().replace_alias(url.to_owned());

    let url_parsed = GitUrl::parse(&url).unwrap();
    debug!("url parsed: {:#?}", url_parsed);

    if url_parsed.host.is_none() || url_parsed.owner.is_none() {
        eprintln!("{}", format!("Invalid git url: {}", url).red());
        return;
    }

    let host = url_parsed.host.clone().unwrap();
    let owner = url_parsed.owner.clone().unwrap();
    let name = url_parsed.name.clone();
    let fullname = url_parsed.fullname.clone();

    debug!("host: {host}, full name: {fullname}, base dir: {base_dir}");

    let full_path = Path::new(&base_dir).join(&host).join(fullname);

    if full_path.exists() {
        println!("{}", format!("Repo already exists: {}", full_path.display()).green());
        platform::clipboard::copy_path(full_path.to_str().unwrap());
        return;
    }

    debug!("target full path: {}", full_path.display());
    let target_path =
        full_path.to_str().unwrap_or_else(|| panic!("Cannot construct full path for {}", url));
    println!("{}", format!("Add: {}", url).green());

    let result = crate::helpers::git::clone(&url, rest, target_path);

    if result.is_err() {
        eprintln!("\n{}", format!("Failed to clone: {}", url).red());
        return;
    }

    c.database_mut().record_item(&base_dir, &url, &host, &name, &owner, target_path);
    c.database_mut().save().unwrap();

    println!("{}", format!("Cloned to: {}", target_path).green());
    platform::clipboard::copy_path(target_path);
}
