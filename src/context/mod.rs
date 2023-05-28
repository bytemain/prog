use super::configuration;
use super::constants;
use super::storage;
use crate::helpers;
use anyhow::{bail, Ok, Result};

pub struct Context<'a> {
    config: &'a configuration::Config,
    storage: storage::Storage,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a configuration::Config) -> Self {
        let storage = storage::Storage::new();
        Self { config, storage }
    }

    pub fn get_base_dir(&self, uri: &String) -> Result<&String> {
        // if no base dir
        // throw error
        if self.config.base.len() == 0 {
            bail!(
                "please configure base dir in : {}",
                helpers::path::get_config_path(constants::CONFIG_TOML_FILE)
            );
        }

        Ok(self.config.base.get(0).unwrap())
    }
}
