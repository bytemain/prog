use super::models::*;
use serde::{Deserialize, Serialize, de::Deserializer, ser::Serializer};
use std::collections::{BTreeMap, BTreeSet};

/// A collection of repository records with O(log n) lookup by path using B-tree structure.
/// Uses BTreeMap for efficient ordered lookups, range queries, and sorted iteration.
///
/// Secondary indexes are maintained for fast lookups by:
/// - host (e.g., github.com, gitlab.com)
/// - owner (e.g., bytemain, microsoft)
/// - repo name (e.g., vscode, prog)
///
/// All indexes provide O(log n) lookup time.
#[derive(Debug, Clone)]
pub(crate) struct IndexedRecords {
    /// BTreeMap storing repository records with path as key, providing O(log n) lookups
    records: BTreeMap<String, Repo>,
    /// Secondary index: host -> set of paths for O(log n) host lookups
    host_index: BTreeMap<String, BTreeSet<String>>,
    /// Secondary index: owner -> set of paths for O(log n) owner lookups
    owner_index: BTreeMap<String, BTreeSet<String>>,
    /// Secondary index: repo name -> set of paths for O(log n) repo name lookups
    repo_index: BTreeMap<String, BTreeSet<String>>,
}

impl IndexedRecords {
    /// Creates a new empty IndexedRecords instance
    pub(crate) fn new() -> Self {
        Self {
            records: BTreeMap::new(),
            host_index: BTreeMap::new(),
            owner_index: BTreeMap::new(),
            repo_index: BTreeMap::new(),
        }
    }

    /// Adds a record to a secondary index
    fn add_to_index(index: &mut BTreeMap<String, BTreeSet<String>>, key: &str, path: &str) {
        index
            .entry(key.to_string())
            .or_insert_with(BTreeSet::new)
            .insert(path.to_string());
    }

    /// Removes a record from a secondary index
    fn remove_from_index(index: &mut BTreeMap<String, BTreeSet<String>>, key: &str, path: &str) {
        if let Some(paths) = index.get_mut(key) {
            paths.remove(path);
            if paths.is_empty() {
                index.remove(key);
            }
        }
    }

    /// Removes all secondary index entries for a record at the given path
    fn remove_from_all_indexes(&mut self, path: &str) {
        if let Some(record) = self.records.get(path) {
            Self::remove_from_index(&mut self.host_index, &record.host, path);
            Self::remove_from_index(&mut self.owner_index, &record.owner, path);
            Self::remove_from_index(&mut self.repo_index, &record.repo, path);
        }
    }

    /// Adds secondary index entries for a record at the given path
    fn add_to_all_indexes(&mut self, record: &Repo, path: &str) {
        Self::add_to_index(&mut self.host_index, &record.host, path);
        Self::add_to_index(&mut self.owner_index, &record.owner, path);
        Self::add_to_index(&mut self.repo_index, &record.repo, path);
    }

    /// Adds a new repository record to the collection
    ///
    /// The record is added to the BTreeMap with its path as the key,
    /// maintaining sorted order for efficient range queries.
    /// Secondary indexes are also updated.
    ///
    /// # Arguments
    ///
    /// * `record` - The repository record to add
    pub(crate) fn add(&mut self, record: Repo) {
        let path = record.full_path.clone();

        // Remove old index entries if this path already exists
        self.remove_from_all_indexes(&path);

        // Add to secondary indexes
        self.add_to_all_indexes(&record, &path);

        self.records.insert(path, record);
    }

    pub(crate) fn insert(&mut self, path: &str, record: Repo) {
        // Remove old index entries if this path already exists
        self.remove_from_all_indexes(path);

        // Add to secondary indexes
        self.add_to_all_indexes(&record, path);

        self.records.insert(path.to_string(), record);
    }

    /// Removes a repository record by its path
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the repository to remove
    ///
    /// # Returns
    ///
    /// * `true` if the record was found and removed
    /// * `false` if no record with the given path exists
    pub(crate) fn remove(&mut self, path: &str) -> bool {
        if let Some(record) = self.records.remove(path) {
            // Remove from secondary indexes
            Self::remove_from_index(&mut self.host_index, &record.host, path);
            Self::remove_from_index(&mut self.owner_index, &record.owner, path);
            Self::remove_from_index(&mut self.repo_index, &record.repo, path);
            true
        } else {
            false
        }
    }

    pub(crate) fn get(&self, path: &str) -> Option<&Repo> {
        self.records.get(path)
    }

    /// Returns a vector containing all records in sorted order by path
    ///
    /// # Returns
    ///
    /// * Vector of all repository records sorted by full_path (already sorted in BTreeMap)
    pub(crate) fn get_all_sorted(&self) -> Vec<Repo> {
        // BTreeMap already maintains sorted order by key (full_path)
        self.records.values().cloned().collect()
    }

    /// Creates an IndexedRecords instance from a vector of repository records.
    /// This method rebuilds all secondary indexes from scratch.
    ///
    /// # Arguments
    ///
    /// * `records` - The vector of repository records
    ///
    /// # Returns
    ///
    /// * An IndexedRecords instance containing all the provided records
    #[allow(dead_code)]
    fn from_vec(records: Vec<Repo>) -> Self {
        let mut indexed = Self::new();
        for record in records {
            indexed.add(record);
        }
        indexed
    }

    /// Converts the IndexedRecords to a Vec, maintaining sorted order
    ///
    /// # Returns
    ///
    /// * A vector of all repository records in sorted order by path
    fn to_vec(&self) -> Vec<Repo> {
        self.records.values().cloned().collect()
    }

    pub fn size(&self) -> usize {
        self.records.len()
    }

    /// Returns an iterator over all records in sorted order by path.
    ///
    /// This is more efficient than `get_all_sorted()` when you only need to iterate
    /// once and don't need to store the results, as it avoids cloning all records.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Repo> {
        self.records.values()
    }

    /// Returns records with paths starting with the given prefix using efficient B-tree range query.
    ///
    /// This operation is O(log n + k) where n is the total number of records and k is the number
    /// of matching records, making it significantly faster than a linear scan for prefix searches.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The path prefix to search for
    ///
    /// # Returns
    ///
    /// * Vector of repository records whose paths start with the given prefix
    pub(crate) fn get_by_prefix(&self, prefix: &str) -> Vec<&Repo> {
        // Use range query to efficiently find all paths starting with the prefix
        // Filter by starts_with to handle edge cases with special characters
        self.records
            .range(prefix.to_string()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .map(|(_, v)| v)
            .collect()
    }

    /// Helper method to get records by paths from an index
    fn get_records_by_paths<'a>(&'a self, paths: Option<&BTreeSet<String>>) -> Vec<&'a Repo> {
        match paths {
            Some(path_set) => path_set
                .iter()
                .filter_map(|path| self.records.get(path))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Returns all repositories hosted on a specific host (e.g., github.com, gitlab.com)
    ///
    /// This operation uses the host secondary index for O(log n) lookup time,
    /// where n is the number of unique hosts.
    ///
    /// # Arguments
    ///
    /// * `host` - The host name to search for (e.g., "github.com")
    ///
    /// # Returns
    ///
    /// * Vector of repository records hosted on the specified host
    pub(crate) fn get_by_host(&self, host: &str) -> Vec<&Repo> {
        self.get_records_by_paths(self.host_index.get(host))
    }

    /// Returns all repositories owned by a specific owner
    ///
    /// This operation uses the owner secondary index for O(log n) lookup time,
    /// where n is the number of unique owners.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner name to search for (e.g., "microsoft", "bytemain")
    ///
    /// # Returns
    ///
    /// * Vector of repository records owned by the specified owner
    pub(crate) fn get_by_owner(&self, owner: &str) -> Vec<&Repo> {
        self.get_records_by_paths(self.owner_index.get(owner))
    }

    /// Returns all repositories with a specific repo name
    ///
    /// This operation uses the repo name secondary index for O(log n) lookup time,
    /// where n is the number of unique repo names.
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository name to search for (e.g., "vscode", "prog")
    ///
    /// # Returns
    ///
    /// * Vector of repository records with the specified repo name
    pub(crate) fn get_by_repo(&self, repo: &str) -> Vec<&Repo> {
        self.get_records_by_paths(self.repo_index.get(repo))
    }

    /// Returns all unique host names in the collection
    ///
    /// # Returns
    ///
    /// * Vector of unique host names, sorted alphabetically
    pub(crate) fn get_all_hosts(&self) -> Vec<&str> {
        self.host_index.keys().map(|s| s.as_str()).collect()
    }

    /// Returns all unique owner names in the collection
    ///
    /// # Returns
    ///
    /// * Vector of unique owner names, sorted alphabetically
    pub(crate) fn get_all_owners(&self) -> Vec<&str> {
        self.owner_index.keys().map(|s| s.as_str()).collect()
    }

    /// Returns all unique repository names in the collection
    ///
    /// # Returns
    ///
    /// * Vector of unique repository names, sorted alphabetically
    pub(crate) fn get_all_repos(&self) -> Vec<&str> {
        self.repo_index.keys().map(|s| s.as_str()).collect()
    }
}

/// Helper struct for serialization that includes both records and indexes.
/// This allows persisting indexes to disk to avoid rebuilding them on every load.
#[derive(Serialize, Deserialize)]
struct SerializedIndexedRecords {
    /// The primary records as a vector
    records: Vec<Repo>,
    /// Secondary index: host -> set of paths
    host_index: BTreeMap<String, BTreeSet<String>>,
    /// Secondary index: owner -> set of paths
    owner_index: BTreeMap<String, BTreeSet<String>>,
    /// Secondary index: repo name -> set of paths
    repo_index: BTreeMap<String, BTreeSet<String>>,
}

/// Custom serialization implementation that persists both records and indexes.
///
/// This approach trades storage space for faster load times by persisting
/// the secondary indexes along with the primary records, eliminating the
/// need to rebuild indexes on every deserialization.
impl Serialize for IndexedRecords {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize both records and indexes for fast loading
        let serialized = SerializedIndexedRecords {
            records: self.to_vec(),
            host_index: self.host_index.clone(),
            owner_index: self.owner_index.clone(),
            repo_index: self.repo_index.clone(),
        };
        serialized.serialize(serializer)
    }
}

/// Custom deserialization implementation that restores both records and indexes.
///
/// Indexes are loaded directly from disk rather than being rebuilt,
/// providing faster load times at the cost of slightly larger storage.
impl<'de> Deserialize<'de> for IndexedRecords {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the complete structure including indexes
        let serialized = SerializedIndexedRecords::deserialize(deserializer)?;

        // Build the primary records map
        let mut records = BTreeMap::new();
        for record in serialized.records {
            records.insert(record.full_path.clone(), record);
        }

        Ok(IndexedRecords {
            records,
            host_index: serialized.host_index,
            owner_index: serialized.owner_index,
            repo_index: serialized.repo_index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repo(full_path: &str, repo: &str) -> Repo {
        let now = chrono::Utc::now().naive_utc();
        Repo {
            created_at: now,
            updated_at: now,
            host: "github.com".to_string(),
            repo: repo.to_string(),
            owner: "user".to_string(),
            remote_url: format!("https://github.com/user/{}.git", repo),
            base_dir: "/base".to_string(),
            full_path: full_path.to_string(),
        }
    }

    fn create_test_repo_with_details(
        full_path: &str,
        repo: &str,
        host: &str,
        owner: &str,
    ) -> Repo {
        let now = chrono::Utc::now().naive_utc();
        Repo {
            created_at: now,
            updated_at: now,
            host: host.to_string(),
            repo: repo.to_string(),
            owner: owner.to_string(),
            remote_url: format!("https://{}/{}/{}.git", host, owner, repo),
            base_dir: "/base".to_string(),
            full_path: full_path.to_string(),
        }
    }

    #[test]
    fn test_btree_sorted_iteration() {
        let mut records = IndexedRecords::new();

        // Add records in non-sorted order
        records.add(create_test_repo("/base/github.com/user/zebra", "zebra"));
        records.add(create_test_repo("/base/github.com/user/apple", "apple"));
        records.add(create_test_repo("/base/github.com/user/mango", "mango"));

        // BTreeMap should maintain sorted order
        let sorted = records.get_all_sorted();
        let paths: Vec<&str> = sorted.iter().map(|r| r.full_path.as_str()).collect();

        assert_eq!(
            paths,
            vec![
                "/base/github.com/user/apple",
                "/base/github.com/user/mango",
                "/base/github.com/user/zebra",
            ]
        );
    }

    #[test]
    fn test_btree_prefix_query() {
        let mut records = IndexedRecords::new();

        // Add records with different prefixes
        records.add(create_test_repo("/base/github.com/user/prog", "prog"));
        records.add(create_test_repo("/base/github.com/user/prog-cli", "prog-cli"));
        records.add(create_test_repo("/base/github.com/user/prog-tools", "prog-tools"));
        records.add(create_test_repo("/base/github.com/other/prog", "prog"));
        records.add(create_test_repo("/base/gitlab.com/user/prog", "prog"));

        // Query by prefix - should use efficient B-tree range query
        let results = records.get_by_prefix("/base/github.com/user/prog");
        let paths: Vec<&str> = results.iter().map(|r| r.full_path.as_str()).collect();

        assert_eq!(
            paths,
            vec![
                "/base/github.com/user/prog",
                "/base/github.com/user/prog-cli",
                "/base/github.com/user/prog-tools",
            ]
        );
    }

    #[test]
    fn test_btree_prefix_query_no_matches() {
        let mut records = IndexedRecords::new();
        records.add(create_test_repo("/base/github.com/user/vscode", "vscode"));

        let results = records.get_by_prefix("/base/gitlab.com");
        assert!(results.is_empty());
    }

    #[test]
    fn test_btree_lookup_performance() {
        let mut records = IndexedRecords::new();

        // Add many records to test B-tree efficiency
        for i in 0..100 {
            let path = format!("/base/github.com/user/repo{:03}", i);
            let name = format!("repo{:03}", i);
            records.add(create_test_repo(&path, &name));
        }

        // Direct lookup should be O(log n)
        assert!(records.get("/base/github.com/user/repo050").is_some());
        assert!(records.get("/base/github.com/user/repo099").is_some());
        assert!(records.get("/base/github.com/user/repo000").is_some());
        assert!(records.get("/base/github.com/user/nonexistent").is_none());
    }

    #[test]
    fn test_btree_serialization_roundtrip() {
        // Wrap IndexedRecords in a struct for TOML serialization
        // (TOML doesn't support arrays at the root level)
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            records: IndexedRecords,
        }

        let mut records = IndexedRecords::new();
        records.add(create_test_repo("/base/github.com/user/zebra", "zebra"));
        records.add(create_test_repo("/base/github.com/user/apple", "apple"));

        let wrapper = Wrapper { records };

        // Serialize
        let serialized = toml::to_string(&wrapper).unwrap();

        // Deserialize
        let deserialized: Wrapper = toml::from_str(&serialized).unwrap();

        // Verify the data is preserved and sorted
        assert_eq!(deserialized.records.size(), 2);
        assert!(deserialized.records.get("/base/github.com/user/zebra").is_some());
        assert!(deserialized.records.get("/base/github.com/user/apple").is_some());

        // Verify sorting is maintained after deserialization
        let sorted = deserialized.records.get_all_sorted();
        assert_eq!(sorted[0].full_path, "/base/github.com/user/apple");
        assert_eq!(sorted[1].full_path, "/base/github.com/user/zebra");

        // Verify secondary indexes are rebuilt after deserialization
        let by_host = deserialized.records.get_by_host("github.com");
        assert_eq!(by_host.len(), 2);

        let by_owner = deserialized.records.get_by_owner("user");
        assert_eq!(by_owner.len(), 2);
    }

    #[test]
    fn test_secondary_index_by_host() {
        let mut records = IndexedRecords::new();

        // Add records with different hosts
        records.add(create_test_repo_with_details(
            "/base/github.com/user/repo1",
            "repo1",
            "github.com",
            "user",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/user/repo2",
            "repo2",
            "github.com",
            "user",
        ));
        records.add(create_test_repo_with_details(
            "/base/gitlab.com/user/repo3",
            "repo3",
            "gitlab.com",
            "user",
        ));
        records.add(create_test_repo_with_details(
            "/base/bitbucket.org/user/repo4",
            "repo4",
            "bitbucket.org",
            "user",
        ));

        // Query by host - should use efficient index lookup
        let github_repos = records.get_by_host("github.com");
        assert_eq!(github_repos.len(), 2);
        assert!(github_repos.iter().all(|r| r.host == "github.com"));

        let gitlab_repos = records.get_by_host("gitlab.com");
        assert_eq!(gitlab_repos.len(), 1);
        assert_eq!(gitlab_repos[0].repo, "repo3");

        let nonexistent = records.get_by_host("nonexistent.com");
        assert!(nonexistent.is_empty());
    }

    #[test]
    fn test_secondary_index_by_owner() {
        let mut records = IndexedRecords::new();

        // Add records with different owners
        records.add(create_test_repo_with_details(
            "/base/github.com/microsoft/vscode",
            "vscode",
            "github.com",
            "microsoft",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/microsoft/typescript",
            "typescript",
            "github.com",
            "microsoft",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/bytemain/prog",
            "prog",
            "github.com",
            "bytemain",
        ));

        // Query by owner
        let microsoft_repos = records.get_by_owner("microsoft");
        assert_eq!(microsoft_repos.len(), 2);
        assert!(microsoft_repos.iter().all(|r| r.owner == "microsoft"));

        let bytemain_repos = records.get_by_owner("bytemain");
        assert_eq!(bytemain_repos.len(), 1);
        assert_eq!(bytemain_repos[0].repo, "prog");

        let nonexistent = records.get_by_owner("nonexistent");
        assert!(nonexistent.is_empty());
    }

    #[test]
    fn test_secondary_index_by_repo() {
        let mut records = IndexedRecords::new();

        // Add records with same repo name but different owners/hosts
        records.add(create_test_repo_with_details(
            "/base/github.com/user1/prog",
            "prog",
            "github.com",
            "user1",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/user2/prog",
            "prog",
            "github.com",
            "user2",
        ));
        records.add(create_test_repo_with_details(
            "/base/gitlab.com/user1/prog",
            "prog",
            "gitlab.com",
            "user1",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/user1/vscode",
            "vscode",
            "github.com",
            "user1",
        ));

        // Query by repo name
        let prog_repos = records.get_by_repo("prog");
        assert_eq!(prog_repos.len(), 3);
        assert!(prog_repos.iter().all(|r| r.repo == "prog"));

        let vscode_repos = records.get_by_repo("vscode");
        assert_eq!(vscode_repos.len(), 1);
        assert_eq!(vscode_repos[0].owner, "user1");

        let nonexistent = records.get_by_repo("nonexistent");
        assert!(nonexistent.is_empty());
    }

    #[test]
    fn test_index_updated_on_remove() {
        let mut records = IndexedRecords::new();

        records.add(create_test_repo_with_details(
            "/base/github.com/user/repo1",
            "repo1",
            "github.com",
            "user",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/user/repo2",
            "repo2",
            "github.com",
            "user",
        ));

        // Verify initial state
        assert_eq!(records.get_by_host("github.com").len(), 2);
        assert_eq!(records.get_by_owner("user").len(), 2);

        // Remove one record
        assert!(records.remove("/base/github.com/user/repo1"));

        // Verify indexes are updated
        assert_eq!(records.get_by_host("github.com").len(), 1);
        assert_eq!(records.get_by_owner("user").len(), 1);
        assert_eq!(records.get_by_repo("repo1").len(), 0);
        assert_eq!(records.get_by_repo("repo2").len(), 1);
    }

    #[test]
    fn test_index_updated_on_update() {
        let mut records = IndexedRecords::new();

        // Add initial record
        records.add(create_test_repo_with_details(
            "/base/github.com/user/repo",
            "repo",
            "github.com",
            "user",
        ));

        assert_eq!(records.get_by_host("github.com").len(), 1);
        assert_eq!(records.get_by_host("gitlab.com").len(), 0);

        // Update the record by inserting at the same path but different host
        let now = chrono::Utc::now().naive_utc();
        let updated_repo = Repo {
            created_at: now,
            updated_at: now,
            host: "gitlab.com".to_string(),
            repo: "repo".to_string(),
            owner: "newowner".to_string(),
            remote_url: "https://gitlab.com/newowner/repo.git".to_string(),
            base_dir: "/base".to_string(),
            full_path: "/base/github.com/user/repo".to_string(),
        };
        records.insert("/base/github.com/user/repo", updated_repo);

        // Verify old index entry is removed and new one is added
        assert_eq!(records.get_by_host("github.com").len(), 0);
        assert_eq!(records.get_by_host("gitlab.com").len(), 1);
        assert_eq!(records.get_by_owner("user").len(), 0);
        assert_eq!(records.get_by_owner("newowner").len(), 1);
    }

    #[test]
    fn test_get_all_hosts() {
        let mut records = IndexedRecords::new();

        records.add(create_test_repo_with_details(
            "/base/github.com/user/repo1",
            "repo1",
            "github.com",
            "user",
        ));
        records.add(create_test_repo_with_details(
            "/base/gitlab.com/user/repo2",
            "repo2",
            "gitlab.com",
            "user",
        ));
        records.add(create_test_repo_with_details(
            "/base/bitbucket.org/user/repo3",
            "repo3",
            "bitbucket.org",
            "user",
        ));

        let hosts = records.get_all_hosts();
        assert_eq!(hosts.len(), 3);
        // BTreeMap maintains sorted order
        assert_eq!(hosts, vec!["bitbucket.org", "github.com", "gitlab.com"]);
    }

    #[test]
    fn test_get_all_owners() {
        let mut records = IndexedRecords::new();

        records.add(create_test_repo_with_details(
            "/base/github.com/alice/repo1",
            "repo1",
            "github.com",
            "alice",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/bob/repo2",
            "repo2",
            "github.com",
            "bob",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/charlie/repo3",
            "repo3",
            "github.com",
            "charlie",
        ));

        let owners = records.get_all_owners();
        assert_eq!(owners.len(), 3);
        assert_eq!(owners, vec!["alice", "bob", "charlie"]);
    }

    #[test]
    fn test_get_all_repos() {
        let mut records = IndexedRecords::new();

        records.add(create_test_repo_with_details(
            "/base/github.com/user/alpha",
            "alpha",
            "github.com",
            "user",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/user/beta",
            "beta",
            "github.com",
            "user",
        ));
        records.add(create_test_repo_with_details(
            "/base/github.com/user/gamma",
            "gamma",
            "github.com",
            "user",
        ));

        let repos = records.get_all_repos();
        assert_eq!(repos.len(), 3);
        assert_eq!(repos, vec!["alpha", "beta", "gamma"]);
    }
}
