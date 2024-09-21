use super::configuration;
use super::constants;
use crate::helpers;
use anyhow::{bail, Ok, Result};

pub struct Path<'a> {
    config: &'a configuration::Config,
}

impl<'a> Path<'a> {
    pub fn new(config: &'a configuration::Config) -> Self {
        Self { config }
    }

    pub fn get_base_dir(&self, uri: &String) -> Result<String> {
        let base_dirs = self.get_all_base_dir();
        if base_dirs.len() == 0 {
            bail!(
                "please configure base dir in : {}",
                helpers::path::get_config_path(constants::CONFIG_TOML_FILE).to_str().unwrap()
            );
        }

        Ok(base_dirs.get(0).unwrap().clone())
    }

    pub fn get_all_base_dir(&self) -> Vec<String> {
        let mut base_dirs = Vec::new();

        for base_dir in &self.config.base {
            base_dirs.push(shellexpand::tilde(base_dir).to_string());
        }

        base_dirs
    }
}
