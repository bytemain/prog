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
        // if no base dir
        // throw error
        if self.config.base.len() == 0 {
            bail!(
                "please configure base dir in : {}",
                helpers::path::get_config_path(constants::CONFIG_TOML_FILE)
                    .to_str()
                    .unwrap()
            );
        }

        Ok(self.config.base.get(0).unwrap().clone())
    }
}
