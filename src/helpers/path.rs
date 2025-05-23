use std::{fs, path::PathBuf};

use dirs::home_dir;

pub fn join_home_dir(path: &str) -> PathBuf {
    let mut home_dir = home_dir().unwrap();
    home_dir.push(path);
    home_dir
}

pub const PROGRAM: &str = "prog";
const FOLDER: &str = ".prog";

pub const DATA_FOLDER: &str = "data";

pub fn get_config_path(file: &str) -> PathBuf {
    let mut config_dir = join_home_dir(FOLDER);
    config_dir.push(file);
    config_dir
}

pub fn expand_path(path: &str) -> PathBuf {
    let path = shellexpand::tilde(path).into_owned();
    PathBuf::from(path)
}

pub fn ensure_dir_exists(path: &PathBuf) {
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

pub fn exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

/// Removes a directory and all its empty parent directories recursively.
///
/// This function first removes the target directory using `remove_dir_all`,
/// then checks if parent directories are empty and removes them if they are.
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

        // Check if directory is empty
        let is_empty = match fs::read_dir(&parent) {
            Ok(mut entries) => entries.next().is_none(),
            Err(_) => break, // If we can't read the directory, stop
        };

        if is_empty {
            // Remove the empty directory
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
