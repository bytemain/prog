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
    pub fn fs_path(&self) -> String {
        format!("{}/{}/{}/{}", self.base_dir, self.host, self.owner, self.repo)
    }
}
