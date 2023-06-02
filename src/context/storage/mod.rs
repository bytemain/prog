use crate::{constants, helpers};
use git_url_parse::GitUrl;

struct Record {
    created_at: i64,
    updated_at: i64,
    repo: String,
    owner: String,
    base_url: String,
    local_path: String,
}

pub struct Storage {
    database_path: String,
}

impl Storage {
    pub fn new() -> Self {
        let database_path = helpers::path::get_config_path(constants::DATABASE_FILE);

        Self { database_path }
    }

    pub fn recordItem(&self, remote_url: &str, git_url: &GitUrl, target_path: &str) {}
}
