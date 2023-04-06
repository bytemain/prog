use super::configuration;
use crate::helpers;
use anyhow::{bail, Ok, Result};

pub struct Context<'a> {
    config: &'a configuration::Config,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a configuration::Config) -> Self {
        Self { config }
    }

    pub fn get_base_dir(&self, uri: &String) -> Result<&String> {
        // if no base dir
        // throw error
        if self.config.base.len() == 0 {
            bail!(
                "please configure base dir in : {}",
                helpers::path::join_home_dir(configuration::DEFAULT_CONFIG_TOML_PATH)
            );
        }

        Ok(self.config.base.get(0).unwrap())
    }
}
