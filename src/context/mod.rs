mod path;
mod storage;

use std::cell::Ref;
use std::cell::RefCell;

use super::configuration;
use super::constants;

pub struct Context<'a> {
    pub config: &'a configuration::Config,
    path: RefCell<path::Path<'a>>,
    storage: RefCell<storage::Storage>,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a configuration::Config) -> Self {
        let path = RefCell::new(path::Path::new(config));
        let storage = RefCell::new(storage::Storage::new());
        Self {
            config,
            path,
            storage,
        }
    }

    #[inline]
    pub fn storage(&self) -> Ref<'_, storage::Storage> {
        self.storage.borrow()
    }

    #[inline]
    pub fn path(&self) -> Ref<'_, path::Path> {
        self.path.borrow()
    }
}
