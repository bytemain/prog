use std::path::Path;
use std::process::Command;

use crate::context::Context;
use url::Url;

pub fn run(c: &Context, url: &String, rest: &Vec<String>) {
    let base_dir = c.get_base_dir(url).unwrap();

    let url_parsed = Url::parse(url.as_str()).unwrap();

    let host = url_parsed.host_str().unwrap();
    let path = url_parsed
        .path()
        .strip_prefix('/')
        .unwrap_or(url_parsed.path());

    println!("Protocol: {}", url_parsed.scheme());
    println!("Host: {}", url_parsed.host_str().unwrap());
    println!("Path: {}", url_parsed.path());

    println!("base dir: {}", base_dir.as_str());

    let full_path = Path::new(base_dir).join(host).join(path);

    println!("target full path: {}", full_path.to_str().unwrap());

    Command::new("program");

    crate::helpers::shell::clone(
        &url,
        &rest,
        full_path
            .to_str()
            .expect(format!("Cannot construct full path for {}", url).as_str()),
    );
}
