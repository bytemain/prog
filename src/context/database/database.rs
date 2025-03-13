use super::models::*;
use crate::constants;
use crate::helpers::path::ensure_dir_exists;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use log::error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    version: String,
    records: Vec<Repo>,
}

const CURRENT_VERSION: &str = "1.0";

impl Database {
    pub fn delete_db_folder() {
        let db_file = constants::DATABASE_FOLDER.clone();
        match std::fs::remove_dir_all(&db_file) {
            Ok(_) => {}
            Err(err) => error!("Could not delete db file: {}", err),
        }
    }

    fn get_db_path() -> PathBuf {
        let database_path = constants::DATABASE_FOLDER.clone();
        ensure_dir_exists(&database_path);
        database_path.join("db.toml")
    }

    fn load_from_file(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| format!("Unable to open database file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| format!("Unable to read database file: {}", e))?;
        toml::from_str(&contents).map_err(|e| format!("Unable to parse database file: {}", e))
    }

    fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let contents = toml::to_string(self).map_err(|e| format!("Unable to serialize database: {}", e))?;
        let mut file = File::create(path).map_err(|e| format!("Unable to create database file: {}", e))?;
        file.write_all(contents.as_bytes()).map_err(|e| format!("Unable to write to database file: {}", e))
    }
    fn create_new_database() -> Self {
        let db = Self { version: CURRENT_VERSION.to_string(), records: vec![] };
        if let Err(e) = db.save() {
            error!("Warning: Failed to save new database: {}", e);
        }
        db
    }

    pub fn new() -> Self {
        let database_file = Self::get_db_path();
        
        let db = if database_file.exists() {
            match Self::load_from_file(&database_file) {
                Ok(db) => {
                    db
                },
                Err(e) => {
                    error!("Error loading database: {}. Creating a new one.", e);
                    Self::create_new_database()
                }
            }
        } else {
            Self::create_new_database()
        };
        
        db
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
        if self.records.iter().any(|r| r.full_path == full_path) {
            println!("Project already exists: {}", full_path);
            return;
        }

        let record = Repo {
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            host: host.to_string(),
            repo: repo.to_string(),
            owner: owner.to_string(),
            base_dir: base_dir.to_string(),
            remote_url: remote_url.to_string(),
            full_path: full_path.to_string(),
        };

        self.records.push(record);
        if let Err(e) = self.save() {
            error!("Warning: Failed to save database after adding record: {}", e);
        }
    }

    pub fn find(&self, keyword: &str) -> Vec<Repo> {
        let keyword = keyword.to_lowercase();
        self.records
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
        self.records.retain(|r| r.full_path != path);
        if let Err(e) = self.save() {
            error!("Warning: Failed to save database after removing record: {}", e);
        }
    }

    pub fn get_all_items(&self) -> Vec<Repo> {
        self.records.clone()
    }

    fn save(&self) -> Result<(), String> {
        let database_file = Self::get_db_path();
        self.save_to_file(&database_file)
    }
}