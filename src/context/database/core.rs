use super::index_records::*;
use super::models::*;
use crate::constants;
use crate::helpers::path::ensure_dir_exists;
use crate::helpers::path::get_config_path;
use log::error;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    version: String,
    records: IndexedRecords,
    last_sync_time: Option<chrono::NaiveDateTime>,
}

const CURRENT_VERSION: &str = "1.0";

impl Data {
    pub fn new() -> Self {
        Self {
            version: CURRENT_VERSION.to_string(),
            records: IndexedRecords::new(),
            last_sync_time: None,
        }
    }

    pub fn reset(&mut self) {
        self.version = CURRENT_VERSION.to_string();
        self.records = IndexedRecords::new();
        self.last_sync_time = None;
    }

    pub fn record_item(
        &mut self,
        base_dir: &str,
        remote_url: &str,
        host: &str,
        repo: &str,
        owner: &str,
        full_path: &str,
    ) {
        let now = chrono::Utc::now().naive_utc();
        // Get the original creation time if available
        let created_at = if let Some(existing) = self.records.get(full_path) {
            existing.created_at
        } else {
            now
        };

        // Create updated record
        let updated_record = Repo {
            created_at,
            updated_at: now,
            host: host.to_string(),
            repo: repo.to_string(),
            owner: owner.to_string(),
            base_dir: base_dir.to_string(),
            remote_url: remote_url.to_string(),
            full_path: full_path.to_string(),
        };
        self.records.insert(full_path, updated_record);
    }

    pub fn find(&self, keyword: &str) -> Vec<Repo> {
        let keyword = keyword.to_lowercase();
        self.records
            .get_all()
            .iter()
            .filter(|r| {
                r.host.to_lowercase().contains(&keyword)
                    || r.repo.to_lowercase().contains(&keyword)
                    || r.owner.to_lowercase().contains(&keyword)
                    || r.base_dir.to_lowercase().contains(&keyword)
                    || r.remote_url.to_lowercase().contains(&keyword)
            })
            .cloned()
            .collect()
    }

    pub fn remove(&mut self, path: &str) {
        self.records.remove(path);
    }

    pub fn get_all_items(&self) -> Vec<Repo> {
        self.records.get_all().clone()
    }
}

pub struct Database {
    data: Data,
}

impl Database {
    pub fn new() -> Self {
        let database_file = Self::get_db_path();

        if database_file.exists() {
            match Self::load_from_file(&database_file) {
                Ok(db) => db,
                Err(e) => {
                    error!("Error loading database: {}. Creating a new one.", e);
                    Self::create_new_database()
                }
            }
        } else {
            Self::create_new_database()
        }
    }

    pub fn reset(&mut self) {
        self.data.reset();
    }

    fn get_db_path() -> PathBuf {
        let database_path = get_config_path(crate::helpers::path::DATA_FOLDER);
        ensure_dir_exists(&database_path);
        database_path.join(constants::DATABASE_FILE)
    }

    fn load_from_file(path: &Path) -> Result<Self, String> {
        let mut file =
            File::open(path).map_err(|e| format!("Unable to open database file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Unable to read database file: {}", e))?;

        toml::from_str(&contents)
            .map(|data| Self { data })
            .map_err(|e| format!("Unable to deserialize database: {}", e))
    }

    fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let contents = toml::to_string(&self.data)
            .map_err(|e| format!("Unable to serialize database: {}", e))?;
        let mut file =
            File::create(path).map_err(|e| format!("Unable to create database file: {}", e))?;
        file.write_all(contents.as_bytes())
            .map_err(|e| format!("Unable to write to database file: {}", e))
    }

    pub fn get_last_sync_time(&self) -> Option<chrono::NaiveDateTime> {
        self.data.last_sync_time
    }

    pub fn update_last_sync_time(&mut self) {
        self.data.last_sync_time = Some(chrono::Utc::now().naive_utc());
    }

    fn create_new_database() -> Self {
        let db = Data::new();
        let db = Self { data: db };

        db.save().unwrap();
        db
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        let database_file = Self::get_db_path();
        self.save_to_file(&database_file)
    }

    pub fn record_item(
        &mut self,
        base_dir: &str,
        remote_url: &str,
        host: &str,
        repo: &str,
        owner: &str,
        full_path: &str,
    ) {
        self.data.record_item(base_dir, remote_url, host, repo, owner, full_path);
    }
    pub fn find(&self, keyword: &str) -> Vec<Repo> {
        self.data.find(keyword)
    }
    pub fn remove(&mut self, path: &str) {
        self.data.remove(path);
    }
    pub fn get_all_items(&self) -> Vec<Repo> {
        self.data.get_all_items()
    }
    pub fn size(&self) -> usize {
        self.data.records.size()
    }
}
