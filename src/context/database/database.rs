use super::models::*;
use crate::helpers::path::ensure_dir_exists;
use crate::constants;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    version: String,
    records: Vec<Repo>,
}

impl Database {
    pub fn new() -> Self {
        let database_path = constants::DATABASE_FOLDER.clone();
        ensure_dir_exists(&database_path);

        let database_file = database_path.join("db.toml");
        if database_file.exists() {
            let mut file = File::open(&database_file).expect("Unable to open database file");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Unable to read database file");
            let db: Database = toml::from_str(&contents).expect("Unable to parse database file");
            db
        } else {
            let db = Self {
                version: "1.0".to_string(),
                records: vec![],
            };
            db.save();
            db
        }
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
        self.save();
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
        self.save();
    }

    pub fn get_all_items(&self) -> Vec<Repo> {
        self.records.clone()
    }

    fn save(&self) {
        let database_path = constants::DATABASE_FOLDER.clone();
        let database_file = database_path.join("db.toml");
        let contents = toml::to_string(self).expect("Unable to serialize database");
        let mut file = File::create(&database_file).expect("Unable to create database file");
        file.write_all(contents.as_bytes()).expect("Unable to write to database file");
    }

    pub fn migrate(&mut self) {
        // Migration logic here
    }
}