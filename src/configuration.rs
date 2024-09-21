use serde::Deserialize;
use std::collections::HashMap;
use anyhow::bail;
use crate::{constants, helpers};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub base: Vec<String>,
    #[serde(default)]
    pub alias: HashMap<String, String>,
}

impl Config {
    pub fn get_base_dir(&self, uri: &String) -> anyhow::Result<String> {
        let base_dirs = self.get_all_base_dir();
        if base_dirs.len() == 0 {
            bail!(
                "please configure base dir in : {}",
                helpers::path::get_config_path(constants::CONFIG_TOML_FILE).to_str().unwrap()
            );
        }

        anyhow::Ok(base_dirs.get(0).unwrap().clone())
    }

    pub fn get_all_base_dir(&self) -> Vec<String> {
        let mut base_dirs = Vec::new();

        for base_dir in &self.base {
            base_dirs.push(shellexpand::tilde(base_dir).to_string());
        }

        base_dirs
    }
}
