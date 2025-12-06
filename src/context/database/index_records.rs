use super::models::*;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize, de::Deserializer, ser::Serializer};

/// A collection of repository records with O(1) lookup by path while maintaining insertion order.
/// Uses LinkedHashMap to combine the benefits of HashMap (fast lookups) and Vec (preserved order).
#[derive(Debug, Clone)]
pub(crate) struct IndexedRecords {
    /// LinkedHashMap storing repository records with path as key, preserving insertion order
    records: LinkedHashMap<String, Repo>,
}

impl IndexedRecords {
    /// Creates a new empty IndexedRecords instance
    pub(crate) fn new() -> Self {
        Self { records: LinkedHashMap::new() }
    }

    /// Adds a new repository record to the collection
    ///
    /// The record is added to the LinkedHashMap with its path as the key,
    /// preserving insertion order.
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

    /// Returns a vector containing all records in their insertion order
    ///
    /// # Returns
    ///
    /// * Vector of all repository records in insertion order (owned values, not references)
    pub(crate) fn get_all_sorted(&self) -> Vec<Repo> {
        let mut items: Vec<Repo> = self.records.values().cloned().collect();
        items.sort_by(|a, b| a.full_path.cmp(&b.full_path));
        items
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

    /// Converts the IndexedRecords to a Vec, maintaining insertion order
    ///
    /// # Returns
    ///
    /// * A vector of all repository records in insertion order
    fn to_vec(&self) -> Vec<Repo> {
        self.records.values().cloned().collect()
    }

    pub fn size(&self) -> usize {
        self.records.len()
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

/// Custom deserialization implementation that builds a LinkedHashMap from the vector
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
