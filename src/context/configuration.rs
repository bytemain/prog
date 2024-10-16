use crate::constants;
use anyhow::bail;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub base: Vec<String>,
    #[serde(default)]
    pub alias: HashMap<String, String>,
}

impl Config {
    pub fn get_base_dir(&self) -> anyhow::Result<String> {
        let base_dirs = self.get_all_base_dir();
        if base_dirs.len() == 0 {
            bail!(
                "please configure base dir in : {}",
                constants::CONFIG_TOML_FILE.to_str().unwrap()
            );
        }

        if base_dirs.len() == 1 {
            return anyhow::Ok(base_dirs.get(0).unwrap().clone());
        }

        bail!("Not implemented multiple base dir yet");
    }

    pub fn get_all_base_dir(&self) -> Vec<String> {
        let mut base_dirs = Vec::new();

        for base_dir in &self.base {
            base_dirs.push(shellexpand::tilde(base_dir).to_string());
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
}