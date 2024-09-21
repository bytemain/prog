use crate::context::storage::migrations::MIGRATIONS;
use crate::{constants, helpers};
use git_url_parse::GitUrl;
use rusqlite::{params, Connection};
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

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new() -> Self {
        let database_path = helpers::path::get_config_path(constants::DATABASE_FILE);
        let conn = Connection::open(database_path);

        match conn {
            Ok(conn) => {
                let mut storage = Self { conn };
                storage.setup_database();
                storage
            }
            Err(e) => panic!("Could not open database: {}", e),
        }
    }

    pub fn record_item(&self, base_dir: &str, remote_url: &str, git_url: &GitUrl) {
        let record = Record {
            created_at: helpers::time::get_current_timestamp(),
            updated_at: helpers::time::get_current_timestamp(),
            host: git_url.host.clone().unwrap(),
            repo: git_url.name.clone(),
            owner: git_url.owner.clone().unwrap(),
            base_dir: base_dir.to_string(),
            remote_url: remote_url.to_string(),
        };

        let mut stmt = self.conn.prepare("INSERT INTO repos (created_at, updated_at, host, repo, owner, base_dir, remote_url) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)").unwrap();
        stmt.execute(params![
            &record.created_at,
            &record.updated_at,
            &record.host,
            &record.repo,
            &record.owner,
            &record.base_dir,
            &record.remote_url,
        ])
        .unwrap();
    }

    fn setup_database(&mut self) {
        MIGRATIONS.to_latest(&mut self.conn).unwrap();

        self.conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        self.conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    }
}
