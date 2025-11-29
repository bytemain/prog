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

        // Get all matching records
        let mut results: Vec<Repo> = self
            .records
            .get_all_sorted()
            .iter()
            .filter(|r| {
                r.full_path.to_lowercase().contains(&keyword)
                    || r.remote_url.to_lowercase().contains(&keyword)
            })
            .cloned()
            .collect();

        // Sort results by match relevance
        results.sort_by(|a, b| {
            // Define match priority enum
            #[derive(PartialEq, Eq, PartialOrd, Ord)]
            enum MatchPriority {
                Low,
                Middle,
                High,
            }

            // Define match priority function
            let match_priority = |repo: &Repo| -> MatchPriority {
                // Repository name exact match (highest priority)
                if repo.repo.to_lowercase() == keyword {
                    return MatchPriority::High;
                }
                // Repository name partial match (medium priority)
                else if repo.repo.to_lowercase().contains(&keyword) {
                    return MatchPriority::Middle;
                }
                // Other matches (lowest priority)
                else {
                    return MatchPriority::Low;
                }
            };

            // First sort by match priority (descending)
            let priority_cmp = match_priority(b).cmp(&match_priority(a));
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }

            // If match priority is the same, sort by repository name alphabetically
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
        
        // Search for "prog" should return "prog" as exact match first
        let results = data.find("prog");
        
        assert!(!results.is_empty(), "Should find results");
        assert_eq!(results[0].repo, "prog", "Exact match 'prog' should be first");
    }

    #[test]
    fn test_find_partial_match_ordering() {
        let data = create_test_data();
        
        // Search for "prog" should have partial matches after exact match
        let results = data.find("prog");
        
        // First result should be exact match
        assert_eq!(results[0].repo, "prog", "Exact match should be first");
        
        // The rest should be partial matches (repos containing "prog")
        let partial_matches: Vec<&str> = results.iter().skip(1).map(|r| r.repo.as_str()).collect();
        assert!(partial_matches.contains(&"prog-cli") || partial_matches.contains(&"my-prog-tools"),
            "Partial matches should follow exact match");
    }

    #[test]
    fn test_find_alphabetical_within_same_priority() {
        let data = create_test_data();
        
        // Search for "prog" - partial matches should be alphabetically sorted
        let results = data.find("prog");
        
        // Skip the exact match ("prog")
        let partial_matches: Vec<&str> = results.iter()
            .filter(|r| r.repo != "prog")
            .map(|r| r.repo.as_str())
            .collect();
        
        // Verify alphabetical order among partial matches
        let mut sorted_matches = partial_matches.clone();
        sorted_matches.sort();
        
        assert_eq!(partial_matches, sorted_matches, 
            "Partial matches should be alphabetically sorted");
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
