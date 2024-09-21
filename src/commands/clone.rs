use std::path::Path;

use crate::context::Context;
use git_url_parse::GitUrl;
use log::debug;

pub fn run(c: &Context, url: &String, rest: &Vec<String>) {
    let base_dir = c.path().get_base_dir(url).unwrap();

    let url_parsed = GitUrl::parse(&url).unwrap();
    debug!("info: {:#?}", url_parsed);

    let host = url_parsed.host.clone().unwrap();
    let owner = url_parsed.owner.clone().unwrap();
    let name = url_parsed.name.clone();
    let fullname = url_parsed.fullname.clone();
    let protocol = url_parsed.scheme.clone();

    debug!("Protocol: {}", protocol);
    debug!("Host: {}", host);
    debug!("Full name: {}", fullname);
    debug!("base dir: {}", &base_dir);

    let full_path = Path::new(&base_dir).join(&host).join(fullname);

    debug!("target full path: {}", full_path.to_str().unwrap());
    let target_path =
        full_path.to_str().expect(format!("Cannot construct full path for {}", url).as_str());

    crate::helpers::shell::clone(&url, &rest, &target_path).unwrap();
    c.storage().record_item(&base_dir, &url, &host, &name, &owner);
}
