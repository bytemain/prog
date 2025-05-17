use std::{fs, path::PathBuf};

use dirs::home_dir;

use std::cell::LazyCell;

const HOME_PATH: LazyCell<PathBuf> = LazyCell::new(|| home_dir().unwrap());

pub fn join_home_dir(path: &str) -> PathBuf {
    let mut home_dir = HOME_PATH.clone();
    home_dir.push(path);
    home_dir
}

pub const PROGRAM: &str = "prog";
const FOLDER: &str = ".prog";

pub fn get_config_path(file: &str) -> PathBuf {
    let mut config_dir = join_home_dir(FOLDER);
    config_dir.push(file);
    config_dir
}

pub fn expand_path(path: &str) -> PathBuf {
    let path = shellexpand::tilde(path).into_owned();
    PathBuf::from(path)
}

pub fn ensure_dir_exists(path: &PathBuf) {
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

pub fn exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}
