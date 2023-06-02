mod path;
mod storage;

use super::configuration;
use super::constants;

pub struct Context<'a> {
    config: &'a configuration::Config,
    pub path: path::Path<'a>,
    pub storage: storage::Storage,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a configuration::Config) -> Self {
        let path = path::Path::new(config);
        let storage = storage::Storage::new();
        Self {
            config,
            path,
            storage,
        }
    }
}
