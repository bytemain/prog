use crate::schema::repos;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = repos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
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
        format!("{}/{}/{}", self.base_dir, self.owner, self.repo)
    }
}

#[derive(Insertable)]
#[diesel(table_name = repos)]
pub struct NewRepo<'a> {
    pub created_at: chrono::naive::NaiveDateTime,
    pub updated_at: chrono::naive::NaiveDateTime,
    pub host: &'a str,
    pub repo: &'a str,
    pub owner: &'a str,
    pub remote_url: &'a str,
    pub base_dir: &'a str,
    pub full_path: &'a str,
}
