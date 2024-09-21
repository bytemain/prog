use std::env;
use std::path::PathBuf;

use directories::BaseDirs;

use std::cell::LazyCell;

const BASE_DIRS: LazyCell<BaseDirs> = LazyCell::new(|| BaseDirs::new().unwrap());

pub fn join_home_dir(path: &str) -> PathBuf {
    let mut home_dir = BASE_DIRS.home_dir().to_owned();
    home_dir.push(path);
    home_dir
}


pub fn get_config_path(file: &str) -> PathBuf {
    let mut config_dir = join_home_dir(".prog");
    config_dir.push(file);
    config_dir
}
