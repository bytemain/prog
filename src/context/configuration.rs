use crate::helpers::{
    path::{expand_tilde, PROGRAM},
    rand::get_random_string,
};
use log::info;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

fn default_auto_sync_interval_secs() -> i64 {
    3600
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub base: Vec<String>,
    #[serde(default)]
    pub alias: HashMap<String, String>,
    #[serde(default)]
    pub tmp_dir: String,
    #[serde(default = "default_auto_sync_interval_secs")]
    pub auto_sync_interval_secs: i64,
}

impl Config {
    pub fn base_dirs(&self) -> Vec<String> {
        let mut base_dirs = Vec::new();

        for base_dir in &self.base {
            base_dirs.push(expand_tilde(base_dir));
        }

        base_dirs
    }

    pub fn replace_alias(&self, url: String) -> String {
        for (key, value) in &self.alias {
            if url.starts_with(key) {
                info!("Replace alias: {} -> {}", key, value);
                return format!("{}{}", value, &url[key.len()..]);
            }
        }
        url
    }

    pub fn tmp_dir(&self) -> String {
        if self.tmp_dir.is_empty() {
            panic!("Please configure tmp_dir in config file");
        }

        expand_tilde(&self.tmp_dir)
    }

    pub fn create_tmp_dir(&self) -> PathBuf {
        let suffix = get_random_string(6);

        let mut path_buf = PathBuf::from(self.tmp_dir());
        path_buf.push(format!("{}-{}", PROGRAM, suffix));
        path_buf
    }

    pub fn get_auto_sync_interval_secs(&self) -> i64 {
        self.auto_sync_interval_secs
    }
}
