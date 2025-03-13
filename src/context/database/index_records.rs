use super::models::*;
use serde::{Deserialize, Serialize, de::Deserializer, ser::Serializer};
use std::collections::HashMap;

/// A collection of repository records with O(1) lookup by path.
/// This structure maintains an index of paths to their positions in the records vector,
/// providing efficient lookup, addition, and removal operations.
#[derive(Debug, Clone)]
pub(crate) struct IndexedRecords {
    /// The vector containing all repository records
    records: Vec<Repo>,
    /// A hashmap that maps repository paths to their indices in the records vector
    path_index: HashMap<String, usize>,
}

impl IndexedRecords {
    /// Creates a new empty IndexedRecords instance
    pub(crate) fn new() -> Self {
        Self {
            records: Vec::new(),
            path_index: HashMap::new(),
        }
    }

    /// Adds a new repository record to the collection
    ///
    /// The record is appended to the records vector and its path is indexed
    /// in the path_index hashmap.
    ///
    /// # Arguments
    ///
    /// * `record` - The repository record to add
    pub(crate) fn add(&mut self, record: Repo) {
        let path = record.full_path.clone();
        self.path_index.insert(path, self.records.len());
        self.records.push(record);
    }

    /// Removes a repository record by its path
    ///
    /// This method:
    /// 1. Removes the entry from the path_index hashmap
    /// 2. Removes the record from the records vector
    /// 3. Updates indices in path_index for all records after the removed one
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
        if let Some(index) = self.path_index.remove(path) {
            self.records.remove(index);
            // Update indices for all records after the removed one
            // since they all shifted one position back
            for (i, record) in self.records.iter().enumerate().skip(index) {
                self.path_index.insert(record.full_path.clone(), i);
            }
            return true;
        }
        false
    }

    /// Checks if a repository with the given path exists in the collection
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check for
    ///
    /// # Returns
    ///
    /// * `true` if a record with the given path exists
    /// * `false` otherwise
    pub(crate) fn contains(&self, path: &str) -> bool {
        self.path_index.contains_key(path)
    }

    /// Returns a reference to the vector containing all records
    ///
    /// # Returns
    ///
    /// * Reference to the vector of all repository records
    pub(crate) fn get_all(&self) -> &Vec<Repo> {
        &self.records
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
}

/// Custom serialization implementation that only serializes the records vector
impl Serialize for IndexedRecords {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Only serialize the records, not the index
        self.records.serialize(serializer)
    }
}

/// Custom deserialization implementation that reconstructs the path index
impl<'de> Deserialize<'de> for IndexedRecords {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as a vector of Repo objects
        let records = Vec::<Repo>::deserialize(deserializer)?;
        // Rebuild the index using from_vec
        Ok(IndexedRecords::from_vec(records))
    }
}