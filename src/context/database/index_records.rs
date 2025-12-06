use super::models::*;
use serde::{Deserialize, Serialize, de::Deserializer, ser::Serializer};
use std::collections::BTreeMap;

/// A collection of repository records with O(log n) lookup by path using B-tree structure.
/// Uses BTreeMap for efficient ordered lookups, range queries, and sorted iteration.
#[derive(Debug, Clone)]
pub(crate) struct IndexedRecords {
    /// BTreeMap storing repository records with path as key, providing O(log n) lookups
    records: BTreeMap<String, Repo>,
}

impl IndexedRecords {
    /// Creates a new empty IndexedRecords instance
    pub(crate) fn new() -> Self {
        Self { records: BTreeMap::new() }
    }

    /// Adds a new repository record to the collection
    ///
    /// The record is added to the BTreeMap with its path as the key,
    /// maintaining sorted order for efficient range queries.
    ///
    /// # Arguments
    ///
    /// * `record` - The repository record to add
    pub(crate) fn add(&mut self, record: Repo) {
        let path = record.full_path.clone();
        self.records.insert(path, record);
    }

    pub(crate) fn insert(&mut self, path: &str, record: Repo) {
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
        self.records.remove(path).is_some()
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

    /// Creates an IndexedRecords instance from a vector of repository records
    ///
    /// # Arguments
    ///
    /// * `records` - The vector of repository records
    ///
    /// # Returns
    ///
    /// * An IndexedRecords instance containing all the provided records
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
}

/// Custom serialization implementation that serializes as a vector to maintain order
impl Serialize for IndexedRecords {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert to vector maintaining order and serialize
        let ordered_records: Vec<Repo> = self.to_vec();
        ordered_records.serialize(serializer)
    }
}

/// Custom deserialization implementation that builds a BTreeMap from the vector
impl<'de> Deserialize<'de> for IndexedRecords {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as a vector of Repo objects
        let records = Vec::<Repo>::deserialize(deserializer)?;
        // Build the IndexedRecords from the vector
        Ok(IndexedRecords::from_vec(records))
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
    }
}
