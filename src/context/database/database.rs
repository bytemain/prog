use std::vec;

use crate::context::database::migrations::MIGRATIONS;
use crate::{constants, helpers};
use log::{debug, info};
use rusqlite::{named_params, params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Record {
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
    /// the full path to the repo
    full_path: String,
}

impl Record {
    pub fn fs_path(&self) -> String {
        format!("{}/{}/{}", self.base_dir, self.owner, self.repo)
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Self {
        let database_path = constants::DATABASE_FILE.clone();
        let conn = Connection::open(database_path);

        match conn {
            Ok(conn) => {
                let mut db = Self { conn };
                db.setup_database();
                db
            }
            Err(e) => panic!("Could not open database: {}", e),
        }
    }

    pub fn record_item(
        &self,
        base_dir: &str,
        remote_url: &str,
        host: &str,
        repo: &str,
        owner: &str,
        full_path: &str,
    ) {
        let record = Record {
            created_at: helpers::time::get_current_timestamp(),
            updated_at: helpers::time::get_current_timestamp(),
            host: host.to_string(),
            repo: repo.to_string(),
            owner: owner.to_string(),
            base_dir: base_dir.to_string(),
            remote_url: remote_url.to_string(),
            full_path: full_path.to_string(),
        };

        let mut stmt = self.conn.prepare("INSERT INTO repos (created_at, updated_at, host, repo, owner, base_dir, remote_url, full_path) VALUES (:created_at, :updated_at, :host, :repo, :owner, :base_dir, :remote_url, :full_path)").unwrap();
        stmt.execute(named_params![
            ":created_at": &record.created_at,
            ":updated_at": &record.updated_at,
            ":host": &record.host,
            ":repo": &record.repo,
            ":owner": &record.owner,
            ":base_dir": &record.base_dir,
            ":remote_url": &record.remote_url,
            ":full_path": &record.full_path,
        ])
        .unwrap();
    }

    pub fn find(&self, keyword: &str) -> Vec<Record> {
        println!("Searching for: {}", keyword);
        let mut stmt = self.conn.prepare("SELECT * FROM repos WHERE host LIKE ?1 OR repo LIKE ?1 OR owner LIKE ?1 OR base_dir LIKE ?1 OR remote_url LIKE ?1").unwrap();
        let mut rows = stmt.query(params![keyword]).unwrap();

        let mut result = vec![];
        while let Some(row) = rows.next().unwrap() {
            let record = Record {
                created_at: row.get("created_at").unwrap(),
                updated_at: row.get("updated_at").unwrap(),
                host: row.get("host").unwrap(),
                repo: row.get("repo").unwrap(),
                owner: row.get("owner").unwrap(),
                base_dir: row.get("base_dir").unwrap(),
                remote_url: row.get("remote_url").unwrap(),
                full_path: row.get("full_path").unwrap(),
            };
            debug!("{:?}", record);
            debug!("Path: {}", record.fs_path());

            result.push(record);
        }

        return result;
    }

    pub fn remove(&self, path: &str) {
        let mut stmt = self.conn.prepare("DELETE FROM repos WHERE full_path = ?1").unwrap();
        stmt.execute(params![path]).unwrap();
    }

    fn setup_database(&mut self) {
        MIGRATIONS.to_latest(&mut self.conn).unwrap();

        self.conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        self.conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    }
}
