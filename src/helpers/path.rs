use std::path::PathBuf;

use dirs::home_dir;

use std::cell::LazyCell;

const HOME_PATH: LazyCell<PathBuf> = LazyCell::new(|| home_dir().unwrap());

pub fn join_home_dir(path: &str) -> PathBuf {
    let mut home_dir = HOME_PATH.clone();
    home_dir.push(path);
    home_dir
}


pub fn get_config_path(file: &str) -> PathBuf {
    let mut config_dir = join_home_dir(".prog");
    config_dir.push(file);
    config_dir
}

