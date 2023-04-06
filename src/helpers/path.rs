use std::env;
use std::path::PathBuf;

pub fn join_home_dir(path: &str) -> String {
    let mut home_dir = match env::var_os("HOME") {
        Some(path) => PathBuf::from(path),
        None => panic!("Could not find home directory"),
    };

    home_dir.push(path);

    match home_dir.to_str() {
        Some(path_str) => path_str.to_owned(),
        None => String::new(),
    }
}
