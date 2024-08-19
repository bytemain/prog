use std::env;
use std::path::PathBuf;

extern crate directories;
use directories::ProjectDirs;

pub fn join_home_dir(path: &str) -> PathBuf {
    let mut home_dir = match env::var_os("HOME") {
        Some(path) => PathBuf::from(path),
        None => panic!("Could not find home directory"),
    };

    home_dir.push(path);
    home_dir
}

pub fn get_config_path(file: &str) -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "prog") {
        let config_path = proj_dirs.config_dir();
        config_path.join(file)
    } else {
        panic!("Could not find project directory")
    }
}
