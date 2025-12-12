use std::{fs, path::PathBuf};
use std::path::Path;

use dirs::home_dir;

pub fn join_home_dir(path: &str) -> PathBuf {
    let mut home_dir = home_dir().unwrap();
    home_dir.push(path);
    home_dir
}

pub const PROGRAM: &str = "prog";
const FOLDER: &str = ".prog";
/// macOS metadata file that should be ignored when checking if a directory is empty
const DS_STORE_FILE: &str = ".DS_Store";

pub const DATA_FOLDER: &str = "data";

pub fn get_config_path(file: &str) -> PathBuf {
    let mut config_dir = join_home_dir(FOLDER);
    config_dir.push(file);
    config_dir
}

pub fn expand_path(path: &str) -> PathBuf {
    PathBuf::from(expand_tilde(path))
}

pub fn ensure_dir_exists(path: &PathBuf) {
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

pub fn exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

/// Checks if a directory is effectively empty.
///
/// A directory is considered effectively empty if it contains no entries,
/// or if it only contains `.DS_Store` files (macOS metadata files).
///
/// # Arguments
///
/// * `path` - The path to the directory to check
///
/// # Returns
///
/// * `bool` - true if the directory is effectively empty, false otherwise
fn is_dir_effectively_empty(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                if file_name != DS_STORE_FILE {
                    return false;
                }
            }
            true
        }
        Err(_) => false,
    }
}

/// Removes .DS_Store file from a directory if it exists.
fn remove_ds_store_if_exists(path: &Path) {
    let ds_store_path = path.join(DS_STORE_FILE);
    if ds_store_path.exists() {
        let _ = fs::remove_file(ds_store_path);
    }
}

/// Removes a directory and all its empty parent directories recursively.
///
/// This function first removes the target directory using `remove_dir_all`,
/// then checks if parent directories are empty and removes them if they are.
/// Directories containing only `.DS_Store` files are treated as empty.
///
/// # Arguments
///
/// * `path` - The path to the directory to remove
/// * `stop_at` - Optional path to stop the recursive deletion at (will not remove this directory)
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - Ok if successful, Err with the IO error otherwise
pub fn remove_dir_with_empty_parents(
    path: &PathBuf,
    stop_at: Option<&PathBuf>,
) -> Result<(), std::io::Error> {
    // First remove the target directory
    fs::remove_dir_all(path)?;

    // Get the parent directory
    let mut current = path.parent().map(PathBuf::from);

    // Recursively check and remove empty parent directories
    while let Some(parent) = current {
        // Stop if we've reached the stop_at directory
        if let Some(stop_dir) = stop_at {
            if parent == *stop_dir {
                break;
            }
        }

        // Check if directory is effectively empty (empty or only contains .DS_Store)
        if is_dir_effectively_empty(&parent) {
            // First remove .DS_Store if it exists, then remove the empty directory
            remove_ds_store_if_exists(&parent);
            match fs::remove_dir(&parent) {
                Ok(_) => {
                    // Continue with the next parent
                    current = parent.parent().map(PathBuf::from);
                }
                Err(_) => break, // If we can't remove the directory, stop
            }
        } else {
            break; // Directory is not empty, stop
        }
    }

    Ok(())
}

/// Contracts a path by replacing the home directory with tilde (~)
///
/// # Arguments
/// * `path` - A string slice that might contain the user's home directory path
///
/// # Returns
/// A String with the home directory path replaced with ~ if it starts with home directory
///
/// # Examples
/// ```
/// let contracted = contract_tilde("/home/username/Documents");
/// // Result would be "~/Documents"
/// ```
pub fn contract_tilde(path: &str) -> String {
    if let Some(home_dir) = dirs::home_dir() {
        let home_str = home_dir.to_string_lossy();
        if path.starts_with(home_str.as_ref()) {
            if path.len() == home_str.len() {
                // Path is exactly the home directory
                return "~".to_string();
            } else if path.chars().nth(home_str.len()) == Some('/') {
                // Path starts with home directory followed by a slash
                return format!("~{}", &path[home_str.len()..]);
            }
        }
    }
    // No home directory found or path doesn't start with home directory
    path.to_string()
}

/// Expands the tilde character (~) in a path string to the user's home directory
///
/// # Arguments
/// * `path` - A string slice that might contain a tilde (~) at the beginning
///
/// # Returns
/// A String with the tilde expanded to the user's home directory path
///
/// # Examples
/// ```
/// let expanded = expand_tilde("~/Documents");
/// // Result might be something like "/home/username/Documents" on Linux/macOS
/// // or "C:\Users\username\Documents" on Windows
/// ```
pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~") {
        if let Some(home_dir) = dirs::home_dir() {
            if path.len() > 1 {
                // Handle ~/rest/of/path
                if let Some(rest) = path.strip_prefix("~/") {
                    return home_dir.join(rest).to_string_lossy().to_string();
                }
                // Handle ~rest/of/path (without slash)
                else if let Some(rest) = path.strip_prefix('~') {
                    return home_dir.join(rest).to_string_lossy().to_string();
                }
            }
            // Just ~ by itself
            return home_dir.to_string_lossy().to_string();
        }
    }
    // No tilde or couldn't get home directory
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        // Get the home directory for comparison
        let home = dirs::home_dir().unwrap().to_string_lossy().to_string();

        // Test case 1: Just the tilde
        assert_eq!(expand_tilde("~"), home);

        // Test case 2: Tilde with slash and path
        let docs_path = format!("{}/Documents", home);
        assert_eq!(expand_tilde("~/Documents"), docs_path);

        // Test case 3: Path without tilde should remain unchanged
        let normal_path = "/usr/local/bin";
        assert_eq!(expand_tilde(normal_path), normal_path);

        // Test case 4: Empty string should remain unchanged
        assert_eq!(expand_tilde(""), "");

        // Test case 5: Tilde in the middle of a path should remain unchanged
        let middle_tilde = "/usr/~local/bin";
        assert_eq!(expand_tilde(middle_tilde), middle_tilde);

        // Test case 6: Tilde without slash
        let no_slash_path = format!("{}/test", home);
        assert_eq!(expand_tilde("~test"), no_slash_path);
    }

    #[test]
    fn test_contract_tilde() {
        // Get the home directory for comparison
        let home = dirs::home_dir().unwrap().to_string_lossy().to_string();

        // Test case 1: Just the home directory
        assert_eq!(contract_tilde(&home), "~");

        // Test case 2: Home directory with path
        let docs_path = format!("{}/Documents", home);
        assert_eq!(contract_tilde(&docs_path), "~/Documents");

        // Test case 3: Path not starting with home directory should remain unchanged
        let normal_path = "/usr/local/bin";
        assert_eq!(contract_tilde(normal_path), normal_path);

        // Test case 4: Empty string should remain unchanged
        assert_eq!(contract_tilde(""), "");

        // Test case 5: Path that contains home directory but doesn't start with it
        let middle_home = format!("/usr{}/bin", home);
        assert_eq!(contract_tilde(&middle_home), middle_home);

        // Test case 6: Nested path under home directory
        let nested_path = format!("{}/Documents/Projects/rust", home);
        assert_eq!(contract_tilde(&nested_path), "~/Documents/Projects/rust");
    }

    #[test]
    fn test_is_dir_effectively_empty() {
        use std::fs::{self, File};
        use std::io::Write;

        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Test case 1: Actually empty directory
        let empty_dir = temp_path.join("empty");
        fs::create_dir(&empty_dir).unwrap();
        assert!(is_dir_effectively_empty(&empty_dir));

        // Test case 2: Directory with only .DS_Store
        let ds_store_dir = temp_path.join("ds_store_only");
        fs::create_dir(&ds_store_dir).unwrap();
        File::create(ds_store_dir.join(".DS_Store"))
            .unwrap()
            .write_all(b"test")
            .unwrap();
        assert!(is_dir_effectively_empty(&ds_store_dir));

        // Test case 3: Directory with other files
        let non_empty_dir = temp_path.join("non_empty");
        fs::create_dir(&non_empty_dir).unwrap();
        File::create(non_empty_dir.join("somefile.txt"))
            .unwrap()
            .write_all(b"test")
            .unwrap();
        assert!(!is_dir_effectively_empty(&non_empty_dir));

        // Test case 4: Directory with .DS_Store and other files
        let mixed_dir = temp_path.join("mixed");
        fs::create_dir(&mixed_dir).unwrap();
        File::create(mixed_dir.join(".DS_Store"))
            .unwrap()
            .write_all(b"test")
            .unwrap();
        File::create(mixed_dir.join("somefile.txt"))
            .unwrap()
            .write_all(b"test")
            .unwrap();
        assert!(!is_dir_effectively_empty(&mixed_dir));
    }

    #[test]
    fn test_remove_dir_with_empty_parents_removes_ds_store_only_dirs() {
        use std::fs::{self, File};
        use std::io::Write;

        // Create a temporary directory structure for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        // Create nested structure: base/parent/child
        // parent will contain only .DS_Store after child is removed
        let parent_dir = base_path.join("parent");
        let child_dir = parent_dir.join("child");
        fs::create_dir_all(&child_dir).unwrap();

        // Add .DS_Store to parent directory
        File::create(parent_dir.join(".DS_Store"))
            .unwrap()
            .write_all(b"test")
            .unwrap();

        // Remove child directory with empty parents cleanup
        remove_dir_with_empty_parents(&child_dir, Some(&base_path)).unwrap();

        // Parent should be removed because it only contained .DS_Store and child
        assert!(!parent_dir.exists());
        // Base should still exist (stop_at)
        assert!(base_path.exists());
    }

    #[test]
    fn test_remove_dir_with_empty_parents_keeps_non_empty_dirs() {
        use std::fs::{self, File};
        use std::io::Write;

        // Create a temporary directory structure for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        // Create nested structure: base/parent/child
        let parent_dir = base_path.join("parent");
        let child_dir = parent_dir.join("child");
        fs::create_dir_all(&child_dir).unwrap();

        // Add a regular file to parent directory (not just .DS_Store)
        File::create(parent_dir.join("important.txt"))
            .unwrap()
            .write_all(b"test")
            .unwrap();

        // Remove child directory with empty parents cleanup
        remove_dir_with_empty_parents(&child_dir, Some(&base_path)).unwrap();

        // Parent should still exist because it contains important.txt
        assert!(parent_dir.exists());
        assert!(parent_dir.join("important.txt").exists());
    }
}
