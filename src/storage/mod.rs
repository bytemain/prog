use crate::{constants, helpers};

struct Record {
    id: i32,
    created_at: i64,
    updated_at: i64,
    name: String,
}

pub struct Storage {
    database_path: String,
}

impl Storage {
    pub fn new() -> Self {
        let database_path = helpers::path::get_config_path(constants::DATABASE_FILE);

        Self { database_path }
    }
}
