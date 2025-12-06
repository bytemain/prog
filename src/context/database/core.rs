use super::index_records::*;
use super::models::*;
use crate::constants;
use crate::helpers::path::ensure_dir_exists;
use crate::helpers::path::get_config_path;
use log::error;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use strsim::levenshtein;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    version: String,
    records: IndexedRecords,
    last_sync_time: Option<chrono::NaiveDateTime>,
}

const CURRENT_VERSION: &str = "1.0";

impl Data {
    pub fn new() -> Self {
        Self {
            version: CURRENT_VERSION.to_string(),
            records: IndexedRecords::new(),
            last_sync_time: None,
        }
    }

    pub fn reset(&mut self) {
        self.version = CURRENT_VERSION.to_string();
        self.records = IndexedRecords::new();
        self.last_sync_time = None;
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
        let now = chrono::Utc::now().naive_utc();
        // Get the original creation time if available
        let created_at = if let Some(existing) = self.records.get(full_path) {
            existing.created_at
        } else {
            now
        };

        // Create updated record
        let updated_record = Repo {
            created_at,
            updated_at: now,
            host: host.to_string(),
            repo: repo.to_string(),
            owner: owner.to_string(),
            base_dir: base_dir.to_string(),
            remote_url: remote_url.to_string(),
            full_path: full_path.to_string(),
        };
        self.records.insert(full_path, updated_record);
    }

    pub fn find(&self, keyword: &str) -> Vec<Repo> {
        let keyword = keyword.to_lowercase();

        // Use iterator to filter matching records without cloning all records first
        // This is more memory-efficient than get_all_sorted() for large datasets
        let mut results: Vec<Repo> = self
            .records
            .iter()
            .filter(|r| {
                r.full_path.to_lowercase().contains(&keyword)
                    || r.remote_url.to_lowercase().contains(&keyword)
            })
            .cloned()
            .collect();

        // Sort results by Levenshtein distance (similarity to keyword)
        results.sort_by(|a, b| {
            // Calculate Levenshtein distance for repo names
            let dist_a = levenshtein(&a.repo.to_lowercase(), &keyword);
            let dist_b = levenshtein(&b.repo.to_lowercase(), &keyword);

            // Sort by distance (lower is better/more similar)
            let dist_cmp = dist_a.cmp(&dist_b);
            if dist_cmp != std::cmp::Ordering::Equal {
                return dist_cmp;
            }

            // If distances are the same, sort by repository name alphabetically
            a.repo.to_lowercase().cmp(&b.repo.to_lowercase())
        });

        results
    }
}

pub struct Database {
    data: Data,
}

impl Database {
    pub fn new() -> Self {
        let database_file = Self::get_db_path();

        if database_file.exists() {
            match Self::load_from_file(&database_file) {
                Ok(db) => db,
                Err(e) => {
                    error!("Error loading database: {}. Creating a new one.", e);
                    Self::create_new_database()
                }
            }
        } else {
            Self::create_new_database()
        }
    }

    pub fn reset(&mut self) {
        self.data.reset();
    }

    fn get_db_path() -> PathBuf {
        let database_path = get_config_path(crate::helpers::path::DATA_FOLDER);
        ensure_dir_exists(&database_path);
        database_path.join(constants::DATABASE_FILE)
    }

    fn load_from_file(path: &Path) -> Result<Self, String> {
        let mut file =
            File::open(path).map_err(|e| format!("Unable to open database file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Unable to read database file: {}", e))?;

        toml::from_str(&contents)
            .map(|data| Self { data })
            .map_err(|e| format!("Unable to deserialize database: {}", e))
    }

    fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let contents = toml::to_string(&self.data)
            .map_err(|e| format!("Unable to serialize database: {}", e))?;
        let mut file =
            File::create(path).map_err(|e| format!("Unable to create database file: {}", e))?;
        file.write_all(contents.as_bytes())
            .map_err(|e| format!("Unable to write to database file: {}", e))
    }

    pub fn get_last_sync_time(&self) -> Option<chrono::NaiveDateTime> {
        self.data.last_sync_time
    }

    pub fn update_last_sync_time(&mut self) {
        self.data.last_sync_time = Some(chrono::Utc::now().naive_utc());
    }

    fn create_new_database() -> Self {
        let db = Data::new();
        let db = Self { data: db };

        db.save().unwrap();
        db
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        let database_file = Self::get_db_path();
        self.save_to_file(&database_file)
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
        self.data.record_item(base_dir, remote_url, host, repo, owner, full_path);
    }
    pub fn find(&self, keyword: &str) -> Vec<Repo> {
        self.data.find(keyword)
    }
    pub fn remove(&mut self, path: &str) {
        self.data.records.remove(path);
    }
    pub fn get_all_items(&self) -> Vec<Repo> {
        self.data.records.get_all_sorted()
    }
    pub fn size(&self) -> usize {
        self.data.records.size()
    }

    /// Get a repository record by its path
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the repository to get
    ///
    /// # Returns
    ///
    /// * `Option<Repo>` - The repository record if found, None otherwise
    pub fn get_by_path(&self, path: &str) -> Option<Repo> {
        self.data.records.get(path).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Data {
        let mut data = Data::new();
        // Add several repos with different names to test sorting
        data.record_item(
            "/base",
            "https://github.com/user/vscode.git",
            "github.com",
            "vscode",
            "user",
            "/base/github.com/user/vscode",
        );
        data.record_item(
            "/base",
            "https://github.com/user/prog.git",
            "github.com",
            "prog",
            "user",
            "/base/github.com/user/prog",
        );
        data.record_item(
            "/base",
            "https://github.com/user/prog-cli.git",
            "github.com",
            "prog-cli",
            "user",
            "/base/github.com/user/prog-cli",
        );
        data.record_item(
            "/base",
            "https://github.com/user/my-prog-tools.git",
            "github.com",
            "my-prog-tools",
            "user",
            "/base/github.com/user/my-prog-tools",
        );
        data
    }

    #[test]
    fn test_find_exact_match_first() {
        let data = create_test_data();
        
        // Search for "prog" should return "prog" as exact match first (distance 0)
        let results = data.find("prog");
        
        assert!(!results.is_empty(), "Should find results");
        assert_eq!(results[0].repo, "prog", "Exact match 'prog' should be first (Levenshtein distance 0)");
    }

    #[test]
    fn test_find_sorted_by_levenshtein_distance() {
        let data = create_test_data();
        
        // Search for "prog"
        // - "prog" has distance 0 (exact match)
        // - "prog-cli" has distance 4 (4 insertions: '-', 'c', 'l', 'i')
        // - "my-prog-tools" has distance 9 (prefix 'my-' and suffix '-tools')
        let results = data.find("prog");
        
        // First result should be exact match
        assert_eq!(results[0].repo, "prog", "Exact match should be first");
        
        // Verify results are sorted by Levenshtein distance
        let repo_names: Vec<&str> = results.iter().map(|r| r.repo.as_str()).collect();
        
        // "prog" (dist 0) should come before "prog-cli" (dist 4)
        let prog_pos = repo_names.iter().position(|&r| r == "prog").unwrap();
        let prog_cli_pos = repo_names.iter().position(|&r| r == "prog-cli").unwrap();
        assert!(prog_pos < prog_cli_pos, "prog should come before prog-cli");
        
        // "prog-cli" (dist 4) should come before "my-prog-tools" (dist 9)
        let my_prog_tools_pos = repo_names.iter().position(|&r| r == "my-prog-tools").unwrap();
        assert!(prog_cli_pos < my_prog_tools_pos, "prog-cli should come before my-prog-tools");
    }

    #[test]
    fn test_find_alphabetical_within_same_distance() {
        let mut data = Data::new();
        // Add repos that will match the keyword "prog" and have same Levenshtein distance
        data.record_item(
            "/base",
            "https://github.com/user/progs.git",
            "github.com",
            "progs",   // distance 1 from "prog" (1 insertion)
            "user",
            "/base/github.com/user/progs",
        );
        data.record_item(
            "/base",
            "https://github.com/user/progx.git",
            "github.com",
            "progx",   // distance 1 from "prog" (1 insertion)
            "user",
            "/base/github.com/user/progx",
        );
        data.record_item(
            "/base",
            "https://github.com/user/prog.git",
            "github.com",
            "prog",   // distance 0 from "prog"
            "user",
            "/base/github.com/user/prog",
        );
        
        let results = data.find("prog");
        
        // "prog" should be first (distance 0)
        assert_eq!(results[0].repo, "prog", "Exact match should be first");
        
        // "progs" and "progx" both have distance 1, should be alphabetically sorted
        // So "progs" should come before "progx"
        let repo_names: Vec<&str> = results.iter().skip(1).map(|r| r.repo.as_str()).collect();
        assert_eq!(repo_names, vec!["progs", "progx"], "Same distance repos should be alphabetically sorted");
    }

    #[test]
    fn test_find_order_is_deterministic() {
        let data = create_test_data();
        
        // Run find multiple times and verify the order is always the same
        let results1 = data.find("prog");
        let results2 = data.find("prog");
        let results3 = data.find("prog");
        
        let order1: Vec<&str> = results1.iter().map(|r| r.repo.as_str()).collect();
        let order2: Vec<&str> = results2.iter().map(|r| r.repo.as_str()).collect();
        let order3: Vec<&str> = results3.iter().map(|r| r.repo.as_str()).collect();
        
        assert_eq!(order1, order2, "Find results should be deterministic");
        assert_eq!(order2, order3, "Find results should be deterministic");
    }
}
