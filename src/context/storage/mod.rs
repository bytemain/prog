use std::{borrow::Borrow, collections::HashMap};

mod utils;

use crate::{constants, helpers};
use git_url_parse::GitUrl;

use ejdb::{bson, Database};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
struct Record {
    created_at: i64,
    updated_at: i64,
    /// The fully qualified domain name (FQDN) or IP of the repo
    host: String,
    /// The name of the repo
    repo: String,
    /// The owner/account/project name
    owner: String,
    /// this repo will be stored in this base dir
    base_dir: String,
    /// user original input
    remote_url: String,
}

pub struct Storage<'a> {
    db: Database,
    collection_cache: HashMap<String, ejdb::Collection<'a>>,
}

impl<'a> Storage<'a> {
    pub fn new() -> Self {
        let database_path = helpers::path::get_config_path(constants::DATABASE_FILE);
        let db = Database::open(database_path).unwrap();
        Self {
            db,
            collection_cache: HashMap::new(),
        }
    }

    pub fn record_item(&'a mut self, base_dir: &str, remote_url: &str, git_url: &GitUrl) {
        let collection = self.get_records_collection();
        let record = Record {
            created_at: helpers::time::get_current_timestamp(),
            updated_at: helpers::time::get_current_timestamp(),
            host: git_url.host.clone().unwrap(),
            repo: git_url.name.clone(),
            owner: git_url.owner.clone().unwrap(),
            remote_url: remote_url.to_string(),
            base_dir: base_dir.to_string(),
        };
        collection
            .save(bson::to_bson(&record).unwrap().as_document().unwrap())
            .unwrap();
    }
    fn get_collection(&'a mut self, name: &str) -> &ejdb::Collection<'a> {
        self.collection_cache
            .entry(name.to_string())
            .or_insert_with(|| self.db.collection(name).unwrap())
    }

    fn get_records_collection(&'a mut self) -> &ejdb::Collection<'a> {
        let collection = self.get_collection("records");
        collection
    }
}
