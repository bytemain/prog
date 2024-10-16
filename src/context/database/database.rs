use super::models::*;
use crate::schema::repos::dsl;
use crate::{constants, helpers, schema::repos};
use diesel::prelude::*;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::debug;
use std::vec;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct Database {
    conn: SqliteConnection,
}

impl Database {
    pub fn new() -> Self {
        let database_path = constants::DATABASE_FOLDER.clone();
        let database_url = database_path.join("db.sqlite3").to_string_lossy().to_string();
        let conn = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        let mut db = Self { conn };

        db.setup_database();
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
        let record_exists =
            dsl::repos.filter(dsl::full_path.eq(full_path)).first::<Repo>(&mut self.conn);
        // check if the record already exists
        if let Ok(r) = record_exists {
            println!("Project already exists: {}", r.fs_path());
            return;
        }

        let record = NewRepo {
            created_at: helpers::time::get_current_timestamp(),
            updated_at: helpers::time::get_current_timestamp(),
            host,
            repo,
            owner,
            base_dir,
            remote_url,
            full_path,
        };

        diesel::insert_into(repos::table)
            .values(&record)
            .returning(Repo::as_returning())
            .get_result(&mut self.conn)
            .expect("Error saving new repo");
    }

    pub fn find(&mut self, keyword: &str) -> Vec<Repo> {
        println!("Searching for: {}", keyword);

        use crate::schema::repos::dsl::*;

        let results = repos
            .filter(host.like(keyword))
            .or_filter(repo.like(keyword))
            .or_filter(owner.like(keyword))
            .or_filter(base_dir.like(keyword))
            .or_filter(remote_url.like(keyword))
            .select(Repo::as_select())
            .load::<Repo>(&mut self.conn)
            .unwrap();

        let mut valid_repos = vec![];

        for result in &results {
            debug!("{:?}", result);
            debug!("Path: {}", result.fs_path());
            valid_repos.push(result.clone());
        }

        valid_repos
    }

    pub fn remove(&mut self, path: &str) {
        use crate::schema::repos::dsl::*;
        diesel::delete(repos.filter(full_path.eq(path))).execute(&mut self.conn);
    }

    fn setup_database(&mut self) {
        diesel::sql_query(
            r#"
            PRAGMA busy_timeout = 60000;
			PRAGMA journal_mode = WAL;
			PRAGMA synchronous = NORMAL;
			PRAGMA foreign_keys = ON;
            "#,
        )
        .execute(&mut self.conn)
        .unwrap();

        self.conn.run_pending_migrations(MIGRATIONS).unwrap();
    }
}
