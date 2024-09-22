use crate::context::storage::migrations::MIGRATIONS;
use crate::{constants, helpers};
use log::{debug, info};
use rusqlite::{named_params, params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
struct Record<'a> {
    created_at: i64,
    updated_at: i64,
    /// The fully qualified domain name (FQDN) or IP of the repo
    host: &'a str,
    /// The name of the repo
    repo: &'a str,
    /// The owner/account/project name
    owner: &'a str,
    /// this repo will be stored in this base dir
    base_dir: &'a str,
    /// user original input
    remote_url: &'a str,
    /// the full path to the repo
    full_path: &'a str,
}

impl<'a> Record<'a> {
    pub fn fs_path(&self) -> String {
        format!("{}/{}/{}", self.base_dir, self.owner, self.repo)
    }
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
            host,
            repo,
            owner,
            base_dir,
            remote_url,
            full_path,
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

    pub fn find(&self, keyword: &str) {
        println!("Searching for: {}", keyword);
        let mut stmt = self.conn.prepare("SELECT * FROM repos WHERE host LIKE ?1 OR repo LIKE ?1 OR owner LIKE ?1 OR base_dir LIKE ?1 OR remote_url LIKE ?1").unwrap();
        let mut rows = stmt.query(params![keyword]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let record = Record {
                created_at: row.get("created_at").unwrap(),
                updated_at: row.get("updated_at").unwrap(),
                host: &row.get::<_, String>("host").unwrap(),
                repo: &row.get::<_, String>("repo").unwrap(),
                owner: &row.get::<_, String>("owner").unwrap(),
                base_dir: &row.get::<_, String>("base_dir").unwrap(),
                remote_url: &row.get::<_, String>("remote_url").unwrap(),
                full_path: &row.get::<_, String>("full_path").unwrap(),
            };
            debug!("{:?}", record);
            debug!("Path: {}", record.fs_path());
        }
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
