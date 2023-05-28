use std::env;
use std::path::PathBuf;

use crate::constants;

pub fn join_home_dir(path: &str) -> PathBuf {
    let mut home_dir = match env::var_os("HOME") {
        Some(path) => PathBuf::from(path),
        None => panic!("Could not find home directory"),
    };

    home_dir.push(path);
    home_dir
}

pub fn get_config_path(file: &str) -> String {
    let config_path: PathBuf = join_home_dir(constants::DEFAULT_CONFIG_PATH);
    let file_path = config_path.join(file);
    match file_path.to_str() {
        Some(path_str) => path_str.to_owned(),
        None => String::new(),
    }
}
