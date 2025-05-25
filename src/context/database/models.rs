use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repo {
    pub created_at: chrono::naive::NaiveDateTime,
    pub updated_at: chrono::naive::NaiveDateTime,
    pub host: String,
    pub repo: String,
    pub owner: String,
    pub remote_url: String,
    pub base_dir: String,
    pub full_path: String,
}

impl Repo {
    pub fn owner_fs_path(&self) -> String {
        let path = PathBuf::new().join(&self.base_dir).join(&self.host).join(&self.owner);
        path.to_str().unwrap().to_string()
    }

    pub fn host_fs_path(&self) -> String {
        let path = PathBuf::new().join(&self.base_dir).join(&self.host);
        path.to_str().unwrap().to_string()
    }
}
